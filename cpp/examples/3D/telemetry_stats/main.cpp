// =============================================================================
// REACTOR — Telemetry & Stats Example (C++)
// =============================================================================
// Demonstrates GPU/CPU telemetry and diagnostics:
//   - Real-time render stats (FPS, draw calls, triangles, VRAM)
//   - Memory budget queries (device local, host visible)
//   - GPU info (name, MSAA, RT support, Vulkan version)
//   - Scene serialization to JSON
//   - Stress test with many objects
//
// Rust queries Vulkan directly, C++ reads stats via shared C ABI.
// =============================================================================

#include <reactor/application.hpp>
#include <cstdio>
#include <cmath>

using namespace reactor;

class TelemetryDemo : public Application {
    MeshHandle* cube_mesh_ = nullptr;
    MaterialHandle* mats_[3] = {};
    int32_t objects_[100];
    int object_count_ = 0;
    float time_ = 0.0f;
    float stats_timer_ = 0.0f;

public:
    Config config() override {
        return Config("REACTOR — Telemetry & Stats", 1280, 720)
            .with_msaa(4);
    }

    void on_init() override {
        printf("=== REACTOR Telemetry & Stats Demo ===\n\n");

        // --- GPU Info ---
        printf("--- GPU Information ---\n");
        printf("  GPU Name: %s\n", reactor_get_gpu_name());
        printf("  VRAM: %u MB\n", reactor_get_vram_mb());
        printf("  MSAA: %ux\n", reactor_get_msaa_samples());
        printf("  Ray Tracing: %s\n", reactor_is_raytracing_supported() ? "YES" : "NO");
        uint32_t vk_major, vk_minor, vk_patch;
        reactor_get_vulkan_version(&vk_major, &vk_minor, &vk_patch);
        printf("  Vulkan: %u.%u.%u\n", vk_major, vk_minor, vk_patch);

        // --- Memory Budget ---
        printf("\n--- Memory Budget ---\n");
        auto budget = RenderStats::memory_budget();
        printf("  Device Local Budget: %llu MB\n", (unsigned long long)(budget.device_local_budget / (1024*1024)));
        printf("  Host Visible Budget: %llu MB\n", (unsigned long long)(budget.host_visible_budget / (1024*1024)));

        // Setup scene
        cube_mesh_ = reactor_create_cube();
        mats_[0] = reactor_create_material_simple(0.8f, 0.3f, 0.2f);
        mats_[1] = reactor_create_material_simple(0.2f, 0.6f, 0.8f);
        mats_[2] = reactor_create_material_simple(0.3f, 0.8f, 0.3f);

        reactor_set_camera_position(0, 15, 25);
        reactor_set_camera_target(0, 0, 0);
        reactor_add_directional_light(-0.5f, -1.0f, -0.3f, 1, 1, 1, 1.0f);

        // Spawn grid of objects for stress test
        printf("\n--- Spawning 100 objects ---\n");
        if (cube_mesh_) {
            for (int z = 0; z < 10; ++z) {
                for (int x = 0; x < 10; ++x) {
                    int mat_idx = (x + z) % 3;
                    if (mats_[mat_idx]) {
                        CMat4 t{};
                        t.cols[0][0] = 0.8f; t.cols[1][1] = 0.8f;
                        t.cols[2][2] = 0.8f; t.cols[3][3] = 1.0f;
                        t.cols[3][0] = -9.0f + x * 2.0f;
                        t.cols[3][1] = 0.4f;
                        t.cols[3][2] = -9.0f + z * 2.0f;
                        objects_[object_count_] = reactor_add_object(cube_mesh_, mats_[mat_idx], t);
                        object_count_++;
                    }
                }
            }
        }
        printf("  Spawned %d objects\n", object_count_);

        // --- Initial Render Stats ---
        printf("\n--- Initial Render Stats ---\n");
        RenderStats::print();

        // --- Scene Serialization ---
        printf("\n--- Scene Serialization ---\n");
        std::string scene_json = SceneSerializer::serialize();
        printf("  Serialized size: %zu bytes\n", scene_json.size());
        if (scene_json.size() > 200) {
            printf("  Preview: %.200s...\n", scene_json.c_str());
        } else {
            printf("  Content: %s\n", scene_json.c_str());
        }

        printf("\nControls:\n");
        printf("  1 - Print render stats\n");
        printf("  2 - Print memory budget\n");
        printf("  3 - Serialize scene\n");
        printf("  ESC - Exit\n\n");
    }

    void on_update(float dt) override {
        time_ += dt;
        stats_timer_ += dt;

        // Animate objects (wave)
        for (int i = 0; i < object_count_; ++i) {
            int x = i % 10;
            int z = i / 10;
            float y = 0.4f + sinf(time_ * 2.0f + x * 0.5f + z * 0.3f) * 0.5f;
            float angle = time_ * 0.5f + i * 0.1f;
            float ca = cosf(angle), sa = sinf(angle);

            CMat4 t{};
            t.cols[0][0] = 0.8f * ca; t.cols[0][2] = 0.8f * sa;
            t.cols[1][1] = 0.8f;
            t.cols[2][0] = -0.8f * sa; t.cols[2][2] = 0.8f * ca;
            t.cols[3][3] = 1.0f;
            t.cols[3][0] = -9.0f + x * 2.0f;
            t.cols[3][1] = y;
            t.cols[3][2] = -9.0f + z * 2.0f;
            reactor_set_object_transform(objects_[i], t);
        }

        // Auto-print stats every 2 seconds
        if (stats_timer_ >= 2.0f) {
            stats_timer_ = 0.0f;
            auto s = RenderStats::get();
            printf("\r[%.1fs] FPS:%.0f Frame:%.1fms Draw:%u Tris:%u Vis:%u/%u VRAM:%uMB    ",
                time_, s.fps, s.frame_time_ms, s.draw_calls, s.triangles,
                s.visible_objects, s.scene_objects, s.vram_total_mb);
            fflush(stdout);
        }

        if (Input::key_pressed(Input::KEY_ESCAPE())) Window::request_close();
    }

    void on_render() override {}
};

int main() {
    TelemetryDemo app;
    return app.run();
}
