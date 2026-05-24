//! Render-pass + framebuffer legacy helpers.
//!
//! Hoy el draw real usa **Dynamic Rendering** (Vulkan 1.3), por lo que estas
//! variantes están marcadas `#[allow(dead_code)]` y se conservan para:
//!  - depurar/comparar contra paths sin dynamic rendering,
//!  - permitir backends alternativos (mobile, MoltenVK antiguos).

#![allow(dead_code)]

use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use crate::core::VulkanContext;
use crate::graphics::swapchain::Swapchain;
use ash::vk;

// =============================================================================
// Render passes
// =============================================================================

/// Render-pass simple sin MSAA, color sólo.
pub(super) fn create_render_pass_simple(
    context: &VulkanContext,
    format: vk::Format,
) -> ReactorResult<vk::RenderPass> {
    let color_attachment = vk::AttachmentDescription::default()
        .format(format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

    let color_attachment_ref = vk::AttachmentReference::default()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let subpass = vk::SubpassDescription::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(std::slice::from_ref(&color_attachment_ref));

    let dependency = vk::SubpassDependency::default()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

    let attachments = [color_attachment];
    let subpasses = [subpass];
    let dependencies = [dependency];

    let render_pass_info = vk::RenderPassCreateInfo::default()
        .attachments(&attachments)
        .subpasses(&subpasses)
        .dependencies(&dependencies);

    unsafe {
        context
            .device
            .create_render_pass(&render_pass_info, None)
            .map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanRenderPass,
                    "Failed to create render pass",
                    e,
                )
            })
    }
}

/// Render-pass con MSAA + resolve, sin depth.
pub(super) fn create_render_pass_msaa(
    context: &VulkanContext,
    format: vk::Format,
    samples: vk::SampleCountFlags,
) -> ReactorResult<vk::RenderPass> {
    let msaa_attachment = vk::AttachmentDescription::default()
        .format(format)
        .samples(samples)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let resolve_attachment = vk::AttachmentDescription::default()
        .format(format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::DONT_CARE)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

    let color_attachment_ref = vk::AttachmentReference::default()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
    let resolve_attachment_ref = vk::AttachmentReference::default()
        .attachment(1)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let color_attachments = [color_attachment_ref];
    let resolve_attachments = [resolve_attachment_ref];

    let subpass = vk::SubpassDescription::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&color_attachments)
        .resolve_attachments(&resolve_attachments);

    let dependency = vk::SubpassDependency::default()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

    let attachments = [msaa_attachment, resolve_attachment];
    let subpasses = [subpass];
    let dependencies = [dependency];

    let render_pass_info = vk::RenderPassCreateInfo::default()
        .attachments(&attachments)
        .subpasses(&subpasses)
        .dependencies(&dependencies);

    unsafe {
        context
            .device
            .create_render_pass(&render_pass_info, None)
            .map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanRenderPass,
                    "Failed to create MSAA render pass",
                    e,
                )
            })
    }
}

/// Wrapper que elige automáticamente entre `simple` y `msaa`.
pub(super) fn create_render_pass_with_msaa(
    context: &VulkanContext,
    format: vk::Format,
    samples: vk::SampleCountFlags,
) -> ReactorResult<vk::RenderPass> {
    if samples == vk::SampleCountFlags::TYPE_1 {
        create_render_pass_simple(context, format)
    } else {
        create_render_pass_msaa(context, format, samples)
    }
}

/// Render-pass completo con MSAA opcional + depth.
pub(super) fn create_render_pass_with_depth(
    context: &VulkanContext,
    color_format: vk::Format,
    depth_format: vk::Format,
    samples: vk::SampleCountFlags,
) -> ReactorResult<vk::RenderPass> {
    if samples == vk::SampleCountFlags::TYPE_1 {
        // No MSAA: color + depth.
        let color_attachment = vk::AttachmentDescription::default()
            .format(color_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let depth_attachment = vk::AttachmentDescription::default()
            .format(depth_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let color_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        let depth_ref = vk::AttachmentReference::default()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let color_attachments = [color_ref];
        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachments)
            .depth_stencil_attachment(&depth_ref);

        let dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            );

        let attachments = [color_attachment, depth_attachment];
        let subpasses = [subpass];
        let dependencies = [dependency];

        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        return unsafe {
            context
                .device
                .create_render_pass(&render_pass_info, None)
                .map_err(|e| {
                    ReactorError::with_source(
                        ErrorCode::VulkanRenderPass,
                        "Failed to create render pass",
                        e,
                    )
                })
        };
    }

    // MSAA + depth.
    let msaa_color = vk::AttachmentDescription::default()
        .format(color_format)
        .samples(samples)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let resolve_color = vk::AttachmentDescription::default()
        .format(color_format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::DONT_CARE)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

    let depth_attachment = vk::AttachmentDescription::default()
        .format(depth_format)
        .samples(samples)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

    let color_ref = vk::AttachmentReference::default()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
    let resolve_ref = vk::AttachmentReference::default()
        .attachment(1)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
    let depth_ref = vk::AttachmentReference::default()
        .attachment(2)
        .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

    let color_attachments = [color_ref];
    let resolve_attachments = [resolve_ref];
    let subpass = vk::SubpassDescription::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&color_attachments)
        .resolve_attachments(&resolve_attachments)
        .depth_stencil_attachment(&depth_ref);

    let dependency = vk::SubpassDependency::default()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        )
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        )
        .dst_access_mask(
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
        );

    let attachments = [msaa_color, resolve_color, depth_attachment];
    let subpasses = [subpass];
    let dependencies = [dependency];

    let render_pass_info = vk::RenderPassCreateInfo::default()
        .attachments(&attachments)
        .subpasses(&subpasses)
        .dependencies(&dependencies);

    unsafe {
        context
            .device
            .create_render_pass(&render_pass_info, None)
            .map_err(|e| {
                ReactorError::with_source(
                    ErrorCode::VulkanRenderPass,
                    "Failed to create MSAA render pass",
                    e,
                )
            })
    }
}

