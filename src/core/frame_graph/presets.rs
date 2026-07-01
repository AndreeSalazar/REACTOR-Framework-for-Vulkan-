use crate::core::frame_graph::types::*;
use crate::core::frame_graph::graph::FrameGraph;

pub fn create_deferred_graph(width: u32, height: u32) -> FrameGraph {
    let mut graph = FrameGraph::new();

    let depth = graph.create_resource("Depth", ResourceType::DepthBuffer, width, height, ResourceFormat::Depth32F);
    let albedo = graph.create_resource("GBuffer0_Albedo_AO", ResourceType::RenderTarget, width, height, ResourceFormat::RGBA8);
    let normal = graph.create_resource("GBuffer1_Normal_Material", ResourceType::RenderTarget, width, height, ResourceFormat::RGBA16F);
    let emissive = graph.create_resource("GBuffer2_Emissive_Material", ResourceType::RenderTarget, width, height, ResourceFormat::RGBA16F);
    let motion_depth = graph.create_resource("GBuffer3_Motion_Depth_Flags", ResourceType::RenderTarget, width, height, ResourceFormat::RGBA16F);
    let lit = graph.create_resource("Lit", ResourceType::RenderTarget, width, height, ResourceFormat::RGBA16F);
    let final_output = graph.create_resource("Final", ResourceType::Swapchain, width, height, ResourceFormat::RGBA8);

    graph.pass("GBuffer")
        .write(depth).write(albedo).write(normal).write(emissive).write(motion_depth)
        .order(0).build();

    graph.pass("Lighting")
        .read(depth).read(albedo).read(normal).read(emissive).read(motion_depth)
        .write(lit).order(1).build();

    graph.pass("PostProcess")
        .read(lit).write(final_output).order(2).build();

    graph.compile();
    graph
}

pub fn create_forward_graph(width: u32, height: u32) -> FrameGraph {
    let mut graph = FrameGraph::new();

    let depth = graph.create_resource("Depth", ResourceType::DepthBuffer, width, height, ResourceFormat::Depth32F);
    let color = graph.create_resource("Color", ResourceType::Swapchain, width, height, ResourceFormat::RGBA8);

    graph.pass("Forward").write(depth).write(color).order(0).build();

    graph.compile();
    graph
}
