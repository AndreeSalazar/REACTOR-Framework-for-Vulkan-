//! Variable Rate Shading support for REACTOR's Pixel Inteligente path.
//!
//! The implementation uses `VK_KHR_fragment_shading_rate` in pipeline mode:
//! when the GPU exposes it, REACTOR can shade 2x1, 1x2, 2x2, or larger pixel
//! blocks per fragment invocation. If the extension is missing, all public
//! APIs gracefully fall back to native 1x1 shading.

use ash::vk;
use std::ffi::CStr;
use std::ptr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VrsRate {
    pub width: u32,
    pub height: u32,
}

impl VrsRate {
    pub const NATIVE: Self = Self { width: 1, height: 1 };
    pub const X2_HORIZONTAL: Self = Self { width: 2, height: 1 };
    pub const X2_VERTICAL: Self = Self { width: 1, height: 2 };
    pub const X4: Self = Self { width: 2, height: 2 };
    pub const X8: Self = Self { width: 4, height: 2 };
    pub const X16: Self = Self { width: 4, height: 4 };

    #[inline]
    pub fn area(self) -> u32 {
        self.width * self.height
    }

    #[inline]
    pub fn as_extent(self) -> vk::Extent2D {
        vk::Extent2D { width: self.width, height: self.height }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelIntelligentProfile {
    Off,
    Quality,
    Balanced,
    Performance,
    UltraPerformance,
}

#[derive(Clone, Debug)]
pub struct PixelIntelligent {
    pub enabled: bool,
    pub profile: PixelIntelligentProfile,
    pub current_rate: VrsRate,
    pub target_fps: f32,
    pub last_scene_objects: usize,
    pub last_pixels: u64,
}

impl Default for PixelIntelligent {
    fn default() -> Self {
        Self {
            enabled: true,
            profile: PixelIntelligentProfile::Balanced,
            current_rate: VrsRate::NATIVE,
            target_fps: 120.0,
            last_scene_objects: 0,
            last_pixels: 0,
        }
    }
}

impl PixelIntelligent {
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            profile: PixelIntelligentProfile::Off,
            ..Self::default()
        }
    }

    pub fn xenofall() -> Self {
        Self {
            enabled: true,
            profile: PixelIntelligentProfile::Performance,
            target_fps: 144.0,
            ..Self::default()
        }
    }

    pub fn set_profile(&mut self, profile: PixelIntelligentProfile) {
        self.profile = profile;
        self.enabled = profile != PixelIntelligentProfile::Off;
    }

    pub fn desired_rate(&mut self, extent: vk::Extent2D, visible_objects: usize) -> VrsRate {
        self.last_scene_objects = visible_objects;
        self.last_pixels = extent.width as u64 * extent.height as u64;

        self.current_rate = if !self.enabled {
            VrsRate::NATIVE
        } else {
            match self.profile {
                PixelIntelligentProfile::Off | PixelIntelligentProfile::Quality => VrsRate::NATIVE,
                PixelIntelligentProfile::Balanced => {
                    if self.last_pixels >= 2_560 * 1_440 || visible_objects > 128 {
                        VrsRate::X4
                    } else if self.last_pixels >= 1_920 * 1_080 || visible_objects > 64 {
                        VrsRate::X2_HORIZONTAL
                    } else {
                        VrsRate::NATIVE
                    }
                }
                PixelIntelligentProfile::Performance => {
                    if self.last_pixels >= 1_920 * 1_080 || visible_objects > 48 {
                        VrsRate::X4
                    } else {
                        VrsRate::X2_HORIZONTAL
                    }
                }
                PixelIntelligentProfile::UltraPerformance => VrsRate::X8,
            }
        };

        self.current_rate
    }

    pub fn theoretical_pixel_work_reduction(&self) -> f32 {
        if !self.enabled || self.current_rate.area() <= 1 {
            0.0
        } else {
            1.0 - (1.0 / self.current_rate.area() as f32)
        }
    }
}

#[derive(Clone, Debug)]
pub struct VrsSupportedRate {
    pub rate: VrsRate,
    pub sample_counts: vk::SampleCountFlags,
}

#[derive(Clone, Debug)]
pub struct VrsCapabilities {
    pub extension_supported: bool,
    pub pipeline_fragment_shading_rate: bool,
    pub primitive_fragment_shading_rate: bool,
    pub attachment_fragment_shading_rate: bool,
    pub non_trivial_combiner_ops: bool,
    pub min_attachment_texel_size: vk::Extent2D,
    pub max_attachment_texel_size: vk::Extent2D,
    pub rates: Vec<VrsSupportedRate>,
}

impl VrsCapabilities {
    pub fn unsupported() -> Self {
        Self {
            extension_supported: false,
            pipeline_fragment_shading_rate: false,
            primitive_fragment_shading_rate: false,
            attachment_fragment_shading_rate: false,
            non_trivial_combiner_ops: false,
            min_attachment_texel_size: vk::Extent2D { width: 1, height: 1 },
            max_attachment_texel_size: vk::Extent2D { width: 1, height: 1 },
            rates: vec![VrsSupportedRate {
                rate: VrsRate::NATIVE,
                sample_counts: vk::SampleCountFlags::TYPE_1,
            }],
        }
    }

    pub fn is_pipeline_ready(&self) -> bool {
        self.extension_supported && self.pipeline_fragment_shading_rate
    }

