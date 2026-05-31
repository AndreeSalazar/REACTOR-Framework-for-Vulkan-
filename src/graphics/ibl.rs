// =============================================================================
// REACTOR · graphics/ibl.rs — Image-Based Lighting (cubemap HDR real)
// =============================================================================
// Sistema de baking de IBL en GPU usando 4 compute shaders:
//
//   ╭─────────────────╮   ╭──────────────╮   ╭──────────────╮   ╭──────────╮
//   │ equirect HDR 2D │──▶│ radiance cube│──▶│ irradiance   │   │ BRDF LUT │
//   │  (RGBA16F)      │   │ (RGBA16F 6×) │   │ (32×32×6)    │   │ (512×512 │
//   ╰─────────────────╯   │              │──▶│ prefilt cube │   │  RG16F)  │
//                         ╰──────────────╯   │ (128, 5 mips)│   ╰──────────╯
//                                            ╰──────────────╯
//
// Salida: `IblTextures` con descriptor set listo para bindear al shader PBR.
//
// Convenciones:
//   • Todos los cubemaps en RGBA16F (32-bit float es overkill para preview).
//   • BRDF LUT en RG16F (2 canales: scale + bias del Fresnel).
//   • Orden de caras Vulkan: +X, -X, +Y, -Y, +Z, -Z.
//   • Una sola sumisión, espera por queue idle (operación one-shot al init).
// =============================================================================

use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use crate::core::VulkanContext;
use ash::vk;
use ash::util::read_spv;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator};
use gpu_allocator::MemoryLocation;
use std::io::Cursor;
use std::path::Path;
use std::sync::{Arc, Mutex};

// =============================================================================
// Configuración por defecto (Karis 2014 / UE4-like)
// =============================================================================

/// Resolución del cubemap base de radiancia.
pub const IBL_RADIANCE_SIZE: u32 = 1024;
/// Resolución del cubemap de irradiancia difusa.
pub const IBL_IRRADIANCE_SIZE: u32 = 32;
/// Resolución del cubemap especular prefiltrado (nivel 0).
pub const IBL_PREFILTER_SIZE: u32 = 128;
/// Niveles de mip del prefilter (1 = roughness 0, 2 = 0.25, ..., N = 1.0).
pub const IBL_PREFILTER_MIPS: u32 = 5;
/// Resolución de la BRDF LUT 2D.
pub const IBL_BRDF_LUT_SIZE: u32 = 512;

const RGBA16F: vk::Format = vk::Format::R16G16B16A16_SFLOAT;
const RG16F: vk::Format = vk::Format::R16G16_SFLOAT;

// =============================================================================
// Bytecode SPIR-V embebido — compilado por build.rs
// =============================================================================

const SPV_EQUIRECT_TO_CUBE: &[u8] = include_bytes!("../../shaders/ibl/equirect_to_cube.spv");
const SPV_IRRADIANCE: &[u8] = include_bytes!("../../shaders/ibl/irradiance.spv");
const SPV_PREFILTER: &[u8] = include_bytes!("../../shaders/ibl/prefilter.spv");
const SPV_BRDF_LUT: &[u8] = include_bytes!("../../shaders/ibl/brdf_lut.spv");

// =============================================================================
// Imagen GPU minimalista (raw handles — no usamos `Image` porque éste no
// soporta cubemaps ni vistas múltiples)
// =============================================================================

/// Imagen GPU con vistas múltiples (cubemap sampleable + 1 vista 2D-array por
/// mip para escritura desde compute).
pub struct IblImage {
    pub image: vk::Image,
    pub allocation: Option<Allocation>,
    /// View principal para sampling (samplerCube o sampler2D).
    pub view: vk::ImageView,
    /// Una view 2D-array por mip (storage image) — vacío para BRDF LUT.
    pub mip_views: Vec<vk::ImageView>,
    pub format: vk::Format,
    pub extent: vk::Extent3D,
    pub mip_levels: u32,
    pub layer_count: u32,
    device: ash::Device,
    allocator: Arc<Mutex<Allocator>>,
}

impl Drop for IblImage {
    fn drop(&mut self) {
        unsafe {
            for v in self.mip_views.drain(..) {
                self.device.destroy_image_view(v, None);
            }
            self.device.destroy_image_view(self.view, None);
            self.device.destroy_image(self.image, None);
        }
        if let Some(alloc) = self.allocation.take() {
            let _ = self.allocator.lock().unwrap().free(alloc);
        }
    }
}

// =============================================================================
// IblTextures — output del baker
// =============================================================================

/// Conjunto completo de texturas IBL más el descriptor set ya cableado.
///
/// El descriptor set usa **set = 1** por convención (set = 0 queda para los
/// uniforms por-objeto del shader que lo consume).
///
/// Layout (4 bindings):
/// ```text
///   binding 0 : samplerCube  irradiance   (FRAGMENT)
///   binding 1 : samplerCube  prefiltered  (FRAGMENT)
///   binding 2 : sampler2D    brdf_lut     (FRAGMENT)
///   binding 3 : uniform buf  IblParams    (FRAGMENT)
/// ```
pub struct IblTextures {
    pub irradiance: IblImage,
    pub prefiltered: IblImage,
    pub brdf_lut: IblImage,
    pub sampler_cube: vk::Sampler,
    pub sampler_2d: vk::Sampler,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_set: vk::DescriptorSet,
    pub params_buffer: vk::Buffer,
    pub params_allocation: Option<Allocation>,
    /// `roughness * max_mip_level` se usa en el shader; expone el valor para
    /// que un consumer también pueda hacerlo desde push constants si quiere.
    pub max_mip_level: f32,
    device: ash::Device,
    allocator: Arc<Mutex<Allocator>>,
}

