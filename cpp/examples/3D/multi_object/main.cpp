// =============================================================================
// REACTOR — Multi-Object Scene Example (C++)
// =============================================================================
// Demonstrates large scene management:
//   - Spawning many objects with different materials
//   - ECS entity queries with component filters
//   - Object visibility toggling
//   - Scene serialization of complex scenes
//   - Performance with 200+ objects
//
// Rust handles GPU rendering at scale, C++ orchestrates the scene.
// =============================================================================

#include <reactor/application.hpp>
#include <cstdio>
#include <cmath>

using namespace reactor;

class MultiObjectDemo : public Application {
    MeshHandle* cube_mesh_ = nullptr;
    MaterialHandle* mats_[6] = {};

    static constexpr int GRID_SIZE = 15;
    static constexpr int TOTAL = GRID_SIZE * GRID_SIZE;
    int32_t scene_indices_[TOTAL];
    Entity entities_[TOTAL];
    float time_ = 0.0f;

public:
    Config config() override {
        return Config("REACTOR — Multi-Object Scene", 1280, 720)
            .with_msaa(4);
    }

    void on_init() override {
        printf("=== REACTOR Multi-Object Scene Demo ===\n\n");

        cube_mesh_ = reactor_create_cube();
        mats_[0] = reactor_create_material_simple(0.9f, 0.2f, 0.2f); // Red
        mats_[1] = reactor_create_material_simple(0.2f, 0.9f, 0.2f); // Green
        mats_[2] = reactor_create_material_simple(0.2f, 0.2f, 0.9f); // Blue
        mats_[3] = reactor_create_material_simple(0.9f, 0.9f, 0.2f); // Yellow
        mats_[4] = reactor_create_material_simple(0.9f, 0.2f, 0.9f); // Magenta
        mats_[5] = reactor_create_material_simple(0.2f, 0.9f, 0.9f); // Cyan

        reactor_set_camera_position(0, 20, 30);
        reactor_set_camera_target(0, 0, 0);
        reactor_add_directional_light(-0.5f, -1.0f, -0.3f, 1, 0.98f, 0.95f, 1.2f);
        reactor_add_point_light(0, 10, 0, 1, 1, 1, 2.0f, 30.0f);

        // Spawn grid
        printf("Spawning %d objects...\n", TOTAL);
        int count = 0;
        for (int z = 0; z < GRID_SIZE; ++z) {
            for (int x = 0; x < GRID_SIZE; ++x) {
                int idx = z * GRID_SIZE + x;
                int mat_idx = (x + z) % 6;

                float px = (x - GRID_SIZE / 2) * 2.0f;
                float pz = (z - GRID_SIZE / 2) * 2.0f;

                if (cube_mesh_ && mats_[mat_idx]) {
                    CMat4 t{};
                    t.cols[0][0] = 0.7f; t.cols[1][1] = 0.7f;
                    t.cols[2][2] = 0.7f; t.cols[3][3] = 1.0f;
                    t.cols[3][0] = px; t.cols[3][1] = 0.35f; t.cols[3][2] = pz;
                    scene_indices_[idx] = reactor_add_object(cube_mesh_, mats_[mat_idx], t);
                    count++;
                }

                // Create ECS entity
                char name[32];
                snprintf(name, sizeof(name), "Obj_%d_%d", x, z);
                entities_[idx] = Entity::create(name);
                entities_[idx].set_position(Vec3(px, 0.35f, pz));
                entities_[idx].add_mesh_renderer(0, mat_idx);

                // Add rigidbody to some
                if ((x + z) % 3 == 0) {
                    entities_[idx].add_rigidbody(1.0f, false);
                }
                // Add light to corners
                if ((x == 0 || x == GRID_SIZE-1) && (z == 0 || z == GRID_SIZE-1)) {
                    CLight light{};
                    light.light_type = 1;
                    light.position = {px, 3, pz};
                    light.color = {1, 0.8f, 0.5f};
                    light.intensity = 2.0f;
                    light.range = 8.0f;
                    entities_[idx].add_light(light);
                }
            }
        }

        printf("Spawned %d Vulkan objects\n", count);
        printf("ECS entities: %u\n", reactor_entity_count());

        // --- Query stats ---
        printf("\n--- Component Queries ---\n");
        auto all = ECS::query(COMPONENT_ALL, 512);
        printf("  All entities: %zu\n", all.size());
        auto renderers = ECS::query(COMPONENT_MESH_RENDERER, 512);
        printf("  With MeshRenderer: %zu\n", renderers.size());
        auto bodies = ECS::query(COMPONENT_RIGIDBODY, 512);
        printf("  With RigidBody: %zu\n", bodies.size());
        auto lights = ECS::query(COMPONENT_LIGHT, 512);
        printf("  With Light: %zu\n", lights.size());

        printf("\nControls:\n");
        printf("  1 - Toggle odd rows visibility\n");
        printf("  2 - Print render stats\n");
        printf("  ESC - Exit\n\n");
    }

    void on_update(float dt) override {
        time_ += dt;

        // Wave animation
        for (int z = 0; z < GRID_SIZE; ++z) {
            for (int x = 0; x < GRID_SIZE; ++x) {
                int idx = z * GRID_SIZE + x;
                float px = (x - GRID_SIZE / 2) * 2.0f;
                float pz = (z - GRID_SIZE / 2) * 2.0f;

                float wave = sinf(time_ * 2.0f + x * 0.4f + z * 0.3f) * 0.5f;
                float y = 0.35f + wave;
                float angle = time_ * 0.5f + (x + z) * 0.2f;
                float ca = cosf(angle), sa = sinf(angle);

                CMat4 t{};
                t.cols[0][0] = 0.7f * ca; t.cols[0][2] = 0.7f * sa;
                t.cols[1][1] = 0.7f;
                t.cols[2][0] = -0.7f * sa; t.cols[2][2] = 0.7f * ca;
                t.cols[3][3] = 1.0f;
                t.cols[3][0] = px; t.cols[3][1] = y; t.cols[3][2] = pz;

                if (scene_indices_[idx] >= 0) {
                    reactor_set_object_transform(scene_indices_[idx], t);
                }
            }
        }

        // Toggle visibility of odd rows
        if (Input::key_pressed(reactor_key_w())) {
            static bool odd_visible = true;
            odd_visible = !odd_visible;
            for (int z = 0; z < GRID_SIZE; ++z) {
                if (z % 2 == 1) {
                    for (int x = 0; x < GRID_SIZE; ++x) {
                        int idx = z * GRID_SIZE + x;
                        if (scene_indices_[idx] >= 0) {
                            reactor_set_object_visible(scene_indices_[idx], odd_visible);
                        }
                        entities_[idx].set_active(odd_visible);
                    }
                }
            }
            printf("\nOdd rows %s\n", odd_visible ? "VISIBLE" : "HIDDEN");
        }

        // Print stats
        if (Input::key_pressed(reactor_key_a())) {
            RenderStats::print();
        }

        if (reactor_get_frame_count() % 120 == 0) {
            auto s = RenderStats::get();
            printf("\rFPS:%.0f Draw:%u Tris:%u Vis:%u/%u    ",
                s.fps, s.draw_calls, s.triangles, s.visible_objects, s.scene_objects);
            fflush(stdout);
        }

        if (Input::key_pressed(Input::KEY_ESCAPE())) Window::request_close();
    }

    void on_render() override {}

    void on_shutdown() override {
        for (auto& e : entities_) e.destroy();
        printf("\nMulti-object demo shutdown. Cleaned %d entities.\n", TOTAL);
    }
};

int main() {
    MultiObjectDemo app;
    return app.run();
}
