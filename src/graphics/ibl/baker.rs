use crate::core::error::ReactorResult;
use crate::core::VulkanContext;
use crate::graphics::ibl::compute_pass::{BrdfLutPC, ComputePass, EquirectPC, IrradiancePC, PrefilterPC};
use crate::graphics::ibl::create::{create_2d_lut, create_cubemap};
use crate::graphics::ibl::helpers::{
    allocate_set, begin_one_shot, combined_image_sampler_b, create_bake_descriptor_pool,
    create_cubemap_sampler, create_final_descriptor_layout, create_final_descriptor_pool,
    create_one_shot_command_pool, create_2d_sampler, end_and_submit, storage_image_b,
    transition_cube, transition_2d, update_set_combined, update_set_storage_image,
    update_set_uniform_buffer,
};
use crate::graphics::ibl::sky::{load_hdr_equirect, procedural_studio_sky};
use crate::graphics::ibl::textures::IblTextures;
use crate::graphics::ibl::upload::upload_equirect_hdr;
use crate::graphics::ibl::{
    IBL_BRDF_LUT_SIZE, IBL_IRRADIANCE_SIZE, IBL_PREFILTER_MIPS, IBL_PREFILTER_SIZE,
    IBL_RADIANCE_SIZE, SPV_BRDF_LUT, SPV_EQUIRECT_TO_CUBE, SPV_IRRADIANCE, SPV_PREFILTER,
};
use ash::vk;
use gpu_allocator::vulkan::Allocator;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct IblBaker;

impl IblBaker {
    pub fn bake_from_equirect_file(
        ctx: &VulkanContext, allocator: Arc<Mutex<Allocator>>, hdr_path: impl AsRef<Path>,
    ) -> ReactorResult<IblTextures> {
        let (pixels, w, h) = load_hdr_equirect(hdr_path.as_ref())?;
        Self::bake_from_equirect_pixels(ctx, allocator, &pixels, w, h)
    }