impl Drop for IblTextures {
    fn drop(&mut self) {
        unsafe {
            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            self.device.destroy_sampler(self.sampler_cube, None);
            self.device.destroy_sampler(self.sampler_2d, None);
            self.device.destroy_buffer(self.params_buffer, None);
        }
        if let Some(a) = self.params_allocation.take() {
            let _ = self.allocator.lock().unwrap().free(a);
        }
    }
}

// =============================================================================
// IblBaker — entry points públicos
// =============================================================================

pub struct IblBaker;

impl IblBaker {
    /// Bake desde un archivo HDR equirectangular en disco (Radiance `.hdr`).
    pub fn bake_from_equirect_file(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        hdr_path: impl AsRef<Path>,
    ) -> ReactorResult<IblTextures> {
        let (pixels, w, h) = load_hdr_equirect(hdr_path.as_ref())?;
        Self::bake_from_equirect_pixels(ctx, allocator, &pixels, w, h)
    }

    /// Bake desde píxeles RGBA16F equirectangulares ya cargados en memoria.
    /// Útil para generar un sky procedural sin tocar el disco.
    pub fn bake_from_equirect_pixels(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
        pixels_rgba_f16: &[u16],
        width: u32,
        height: u32,
    ) -> ReactorResult<IblTextures> {
        assert_eq!(
            pixels_rgba_f16.len() as u32,
            width * height * 4,
            "equirect HDR debe ser RGBA16F: w*h*4 elementos esperados"
        );

        // ── 0. Estado compartido ────────────────────────────────────────────
        let device = ctx.ash_device();
        let pool = create_one_shot_command_pool(ctx)?;

        // ── 1. Subir equirect HDR como sampler2D ────────────────────────────
        let equirect_img = upload_equirect_hdr(ctx, allocator.clone(), pool, pixels_rgba_f16, width, height)?;

        // ── 2. Crear los 3 outputs ──────────────────────────────────────────
        let mut radiance = create_cubemap(
            ctx, allocator.clone(),
            IBL_RADIANCE_SIZE,
            1, // 1 mip — fuente para los siguientes pasos
            vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_SRC,
        )?;
        let irradiance = create_cubemap(
            ctx, allocator.clone(),
            IBL_IRRADIANCE_SIZE,
            1,
            vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED,
        )?;
        let prefiltered = create_cubemap(
            ctx, allocator.clone(),
            IBL_PREFILTER_SIZE,
            IBL_PREFILTER_MIPS,
            vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED,
        )?;
        let brdf_lut = create_2d_lut(
            ctx, allocator.clone(),
            IBL_BRDF_LUT_SIZE,
            vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED,
        )?;

        // ── 3. Crear samplers compartidos ───────────────────────────────────
        let sampler_cube = create_cubemap_sampler(ctx, IBL_PREFILTER_MIPS as f32)?;
        let sampler_2d = create_2d_sampler(ctx)?;

        // ── 4. Recompilar pipelines de cada paso ────────────────────────────
        let p_equirect = ComputePass::new(
            ctx, SPV_EQUIRECT_TO_CUBE, /*ubo_binding*/ &[combined_image_sampler_b(0), storage_image_b(1)],
            std::mem::size_of::<EquirectPC>() as u32,
        )?;
        let p_irradiance = ComputePass::new(
            ctx, SPV_IRRADIANCE, &[combined_image_sampler_b(0), storage_image_b(1)],
            std::mem::size_of::<IrradiancePC>() as u32,
        )?;
        let p_prefilter = ComputePass::new(
            ctx, SPV_PREFILTER, &[combined_image_sampler_b(0), storage_image_b(1)],
            std::mem::size_of::<PrefilterPC>() as u32,
        )?;
        let p_brdf = ComputePass::new(
            ctx, SPV_BRDF_LUT, &[storage_image_b(0)],
            std::mem::size_of::<BrdfLutPC>() as u32,
        )?;

        // ── 5. Descriptor pool grande para todos los passes ─────────────────
        let bake_desc_pool = create_bake_descriptor_pool(ctx, IBL_PREFILTER_MIPS + 3)?;

        // ── 6. Grabar y submitir todo el baking ─────────────────────────────
        let cmd = begin_one_shot(ctx, pool)?;

        // 6a. radiance: UNDEFINED → GENERAL (para storage write)
        transition_cube(ctx, cmd, radiance.image, radiance.mip_levels, radiance.layer_count,
            vk::ImageLayout::UNDEFINED, vk::ImageLayout::GENERAL,
            vk::AccessFlags::empty(), vk::AccessFlags::SHADER_WRITE,
            vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::COMPUTE_SHADER);

        // 6b. equirect → radiance cube
        {
            let set = allocate_set(ctx, bake_desc_pool, p_equirect.layout_set)?;
            update_set_combined(ctx, set, 0, equirect_img.view, sampler_2d, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
            update_set_storage_image(ctx, set, 1, radiance.mip_views[0]);
            p_equirect.dispatch(ctx, cmd, &[set], &EquirectPC {
                face_size: IBL_RADIANCE_SIZE as i32,
                num_faces: 6,
                _pad: [0.0; 2],
            }, IBL_RADIANCE_SIZE / 8, IBL_RADIANCE_SIZE / 8, 6);
        }

        // 6c. radiance: GENERAL → SHADER_READ_ONLY (para sample en los siguientes)
        transition_cube(ctx, cmd, radiance.image, radiance.mip_levels, radiance.layer_count,
            vk::ImageLayout::GENERAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            vk::AccessFlags::SHADER_WRITE, vk::AccessFlags::SHADER_READ,
            vk::PipelineStageFlags::COMPUTE_SHADER, vk::PipelineStageFlags::COMPUTE_SHADER);

        // 6d. irradiance: UNDEFINED → GENERAL
        transition_cube(ctx, cmd, irradiance.image, irradiance.mip_levels, irradiance.layer_count,
            vk::ImageLayout::UNDEFINED, vk::ImageLayout::GENERAL,
            vk::AccessFlags::empty(), vk::AccessFlags::SHADER_WRITE,
            vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::COMPUTE_SHADER);

        // 6e. radiance → irradiance
        {
            let set = allocate_set(ctx, bake_desc_pool, p_irradiance.layout_set)?;
            update_set_combined(ctx, set, 0, radiance.view, sampler_cube, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
            update_set_storage_image(ctx, set, 1, irradiance.mip_views[0]);
            p_irradiance.dispatch(ctx, cmd, &[set], &IrradiancePC {
                face_size: IBL_IRRADIANCE_SIZE as i32,
                num_faces: 6,
                _pad: [0.0; 2],
            }, IBL_IRRADIANCE_SIZE.div_ceil(8), IBL_IRRADIANCE_SIZE.div_ceil(8), 6);
        }

        // 6f. prefiltered: UNDEFINED → GENERAL (todas las mips)
        transition_cube(ctx, cmd, prefiltered.image, prefiltered.mip_levels, prefiltered.layer_count,
            vk::ImageLayout::UNDEFINED, vk::ImageLayout::GENERAL,
            vk::AccessFlags::empty(), vk::AccessFlags::SHADER_WRITE,
            vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::COMPUTE_SHADER);

        // 6g. prefilter — un dispatch por mip
        for mip in 0..IBL_PREFILTER_MIPS {
            let mip_size = IBL_PREFILTER_SIZE >> mip;
            let roughness = mip as f32 / (IBL_PREFILTER_MIPS - 1) as f32;

            let set = allocate_set(ctx, bake_desc_pool, p_prefilter.layout_set)?;
            update_set_combined(ctx, set, 0, radiance.view, sampler_cube, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
            update_set_storage_image(ctx, set, 1, prefiltered.mip_views[mip as usize]);
            p_prefilter.dispatch(ctx, cmd, &[set], &PrefilterPC {
                mip_size: mip_size as i32,
                num_faces: 6,
                roughness,
                src_face_size: IBL_RADIANCE_SIZE as i32,
            }, mip_size.div_ceil(8).max(1), mip_size.div_ceil(8).max(1), 6);
        }

        // 6h. brdf_lut: UNDEFINED → GENERAL → dispatch → SHADER_READ_ONLY
        transition_2d(ctx, cmd, brdf_lut.image, 1,
            vk::ImageLayout::UNDEFINED, vk::ImageLayout::GENERAL,
            vk::AccessFlags::empty(), vk::AccessFlags::SHADER_WRITE,
            vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::COMPUTE_SHADER);
        {
            let set = allocate_set(ctx, bake_desc_pool, p_brdf.layout_set)?;
            update_set_storage_image(ctx, set, 0, brdf_lut.view);
            p_brdf.dispatch(ctx, cmd, &[set], &BrdfLutPC {
                size: IBL_BRDF_LUT_SIZE as i32,
                _pad: 0,
                _pad2: [0.0; 2],
            }, IBL_BRDF_LUT_SIZE / 8, IBL_BRDF_LUT_SIZE / 8, 1);
        }

        // 6i. Outputs finales: GENERAL → SHADER_READ_ONLY_OPTIMAL
        transition_cube(ctx, cmd, irradiance.image, irradiance.mip_levels, irradiance.layer_count,
            vk::ImageLayout::GENERAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            vk::AccessFlags::SHADER_WRITE, vk::AccessFlags::SHADER_READ,
            vk::PipelineStageFlags::COMPUTE_SHADER, vk::PipelineStageFlags::FRAGMENT_SHADER);
        transition_cube(ctx, cmd, prefiltered.image, prefiltered.mip_levels, prefiltered.layer_count,
            vk::ImageLayout::GENERAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            vk::AccessFlags::SHADER_WRITE, vk::AccessFlags::SHADER_READ,
            vk::PipelineStageFlags::COMPUTE_SHADER, vk::PipelineStageFlags::FRAGMENT_SHADER);
        transition_2d(ctx, cmd, brdf_lut.image, 1,
            vk::ImageLayout::GENERAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            vk::AccessFlags::SHADER_WRITE, vk::AccessFlags::SHADER_READ,
            vk::PipelineStageFlags::COMPUTE_SHADER, vk::PipelineStageFlags::FRAGMENT_SHADER);

        end_and_submit(ctx, pool, cmd)?;
        // Free internal pools/passes — destrucción RAII al salir del scope.
        unsafe {
            device.destroy_descriptor_pool(bake_desc_pool, None);
            device.destroy_command_pool(pool, None);
        }
        // Aseguramos que `radiance` se libere — sólo nos quedamos con irradiance/prefilter/brdf.
        drop(radiance);
        drop(equirect_img);

        // ── 7. Descriptor set final (set = 1, FRAGMENT) ─────────────────────
        let final_layout = create_final_descriptor_layout(ctx)?;
        let final_pool = create_final_descriptor_pool(ctx)?;
        let final_set = allocate_set(ctx, final_pool, final_layout)?;
        update_set_combined(ctx, final_set, 0, irradiance.view, sampler_cube, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        update_set_combined(ctx, final_set, 1, prefiltered.view, sampler_cube, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        update_set_combined(ctx, final_set, 2, brdf_lut.view, sampler_2d, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

        let max_mip_level = (IBL_PREFILTER_MIPS - 1) as f32;
        let (params_buf, params_alloc) = create_uniform_buffer(ctx, allocator.clone(), max_mip_level)?;
        update_set_uniform_buffer(ctx, final_set, 3, params_buf, std::mem::size_of::<f32>() as u64);

        Ok(IblTextures {
            irradiance,
            prefiltered,
            brdf_lut,
            sampler_cube,
            sampler_2d,
            descriptor_pool: final_pool,
            descriptor_set_layout: final_layout,
            descriptor_set: final_set,
            params_buffer: params_buf,
            params_allocation: Some(params_alloc),
            max_mip_level,
            device: device.clone(),
            allocator,
        })
    }

    /// Bake usando un sky procedural (sin necesitar HDR en disco). Útil para
    /// tests y como default cuando el proyecto aún no tiene un HDR cocinado.
    pub fn bake_procedural(
        ctx: &VulkanContext,
        allocator: Arc<Mutex<Allocator>>,
    ) -> ReactorResult<IblTextures> {
        let (pixels, w, h) = procedural_studio_sky(1024, 512);
        Self::bake_from_equirect_pixels(ctx, allocator, &pixels, w, h)
    }
}

// =============================================================================
// HDR loaders / encoders
// =============================================================================

/// Lee un Radiance `.hdr` y devuelve píxeles RGBA16F (`u16` × 4 por píxel)
/// + dimensiones.
fn load_hdr_equirect(path: &Path) -> ReactorResult<(Vec<u16>, u32, u32)> {
    let img = image::open(path).map_err(|e| {
        ReactorError::with_source(ErrorCode::Generic, "no se pudo abrir HDR equirect", e)
    })?;
    let rgb32 = img.to_rgb32f();
    let (w, h) = (rgb32.width(), rgb32.height());
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for p in rgb32.pixels() {
        out.push(half::f16::from_f32(p[0]).to_bits());
        out.push(half::f16::from_f32(p[1]).to_bits());
        out.push(half::f16::from_f32(p[2]).to_bits());
        out.push(half::f16::from_f32(1.0).to_bits());
    }
    Ok((out, w, h))
}

/// Sky procedural sencillo (gradiente cielo/horizonte/suelo + disco solar).
/// Devuelve píxeles RGBA16F en proyección equirectangular.
fn procedural_studio_sky(width: u32, height: u32) -> (Vec<u16>, u32, u32) {
    let mut out = Vec::with_capacity((width * height * 4) as usize);
    let sun_dir = glam::Vec3::new(-0.45, 0.85, 0.40).normalize();
    let sky_zen = glam::Vec3::new(0.42, 0.58, 0.85);
    let sky_hor = glam::Vec3::new(0.88, 0.92, 0.98);
    let gnd_nad = glam::Vec3::new(0.10, 0.10, 0.12);
    let sun_col = glam::Vec3::new(2.4, 2.2, 1.95);

    for y in 0..height {
        let v = (y as f32 + 0.5) / height as f32;     // 0..1
        let theta = v * std::f32::consts::PI;          // 0..π
        for x in 0..width {
            let u = (x as f32 + 0.5) / width as f32;  // 0..1
            let phi = u * std::f32::consts::TAU - std::f32::consts::PI; // -π..π
            let dir = glam::Vec3::new(
                theta.sin() * phi.cos(),
                theta.cos(),
                theta.sin() * phi.sin(),
            ).normalize();

            let up = dir.y;
            let t = up.signum() * up.abs().powf(0.6);
            let sky = sky_hor.lerp(sky_zen, t.clamp(0.0, 1.0));
            let gnd = sky_hor.lerp(gnd_nad, (-t).clamp(0.0, 1.0));
            let mut col = if up >= 0.0 { sky } else { gnd };
            let ds = dir.dot(sun_dir);
            let disc = if ds > 0.9995 { (ds - 0.9995) / 0.0005 } else { 0.0 };
            let halo = ds.max(0.0).powf(80.0) * 0.35;
            col += sun_col * (disc * 25.0 + halo);

            out.push(half::f16::from_f32(col.x).to_bits());
            out.push(half::f16::from_f32(col.y).to_bits());
            out.push(half::f16::from_f32(col.z).to_bits());
            out.push(half::f16::from_f32(1.0).to_bits());
        }
    }
    (out, width, height)
}

// =============================================================================
// Helpers: imagen 2D equirect → GPU
// =============================================================================

fn upload_equirect_hdr(
    ctx: &VulkanContext,
    allocator: Arc<Mutex<Allocator>>,
    pool: vk::CommandPool,
    pixels: &[u16],
    width: u32,
    height: u32,
) -> ReactorResult<IblImage> {
    let device = ctx.ash_device();
    let extent = vk::Extent3D { width, height, depth: 1 };

    // 1. Imagen GPU 2D RGBA16F.
    let img_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .extent(extent)
        .mip_levels(1)
        .array_layers(1)
        .format(RGBA16F)
        .tiling(vk::ImageTiling::OPTIMAL)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(vk::SampleCountFlags::TYPE_1);
    let image = unsafe { device.create_image(&img_info, None).map_err(verr)? };
    let req = unsafe { device.get_image_memory_requirements(image) };
    let alloc = allocator.lock().unwrap().allocate(&AllocationCreateDesc {
        name: "ibl_equirect_hdr",
        requirements: req,
        location: MemoryLocation::GpuOnly,
        linear: false,
        allocation_scheme: AllocationScheme::GpuAllocatorManaged,
    }).map_err(verr)?;
    unsafe { device.bind_image_memory(image, alloc.memory(), alloc.offset()).map_err(verr)? };

    let view = unsafe {
        device.create_image_view(&vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(RGBA16F)
            .subresource_range(vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0).level_count(1)
                .base_array_layer(0).layer_count(1)),
            None).map_err(verr)?
    };

    // 2. Staging buffer CPU-visible con los píxeles.
    let bytes = std::mem::size_of_val(pixels);
    let staging_info = vk::BufferCreateInfo::default()
        .size(bytes as u64)
        .usage(vk::BufferUsageFlags::TRANSFER_SRC)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    let staging = unsafe { device.create_buffer(&staging_info, None).map_err(verr)? };
    let sreq = unsafe { device.get_buffer_memory_requirements(staging) };
    let mut salloc = allocator.lock().unwrap().allocate(&AllocationCreateDesc {
        name: "ibl_equirect_staging",
        requirements: sreq,
        location: MemoryLocation::CpuToGpu,
        linear: true,
        allocation_scheme: AllocationScheme::GpuAllocatorManaged,
    }).map_err(verr)?;
    unsafe { device.bind_buffer_memory(staging, salloc.memory(), salloc.offset()).map_err(verr)? };
    // Copia los píxeles al staging.
    let dst_slice = salloc.mapped_slice_mut().expect("staging not mapped");
    let src_bytes: &[u8] = bytemuck::cast_slice(pixels);
    dst_slice[..src_bytes.len()].copy_from_slice(src_bytes);

    // 3. Copia GPU + transición a SHADER_READ_ONLY.
    let cmd = begin_one_shot(ctx, pool)?;
    transition_2d(ctx, cmd, image, 1,
        vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::AccessFlags::empty(), vk::AccessFlags::TRANSFER_WRITE,
        vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER);
    let region = vk::BufferImageCopy::default()
        .buffer_offset(0)
        .image_subresource(vk::ImageSubresourceLayers::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(0).base_array_layer(0).layer_count(1))
        .image_extent(extent);
    unsafe {
        device.cmd_copy_buffer_to_image(cmd, staging, image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[region]);
    }
    transition_2d(ctx, cmd, image, 1,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::SHADER_READ,
        vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::COMPUTE_SHADER);
    end_and_submit(ctx, pool, cmd)?;

    // Liberar staging.
    unsafe { device.destroy_buffer(staging, None); }
    let _ = allocator.lock().unwrap().free(salloc);

    Ok(IblImage {
        image,
        allocation: Some(alloc),
        view,
        mip_views: vec![],
        format: RGBA16F,
        extent,
        mip_levels: 1,
        layer_count: 1,
        device: device.clone(),
        allocator,
    })
}

// =============================================================================
// Helpers: cubemap + 2D LUT
// =============================================================================

fn create_cubemap(
    ctx: &VulkanContext,
    allocator: Arc<Mutex<Allocator>>,
    size: u32,
    mip_levels: u32,
    usage: vk::ImageUsageFlags,
) -> ReactorResult<IblImage> {
    let device = ctx.ash_device();
    let extent = vk::Extent3D { width: size, height: size, depth: 1 };
    let img_info = vk::ImageCreateInfo::default()
        .flags(vk::ImageCreateFlags::CUBE_COMPATIBLE)
        .image_type(vk::ImageType::TYPE_2D)
        .extent(extent)
        .mip_levels(mip_levels)
        .array_layers(6)
        .format(RGBA16F)
        .tiling(vk::ImageTiling::OPTIMAL)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(vk::SampleCountFlags::TYPE_1);
    let image = unsafe { device.create_image(&img_info, None).map_err(verr)? };
    let req = unsafe { device.get_image_memory_requirements(image) };
    let alloc = allocator.lock().unwrap().allocate(&AllocationCreateDesc {
        name: "ibl_cubemap",
        requirements: req,
        location: MemoryLocation::GpuOnly,
        linear: false,
        allocation_scheme: AllocationScheme::GpuAllocatorManaged,
    }).map_err(verr)?;
    unsafe { device.bind_image_memory(image, alloc.memory(), alloc.offset()).map_err(verr)? };

    // View principal: cubemap (sampling)
    let view = unsafe {
        device.create_image_view(&vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::CUBE)
            .format(RGBA16F)
            .subresource_range(vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0).level_count(mip_levels)
                .base_array_layer(0).layer_count(6)),
            None).map_err(verr)?
    };

    // Views por mip: 2D array (storage write desde compute)
    let mut mip_views = Vec::with_capacity(mip_levels as usize);
    for mip in 0..mip_levels {
        let v = unsafe {
            device.create_image_view(&vk::ImageViewCreateInfo::default()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D_ARRAY)
                .format(RGBA16F)
                .subresource_range(vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(mip).level_count(1)
                    .base_array_layer(0).layer_count(6)),
                None).map_err(verr)?
        };
        mip_views.push(v);
    }

    Ok(IblImage {
        image,
        allocation: Some(alloc),
        view,
        mip_views,
        format: RGBA16F,
        extent,
        mip_levels,
        layer_count: 6,
        device: device.clone(),
        allocator,
    })
}

fn create_2d_lut(
    ctx: &VulkanContext,
    allocator: Arc<Mutex<Allocator>>,
    size: u32,
    usage: vk::ImageUsageFlags,
) -> ReactorResult<IblImage> {
    let device = ctx.ash_device();
    let extent = vk::Extent3D { width: size, height: size, depth: 1 };
    let img_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .extent(extent)
        .mip_levels(1)
        .array_layers(1)
        .format(RG16F)
        .tiling(vk::ImageTiling::OPTIMAL)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(vk::SampleCountFlags::TYPE_1);
    let image = unsafe { device.create_image(&img_info, None).map_err(verr)? };
    let req = unsafe { device.get_image_memory_requirements(image) };
    let alloc = allocator.lock().unwrap().allocate(&AllocationCreateDesc {
        name: "ibl_brdf_lut",
        requirements: req,
        location: MemoryLocation::GpuOnly,
        linear: false,
        allocation_scheme: AllocationScheme::GpuAllocatorManaged,
    }).map_err(verr)?;
    unsafe { device.bind_image_memory(image, alloc.memory(), alloc.offset()).map_err(verr)? };

    let view = unsafe {
        device.create_image_view(&vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(RG16F)
            .subresource_range(vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0).level_count(1)
                .base_array_layer(0).layer_count(1)),
            None).map_err(verr)?
    };

    Ok(IblImage {
        image,
        allocation: Some(alloc),
        view,
        mip_views: vec![view], // alias: misma vista para storage y sampling
        format: RG16F,
        extent,
        mip_levels: 1,
        layer_count: 1,
        device: device.clone(),
        allocator,
    })
}

// =============================================================================
// Helpers: samplers / descriptor sets / barriers / submisión
// =============================================================================

fn create_cubemap_sampler(ctx: &VulkanContext, max_lod: f32) -> ReactorResult<vk::Sampler> {
    let info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::LINEAR).min_filter(vk::Filter::LINEAR)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .anisotropy_enable(false).max_anisotropy(1.0)
        .compare_enable(false).min_lod(0.0).max_lod(max_lod)
        .border_color(vk::BorderColor::FLOAT_OPAQUE_BLACK);
    unsafe { ctx.ash_device().create_sampler(&info, None).map_err(verr) }
}