    pub fn best_supported_rate(&self, desired: VrsRate, samples: vk::SampleCountFlags) -> VrsRate {
        if !self.is_pipeline_ready() {
            return VrsRate::NATIVE;
        }

        let mut candidates: Vec<VrsRate> = self
            .rates
            .iter()
            .filter(|rate| {
                rate.sample_counts.contains(samples)
                    || (samples == vk::SampleCountFlags::TYPE_1
                        && rate.sample_counts.contains(vk::SampleCountFlags::TYPE_1))
            })
            .map(|rate| rate.rate)
            .collect();

        candidates.push(VrsRate::NATIVE);
        candidates.sort_by_key(|rate| rate.area());
        candidates.dedup();

        if candidates.contains(&desired) {
            return desired;
        }

        candidates
            .into_iter()
            .filter(|rate| rate.width <= desired.width && rate.height <= desired.height)
            .max_by_key(|rate| rate.area())
            .unwrap_or(VrsRate::NATIVE)
    }
}

#[derive(Clone)]
pub struct VrsContext {
    pub loader: ash::khr::fragment_shading_rate::Device,
    pub capabilities: VrsCapabilities,
}

impl VrsContext {
    pub fn new(
        instance: &ash::Instance,
        device: &ash::Device,
        capabilities: VrsCapabilities,
    ) -> Self {
        Self {
            loader: ash::khr::fragment_shading_rate::Device::new(instance, device),
            capabilities,
        }
    }

    pub unsafe fn cmd_set_rate(&self, command_buffer: vk::CommandBuffer, rate: VrsRate) {
        let extent = rate.as_extent();
        let combiner_ops = [
            vk::FragmentShadingRateCombinerOpKHR::KEEP,
            vk::FragmentShadingRateCombinerOpKHR::KEEP,
        ];
        (self.loader.fp().cmd_set_fragment_shading_rate_khr)(
            command_buffer,
            &extent,
            &combiner_ops,
        );
    }
}

pub fn extension_supported(instance: &ash::Instance, physical_device: vk::PhysicalDevice) -> bool {
    let exts = unsafe { instance.enumerate_device_extension_properties(physical_device) };
    exts.map(|exts| {
        exts.iter().any(|ext| {
            let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
            name == ash::khr::fragment_shading_rate::NAME
        })
    })
    .unwrap_or(false)
}

pub fn query_capabilities(
    entry: &ash::Entry,
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
) -> VrsCapabilities {
    if !extension_supported(instance, physical_device) {
        return VrsCapabilities::unsupported();
    }

    let mut features = vk::PhysicalDeviceFragmentShadingRateFeaturesKHR::default();
    let mut features2 = vk::PhysicalDeviceFeatures2::default().push_next(&mut features);
    unsafe {
        instance.get_physical_device_features2(physical_device, &mut features2);
    }

    let mut properties = vk::PhysicalDeviceFragmentShadingRatePropertiesKHR::default();
    let mut properties2 = vk::PhysicalDeviceProperties2::default().push_next(&mut properties);
    unsafe {
        instance.get_physical_device_properties2(physical_device, &mut properties2);
    }

    let rates = query_supported_rates(entry, instance, physical_device);

    VrsCapabilities {
        extension_supported: true,
        pipeline_fragment_shading_rate: features.pipeline_fragment_shading_rate == vk::TRUE,
        primitive_fragment_shading_rate: features.primitive_fragment_shading_rate == vk::TRUE,
        attachment_fragment_shading_rate: features.attachment_fragment_shading_rate == vk::TRUE,
        non_trivial_combiner_ops: properties.fragment_shading_rate_non_trivial_combiner_ops
            == vk::TRUE,
        min_attachment_texel_size: properties.min_fragment_shading_rate_attachment_texel_size,
        max_attachment_texel_size: properties.max_fragment_shading_rate_attachment_texel_size,
        rates,
    }
}

fn query_supported_rates(
    entry: &ash::Entry,
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
) -> Vec<VrsSupportedRate> {
    let loader = ash::khr::fragment_shading_rate::Instance::new(entry, instance);
    let mut count = 0u32;

    let result = unsafe {
        (loader.fp().get_physical_device_fragment_shading_rates_khr)(
            physical_device,
            &mut count,
            ptr::null_mut(),
        )
    };

    if result != vk::Result::SUCCESS || count == 0 {
        return vec![VrsSupportedRate {
            rate: VrsRate::NATIVE,
            sample_counts: vk::SampleCountFlags::TYPE_1,
        }];
    }

    let mut raw_rates = vec![vk::PhysicalDeviceFragmentShadingRateKHR::default(); count as usize];
    let result = unsafe {
        (loader.fp().get_physical_device_fragment_shading_rates_khr)(
            physical_device,
            &mut count,
            raw_rates.as_mut_ptr(),
        )
    };

    if result != vk::Result::SUCCESS {
        return vec![VrsSupportedRate {
            rate: VrsRate::NATIVE,
            sample_counts: vk::SampleCountFlags::TYPE_1,
        }];
    }

    raw_rates
        .into_iter()
        .map(|rate| VrsSupportedRate {
            rate: VrsRate {
                width: rate.fragment_size.width,
                height: rate.fragment_size.height,
            },
            sample_counts: rate.sample_counts,
        })
        .collect()
}