// =============================================================================
// Framebuffers
// =============================================================================

/// Framebuffers simples: una vista de color por imagen del swapchain.
pub(super) fn create_framebuffers(
    context: &VulkanContext,
    swapchain: &Swapchain,
    render_pass: vk::RenderPass,
) -> ReactorResult<Vec<vk::Framebuffer>> {
    swapchain
        .image_views
        .iter()
        .map(|&view| {
            let attachments = [view];
            let framebuffer_info = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(swapchain.extent.width)
                .height(swapchain.extent.height)
                .layers(1);
            unsafe {
                context
                    .device
                    .create_framebuffer(&framebuffer_info, None)
                    .map_err(|e| {
                        ReactorError::with_source(
                            ErrorCode::VulkanFramebuffer,
                            "Failed to create framebuffer",
                            e,
                        )
                    })
            }
        })
        .collect()
}

/// Framebuffers con MSAA (color multi-sample + resolve por imagen del swapchain).
pub(super) fn create_framebuffers_msaa(
    context: &VulkanContext,
    swapchain: &Swapchain,
    render_pass: vk::RenderPass,
    msaa_view: Option<vk::ImageView>,
    samples: vk::SampleCountFlags,
) -> ReactorResult<Vec<vk::Framebuffer>> {
    if samples == vk::SampleCountFlags::TYPE_1 {
        return create_framebuffers(context, swapchain, render_pass);
    }

    let msaa_view = match msaa_view {
        Some(view) => view,
        None => return create_framebuffers(context, swapchain, render_pass),
    };

    swapchain
        .image_views
        .iter()
        .map(|&resolve_view| {
            let attachments = [msaa_view, resolve_view];
            let framebuffer_info = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(swapchain.extent.width)
                .height(swapchain.extent.height)
                .layers(1);
            unsafe {
                context
                    .device
                    .create_framebuffer(&framebuffer_info, None)
                    .map_err(|e| {
                        ReactorError::with_source(
                            ErrorCode::VulkanFramebuffer,
                            "Failed to create MSAA framebuffer",
                            e,
                        )
                    })
            }
        })
        .collect()
}

/// Framebuffers con depth (y MSAA opcional).
pub(super) fn create_framebuffers_with_depth(
    context: &VulkanContext,
    swapchain: &Swapchain,
    render_pass: vk::RenderPass,
    msaa_view: Option<vk::ImageView>,
    depth_view: vk::ImageView,
    samples: vk::SampleCountFlags,
) -> ReactorResult<Vec<vk::Framebuffer>> {
    if samples == vk::SampleCountFlags::TYPE_1 {
        return swapchain
            .image_views
            .iter()
            .map(|&color_view| {
                let attachments = [color_view, depth_view];
                let framebuffer_info = vk::FramebufferCreateInfo::default()
                    .render_pass(render_pass)
                    .attachments(&attachments)
                    .width(swapchain.extent.width)
                    .height(swapchain.extent.height)
                    .layers(1);
                unsafe {
                    context
                        .device
                        .create_framebuffer(&framebuffer_info, None)
                        .map_err(|e| {
                            ReactorError::with_source(
                                ErrorCode::VulkanFramebuffer,
                                "Failed to create framebuffer",
                                e,
                            )
                        })
                }
            })
            .collect();
    }

    let msaa_view = msaa_view.ok_or_else(|| {
        ReactorError::new(
            ErrorCode::VulkanFramebuffer,
            "MSAA view required for MSAA framebuffers",
        )
    })?;

    swapchain
        .image_views
        .iter()
        .map(|&resolve_view| {
            let attachments = [msaa_view, resolve_view, depth_view];
            let framebuffer_info = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(swapchain.extent.width)
                .height(swapchain.extent.height)
                .layers(1);
            unsafe {
                context
                    .device
                    .create_framebuffer(&framebuffer_info, None)
                    .map_err(|e| {
                        ReactorError::with_source(
                            ErrorCode::VulkanFramebuffer,
                            "Failed to create MSAA framebuffer",
                            e,
                        )
                    })
            }
        })
        .collect()
}