fn create_2d_sampler(ctx: &VulkanContext) -> ReactorResult<vk::Sampler> {
    let info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::LINEAR).min_filter(vk::Filter::LINEAR)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .min_lod(0.0).max_lod(1.0);
    unsafe { ctx.ash_device().create_sampler(&info, None).map_err(verr) }
}

fn combined_image_sampler_b(b: u32) -> vk::DescriptorSetLayoutBinding<'static> {
    vk::DescriptorSetLayoutBinding::default()
        .binding(b)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::COMPUTE)
}

fn storage_image_b(b: u32) -> vk::DescriptorSetLayoutBinding<'static> {
    vk::DescriptorSetLayoutBinding::default()
        .binding(b)
        .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::COMPUTE)
}

fn create_bake_descriptor_pool(ctx: &VulkanContext, max_sets: u32) -> ReactorResult<vk::DescriptorPool> {
    let sizes = [
        vk::DescriptorPoolSize::default().ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(max_sets * 2),
        vk::DescriptorPoolSize::default().ty(vk::DescriptorType::STORAGE_IMAGE).descriptor_count(max_sets * 2),
    ];
    let info = vk::DescriptorPoolCreateInfo::default()
        .max_sets(max_sets)
        .pool_sizes(&sizes);
    unsafe { ctx.ash_device().create_descriptor_pool(&info, None).map_err(verr) }
}