    pub fn bake_from_equirect_pixels(
        ctx: &VulkanContext, allocator: Arc<Mutex<Allocator>>,
        pixels_rgba_f16: &[u16], width: u32, height: u32,
    ) -> ReactorResult<IblTextures> {
        assert_eq!(
            pixels_rgba_f16.len() as u32, width * height * 4,
            "equirect HDR debe ser RGBA16F: w*h*4 elementos esperados"
        );
        let device = ctx.ash_device();
        let pool = create_one_shot_command_pool(ctx)?;
        let equirect_img = upload_equirect_hdr(ctx, allocator.clone(), pool, pixels_rgba_f16, width, height)?;
        let radiance = create_cubemap(ctx, allocator.clone(), IBL_RADIANCE_SIZE, 1,
            vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_SRC)?;
        let irradiance = create_cubemap(ctx, allocator.clone(), IBL_IRRADIANCE_SIZE, 1,
            vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED)?;
        let prefiltered = create_cubemap(ctx, allocator.clone(), IBL_PREFILTER_SIZE, IBL_PREFILTER_MIPS,
            vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED)?;
        let brdf_lut = create_2d_lut(ctx, allocator.clone(), IBL_BRDF_LUT_SIZE,
            vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED)?;
        let sampler_cube = create_cubemap_sampler(ctx, IBL_PREFILTER_MIPS as f32)?;
        let sampler_2d = create_2d_sampler(ctx)?;
        let p_equirect = ComputePass::new(ctx, SPV_EQUIRECT_TO_CUBE,
            &[combined_image_sampler_b(0), storage_image_b(1)],
            std::mem::size_of::<EquirectPC>() as u32)?;
        let p_irradiance = ComputePass::new(ctx, SPV_IRRADIANCE,
            &[combined_image_sampler_b(0), storage_image_b(1)],
            std::mem::size_of::<IrradiancePC>() as u32)?;
        let p_prefilter = ComputePass::new(ctx, SPV_PREFILTER,
            &[combined_image_sampler_b(0), storage_image_b(1)],
            std::mem::size_of::<PrefilterPC>() as u32)?;
        let p_brdf = ComputePass::new(ctx, SPV_BRDF_LUT,
            &[storage_image_b(0)],
            std::mem::size_of::<BrdfLutPC>() as u32)?;
        let bake_desc_pool = create_bake_descriptor_pool(ctx, IBL_PREFILTER_MIPS + 3)?;
        let cmd = begin_one_shot(ctx, pool)?;
        transition_cube(ctx, cmd, radiance.image, radiance.mip_levels, radiance.layer_count,
            vk::ImageLayout::UNDEFINED, vk::ImageLayout::GENERAL,
            vk::AccessFlags::empty(), vk::AccessFlags::SHADER_WRITE,
            vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::COMPUTE_SHADER);
        {
            let set = allocate_set(ctx, bake_desc_pool, p_equirect.layout_set)?;
            update_set_combined(ctx, set, 0, equirect_img.view, sampler_2d, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
            update_set_storage_image(ctx, set, 1, radiance.mip_views[0]);
            p_equirect.dispatch(ctx, cmd, &[set], &EquirectPC {
                face_size: IBL_RADIANCE_SIZE as i32, num_faces: 6, _pad: [0.0; 2],
            }, IBL_RADIANCE_SIZE / 8, IBL_RADIANCE_SIZE / 8, 6);
        }
        transition_cube(ctx, cmd, radiance.image, radiance.mip_levels, radiance.layer_count,
            vk::ImageLayout::GENERAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            vk::AccessFlags::SHADER_WRITE, vk::AccessFlags::SHADER_READ,
            vk::PipelineStageFlags::COMPUTE_SHADER, vk::PipelineStageFlags::COMPUTE_SHADER);
        transition_cube(ctx, cmd, irradiance.image, irradiance.mip_levels, irradiance.layer_count,
            vk::ImageLayout::UNDEFINED, vk::ImageLayout::GENERAL,
            vk::AccessFlags::empty(), vk::AccessFlags::SHADER_WRITE,
            vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::COMPUTE_SHADER);
        {
            let set = allocate_set(ctx, bake_desc_pool, p_irradiance.layout_set)?;
            update_set_combined(ctx, set, 0, radiance.view, sampler_cube, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
            update_set_storage_image(ctx, set, 1, irradiance.mip_views[0]);
            p_irradiance.dispatch(ctx, cmd, &[set], &IrradiancePC {
                face_size: IBL_IRRADIANCE_SIZE as i32, num_faces: 6, _pad: [0.0; 2],
            }, IBL_IRRADIANCE_SIZE.div_ceil(8), IBL_IRRADIANCE_SIZE.div_ceil(8), 6);
        }
        transition_cube(ctx, cmd, prefiltered.image, prefiltered.mip_levels, prefiltered.layer_count,
            vk::ImageLayout::UNDEFINED, vk::ImageLayout::GENERAL,
            vk::AccessFlags::empty(), vk::AccessFlags::SHADER_WRITE,
            vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::COMPUTE_SHADER);
        for mip in 0..IBL_PREFILTER_MIPS {
            let mip_size = IBL_PREFILTER_SIZE >> mip;
            let roughness = mip as f32 / (IBL_PREFILTER_MIPS - 1) as f32;
            let set = allocate_set(ctx, bake_desc_pool, p_prefilter.layout_set)?;
            update_set_combined(ctx, set, 0, radiance.view, sampler_cube, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
            update_set_storage_image(ctx, set, 1, prefiltered.mip_views[mip as usize]);
            p_prefilter.dispatch(ctx, cmd, &[set], &PrefilterPC {
                mip_size: mip_size as i32, num_faces: 6, roughness, src_face_size: IBL_RADIANCE_SIZE as i32,
            }, mip_size.div_ceil(8).max(1), mip_size.div_ceil(8).max(1), 6);
        }
        transition_2d(ctx, cmd, brdf_lut.image, 1,
            vk::ImageLayout::UNDEFINED, vk::ImageLayout::GENERAL,
            vk::AccessFlags::empty(), vk::AccessFlags::SHADER_WRITE,
            vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::COMPUTE_SHADER);
        {
            let set = allocate_set(ctx, bake_desc_pool, p_brdf.layout_set)?;
            update_set_storage_image(ctx, set, 0, brdf_lut.view);
            p_brdf.dispatch(ctx, cmd, &[set], &BrdfLutPC {
                size: IBL_BRDF_LUT_SIZE as i32, _pad: 0, _pad2: [0.0; 2],
            }, IBL_BRDF_LUT_SIZE / 8, IBL_BRDF_LUT_SIZE / 8, 1);
        }
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
        unsafe {
            device.destroy_descriptor_pool(bake_desc_pool, None);
            device.destroy_command_pool(pool, None);
        }
        drop(radiance);
        drop(equirect_img);
        let final_layout = create_final_descriptor_layout(ctx)?;
        let final_pool = create_final_descriptor_pool(ctx)?;
        let final_set = allocate_set(ctx, final_pool, final_layout)?;
        update_set_combined(ctx, final_set, 0, irradiance.view, sampler_cube, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        update_set_combined(ctx, final_set, 1, prefiltered.view, sampler_cube, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        update_set_combined(ctx, final_set, 2, brdf_lut.view, sampler_2d, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        let max_mip_level = (IBL_PREFILTER_MIPS - 1) as f32;
        let (params_buf, params_alloc) = super::helpers::create_uniform_buffer(ctx, allocator.clone(), max_mip_level)?;
        update_set_uniform_buffer(ctx, final_set, 3, params_buf, std::mem::size_of::<f32>() as u64);
        Ok(IblTextures {
            irradiance, prefiltered, brdf_lut,
            sampler_cube, sampler_2d,
            descriptor_pool: final_pool, descriptor_set_layout: final_layout, descriptor_set: final_set,
            params_buffer: params_buf, params_allocation: Some(params_alloc), max_mip_level,
            device: device.clone(), allocator,
        })
    }

    pub fn bake_procedural(
        ctx: &VulkanContext, allocator: Arc<Mutex<Allocator>>,
    ) -> ReactorResult<IblTextures> {
        let (pixels, w, h) = procedural_studio_sky(1024, 512);
        Self::bake_from_equirect_pixels(ctx, allocator, &pixels, w, h)
    }
}
