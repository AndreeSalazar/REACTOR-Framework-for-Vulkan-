// =============================================================================
// REACTOR — FrameGraph Example (C++)
// =============================================================================
// Demonstrates the render graph system:
//   - Creating custom FrameGraphs
//   - Declaring resources (textures, depth, render targets)
//   - Adding passes with read/write dependencies
//   - Compiling and inspecting graph stats
//   - Using pre-built forward/deferred graphs
//
// The FrameGraph is declared in C++, compiled in Rust, barriers auto-generated.
// =============================================================================

#include <reactor/application.hpp>
#include <cstdio>

using namespace reactor;

class FrameGraphDemo : public Application {
    MeshHandle* cube_mesh_ = nullptr;
    MaterialHandle* cube_mat_ = nullptr;
    float time_ = 0.0f;

public:
    Config config() override {
        return Config("REACTOR — FrameGraph Demo", 1280, 720)
            .with_msaa(4);
    }

    void on_init() override {
        printf("=== REACTOR FrameGraph Demo ===\n\n");

        // Setup basic scene
        cube_mesh_ = reactor_create_cube();
        cube_mat_ = reactor_create_material_simple(0.6f, 0.4f, 0.8f);
        reactor_set_camera_position(0, 3, 6);
        reactor_set_camera_target(0, 0, 0);
        reactor_add_directional_light(-0.5f, -1.0f, -0.3f, 1, 1, 1, 1.0f);

        if (cube_mesh_ && cube_mat_) {
            CMat4 t{}; t.cols[0][0]=1; t.cols[1][1]=1; t.cols[2][2]=1; t.cols[3][3]=1;
            reactor_add_object(cube_mesh_, cube_mat_, t);
        }

        // =====================================================================
        // 1. Custom FrameGraph — manual pass declaration
        // =====================================================================
        printf("--- Custom FrameGraph ---\n");
        {
            FrameGraph graph;

            // Declare resources
            uint32_t color_rt = graph.create_resource("ColorRT",
                FGResourceType::RenderTarget, 1280, 720, FGFormat::RGBA16F);
            uint32_t depth = graph.create_resource("DepthBuffer",
                FGResourceType::DepthBuffer, 1280, 720, FGFormat::Depth32F);
            uint32_t gbuffer_albedo = graph.create_resource("GBuffer_Albedo",
                FGResourceType::RenderTarget, 1280, 720, FGFormat::RGBA8);
            uint32_t gbuffer_normal = graph.create_resource("GBuffer_Normal",
                FGResourceType::RenderTarget, 1280, 720, FGFormat::RGBA16F);
            uint32_t shadow_map = graph.create_resource("ShadowMap",
                FGResourceType::DepthBuffer, 2048, 2048, FGFormat::Depth32F, true);
            uint32_t swapchain = graph.create_resource("Swapchain",
                FGResourceType::Swapchain, 1280, 720, FGFormat::RGBA8);

            printf("  Resources created: ColorRT=%u, Depth=%u, Albedo=%u, Normal=%u, Shadow=%u, Swap=%u\n",
                color_rt, depth, gbuffer_albedo, gbuffer_normal, shadow_map, swapchain);

            // Declare passes
            uint32_t shadow_pass = graph.add_pass("ShadowPass", {}, {shadow_map}, 0);
            uint32_t gbuffer_pass = graph.add_pass("GBufferPass", {shadow_map},
                {gbuffer_albedo, gbuffer_normal, depth}, 1);
            uint32_t lighting_pass = graph.add_pass("LightingPass",
                {gbuffer_albedo, gbuffer_normal, depth, shadow_map}, {color_rt}, 2);
            uint32_t tonemap_pass = graph.add_pass("TonemapPass", {color_rt}, {swapchain}, 3);

            printf("  Passes: Shadow=%u, GBuffer=%u, Lighting=%u, Tonemap=%u\n",
                shadow_pass, gbuffer_pass, lighting_pass, tonemap_pass);

            // Compile — generates barriers and execution order
            bool ok = graph.compile();
            printf("  Compiled: %s\n", ok ? "YES" : "NO");

            // Inspect stats
            auto stats = graph.stats();
            printf("  Stats:\n");
            printf("    Total passes:       %u\n", stats.total_passes);
            printf("    Enabled passes:     %u\n", stats.enabled_passes);
            printf("    Total resources:    %u\n", stats.total_resources);
            printf("    Transient resources:%u\n", stats.transient_resources);
            printf("    Barriers generated: %u\n", stats.barriers_generated);
        }

        // =====================================================================
        // 2. Pre-built Forward Graph
        // =====================================================================
        printf("\n--- Pre-built Forward Graph ---\n");
        {
            auto fwd = FrameGraph::forward(1920, 1080);
            auto stats = fwd.stats();
            printf("  Passes: %u | Resources: %u | Barriers: %u\n",
                stats.total_passes, stats.total_resources, stats.barriers_generated);
        }

        // =====================================================================
        // 3. Pre-built Deferred Graph
        // =====================================================================
        printf("\n--- Pre-built Deferred Graph ---\n");
        {
            auto def = FrameGraph::deferred(1920, 1080);
            auto stats = def.stats();
            printf("  Passes: %u | Resources: %u | Barriers: %u\n",
                stats.total_passes, stats.total_resources, stats.barriers_generated);
        }

        printf("\nControls: ESC = Exit\n\n");
    }

    void on_update(float dt) override {
        time_ += dt;

        // Rotate cube
        float a = time_ * 0.5f;
        float ca = cosf(a), sa = sinf(a);
        CMat4 t{};
        t.cols[0][0] = ca; t.cols[0][2] = sa;
        t.cols[1][1] = 1;
        t.cols[2][0] = -sa; t.cols[2][2] = ca;
        t.cols[3][3] = 1;
        reactor_set_object_transform(0, t);

        if (Input::key_pressed(Input::KEY_ESCAPE())) Window::request_close();
    }

    void on_render() override {}
};

int main() {
    FrameGraphDemo app;
    return app.run();
}