fn create_final_descriptor_layout(ctx: &VulkanContext) -> ReactorResult<vk::DescriptorSetLayout> {
    let bindings = [
        vk::DescriptorSetLayoutBinding::default().binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
        vk::DescriptorSetLayoutBinding::default().binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
        vk::DescriptorSetLayoutBinding::default().binding(2)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
        vk::DescriptorSetLayoutBinding::default().binding(3)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
    ];
    let info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
    unsafe { ctx.ash_device().create_descriptor_set_layout(&info, None).map_err(verr) }
}

fn create_final_descriptor_pool(ctx: &VulkanContext) -> ReactorResult<vk::DescriptorPool> {
    let sizes = [
        vk::DescriptorPoolSize::default().ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(3),
        vk::DescriptorPoolSize::default().ty(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(1),
    ];
    let info = vk::DescriptorPoolCreateInfo::default()
        .max_sets(1)
        .pool_sizes(&sizes)
        .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET);
    unsafe { ctx.ash_device().create_descriptor_pool(&info, None).map_err(verr) }
}

fn allocate_set(ctx: &VulkanContext, pool: vk::DescriptorPool, layout: vk::DescriptorSetLayout)
    -> ReactorResult<vk::DescriptorSet>
{
    let layouts = [layout];
    let info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool).set_layouts(&layouts);
    let sets = unsafe { ctx.ash_device().allocate_descriptor_sets(&info).map_err(verr)? };
    Ok(sets[0])
}

fn update_set_combined(
    ctx: &VulkanContext, set: vk::DescriptorSet, binding: u32,
    view: vk::ImageView, sampler: vk::Sampler, layout: vk::ImageLayout,
) {
    let img = [vk::DescriptorImageInfo::default()
        .image_layout(layout).image_view(view).sampler(sampler)];
    let w = vk::WriteDescriptorSet::default()
        .dst_set(set).dst_binding(binding).dst_array_element(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .image_info(&img);
    unsafe { ctx.ash_device().update_descriptor_sets(&[w], &[]); }
}

fn update_set_storage_image(
    ctx: &VulkanContext, set: vk::DescriptorSet, binding: u32, view: vk::ImageView,
) {
    let img = [vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::GENERAL).image_view(view).sampler(vk::Sampler::null())];
    let w = vk::WriteDescriptorSet::default()
        .dst_set(set).dst_binding(binding).dst_array_element(0)
        .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
        .image_info(&img);
    unsafe { ctx.ash_device().update_descriptor_sets(&[w], &[]); }
}

fn update_set_uniform_buffer(
    ctx: &VulkanContext, set: vk::DescriptorSet, binding: u32,
    buffer: vk::Buffer, size: u64,
) {
    let bi = [vk::DescriptorBufferInfo::default().buffer(buffer).offset(0).range(size)];
    let w = vk::WriteDescriptorSet::default()
        .dst_set(set).dst_binding(binding).dst_array_element(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .buffer_info(&bi);
    unsafe { ctx.ash_device().update_descriptor_sets(&[w], &[]); }
}

fn create_uniform_buffer(
    ctx: &VulkanContext, allocator: Arc<Mutex<Allocator>>, max_mip: f32,
) -> ReactorResult<(vk::Buffer, Allocation)> {
    let device = ctx.ash_device();
    let size = std::mem::size_of::<f32>() as u64;
    let info = vk::BufferCreateInfo::default()
        .size(size)
        .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    let buf = unsafe { device.create_buffer(&info, None).map_err(verr)? };
    let req = unsafe { device.get_buffer_memory_requirements(buf) };
    let mut alloc = allocator.lock().unwrap().allocate(&AllocationCreateDesc {
        name: "ibl_params_ubo",
        requirements: req,
        location: MemoryLocation::CpuToGpu,
        linear: true,
        allocation_scheme: AllocationScheme::GpuAllocatorManaged,
    }).map_err(verr)?;
    unsafe { device.bind_buffer_memory(buf, alloc.memory(), alloc.offset()).map_err(verr)? };
    let slice = alloc.mapped_slice_mut().expect("ubo no mapeado");
    slice[..4].copy_from_slice(&max_mip.to_le_bytes());
    Ok((buf, alloc))
}

fn create_one_shot_command_pool(ctx: &VulkanContext) -> ReactorResult<vk::CommandPool> {
    let info = vk::CommandPoolCreateInfo::default()
        .queue_family_index(ctx.queue_family_index)
        .flags(vk::CommandPoolCreateFlags::TRANSIENT | vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
    unsafe { ctx.ash_device().create_command_pool(&info, None).map_err(verr) }
}

fn begin_one_shot(ctx: &VulkanContext, pool: vk::CommandPool) -> ReactorResult<vk::CommandBuffer> {
    let alloc = vk::CommandBufferAllocateInfo::default()
        .command_pool(pool).level(vk::CommandBufferLevel::PRIMARY).command_buffer_count(1);
    let cb = unsafe { ctx.ash_device().allocate_command_buffers(&alloc).map_err(verr)?[0] };
    let begin = vk::CommandBufferBeginInfo::default()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    unsafe { ctx.ash_device().begin_command_buffer(cb, &begin).map_err(verr)? };
    Ok(cb)
}

fn end_and_submit(
    ctx: &VulkanContext, _pool: vk::CommandPool, cb: vk::CommandBuffer,
) -> ReactorResult<()> {
    let device = ctx.ash_device();
    unsafe { device.end_command_buffer(cb).map_err(verr)?; }
    let cbs = [cb];
    let submit = vk::SubmitInfo::default().command_buffers(&cbs);
    unsafe {
        device.queue_submit(ctx.graphics_queue, &[submit], vk::Fence::null()).map_err(verr)?;
        device.queue_wait_idle(ctx.graphics_queue).map_err(verr)?;
    }
    Ok(())
}

fn transition_2d(
    ctx: &VulkanContext, cmd: vk::CommandBuffer, image: vk::Image, mip_levels: u32,
    old_l: vk::ImageLayout, new_l: vk::ImageLayout,
    src_a: vk::AccessFlags, dst_a: vk::AccessFlags,
    src_s: vk::PipelineStageFlags, dst_s: vk::PipelineStageFlags,
) {
    let b = vk::ImageMemoryBarrier::default()
        .old_layout(old_l).new_layout(new_l)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0).level_count(mip_levels)
            .base_array_layer(0).layer_count(1))
        .src_access_mask(src_a).dst_access_mask(dst_a);
    unsafe {
        ctx.ash_device().cmd_pipeline_barrier(cmd, src_s, dst_s,
            vk::DependencyFlags::empty(), &[], &[], &[b]);
    }
}

fn transition_cube(
    ctx: &VulkanContext, cmd: vk::CommandBuffer, image: vk::Image,
    mip_levels: u32, layer_count: u32,
    old_l: vk::ImageLayout, new_l: vk::ImageLayout,
    src_a: vk::AccessFlags, dst_a: vk::AccessFlags,
    src_s: vk::PipelineStageFlags, dst_s: vk::PipelineStageFlags,
) {
    let b = vk::ImageMemoryBarrier::default()
        .old_layout(old_l).new_layout(new_l)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0).level_count(mip_levels)
            .base_array_layer(0).layer_count(layer_count))
        .src_access_mask(src_a).dst_access_mask(dst_a);
    unsafe {
        ctx.ash_device().cmd_pipeline_barrier(cmd, src_s, dst_s,
            vk::DependencyFlags::empty(), &[], &[], &[b]);
    }
}

// =============================================================================
// ComputePass — pipeline + layout reutilizable
// =============================================================================

struct ComputePass {
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
    layout_set: vk::DescriptorSetLayout,
    device: ash::Device,
}

impl ComputePass {
    fn new(
        ctx: &VulkanContext,
        spv: &[u8],
        bindings: &[vk::DescriptorSetLayoutBinding],
        push_size: u32,
    ) -> ReactorResult<Self> {
        let device = ctx.ash_device();
        let code = read_spv(&mut Cursor::new(spv)).map_err(|e| ReactorError::with_source(
            ErrorCode::Generic, "spv inválido", e))?;
        let sm = unsafe { device.create_shader_module(
            &vk::ShaderModuleCreateInfo::default().code(&code), None).map_err(verr)? };

        let layout_set = unsafe { device.create_descriptor_set_layout(
            &vk::DescriptorSetLayoutCreateInfo::default().bindings(bindings), None).map_err(verr)? };

        let layouts = [layout_set];
        let push_ranges = [vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::COMPUTE, offset: 0, size: push_size,
        }];
        let layout = unsafe { device.create_pipeline_layout(
            &vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts).push_constant_ranges(&push_ranges),
            None).map_err(verr)? };

        let stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(sm)
            .name(unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(b"main\0") });
        let pipeline_info = vk::ComputePipelineCreateInfo::default().stage(stage).layout(layout);
        let pipelines = unsafe { device.create_compute_pipelines(vk::PipelineCache::null(),
            &[pipeline_info], None).map_err(|(_, e)| ReactorError::from(e))? };
        unsafe { device.destroy_shader_module(sm, None); }

        Ok(Self { pipeline: pipelines[0], layout, layout_set, device: device.clone() })
    }

    fn dispatch<T: Copy>(
        &self, ctx: &VulkanContext, cmd: vk::CommandBuffer,
        sets: &[vk::DescriptorSet], pc: &T,
        gx: u32, gy: u32, gz: u32,
    ) {
        let device = ctx.ash_device();
        unsafe {
            device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::COMPUTE, self.pipeline);
            device.cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::COMPUTE,
                self.layout, 0, sets, &[]);
            let bytes = std::slice::from_raw_parts(
                pc as *const T as *const u8, std::mem::size_of::<T>());
            device.cmd_push_constants(cmd, self.layout, vk::ShaderStageFlags::COMPUTE, 0, bytes);
            device.cmd_dispatch(cmd, gx, gy, gz);
            // Barrier global storage-write → storage-read para el siguiente paso.
            let mb = vk::MemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ);
            device.cmd_pipeline_barrier(cmd,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::COMPUTE_SHADER | vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(), &[mb], &[], &[]);
        }
    }
}

impl Drop for ComputePass {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.layout, None);
            self.device.destroy_descriptor_set_layout(self.layout_set, None);
        }
    }
}

// =============================================================================
// Push-constant structs (deben matchear con los .comp)
// =============================================================================

#[repr(C)]
#[derive(Clone, Copy)]
struct EquirectPC { face_size: i32, num_faces: i32, _pad: [f32; 2] }

#[repr(C)]
#[derive(Clone, Copy)]
struct IrradiancePC { face_size: i32, num_faces: i32, _pad: [f32; 2] }

#[repr(C)]
#[derive(Clone, Copy)]
struct PrefilterPC {
    mip_size: i32,
    num_faces: i32,
    roughness: f32,
    src_face_size: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct BrdfLutPC { size: i32, _pad: i32, _pad2: [f32; 2] }

// =============================================================================
// Convert helpers
// =============================================================================

fn verr<E: std::fmt::Display>(e: E) -> ReactorError {
    ReactorError::new(ErrorCode::Generic, format!("IBL Vulkan error: {}", e))
}
