// =============================================================================
// REACTOR — Lighting Showcase Example (C++)
// =============================================================================
// Demonstrates the full lighting system:
//   - Directional lights (sun)
//   - Point lights (colored, animated)
//   - Spot lights (flashlight effect)
//   - Dynamic light parameters at runtime
//   - ECS light components on entities
//
// Rust Vulkan backend renders lights, C++ controls parameters via C ABI.
// =============================================================================

#include <reactor/application.hpp>
#include <cstdio>
#include <cmath>

using namespace reactor;

class LightingDemo : public Application {
    MeshHandle* cube_mesh_ = nullptr;
    MaterialHandle* floor_mat_ = nullptr;
    MaterialHandle* pillar_mat_ = nullptr;
    MaterialHandle* sphere_mat_ = nullptr;

    Entity point_lights_[4];
    Entity spot_entity_;
    float time_ = 0.0f;

public:
    Config config() override {
        return Config("REACTOR — Lighting Showcase", 1280, 720)
            .with_msaa(4);
    }

    void on_init() override {
        printf("=== REACTOR Lighting Showcase ===\n\n");

        cube_mesh_ = reactor_create_cube();
        floor_mat_ = reactor_create_material_simple(0.4f, 0.4f, 0.45f);
        pillar_mat_ = reactor_create_material_simple(0.7f, 0.7f, 0.75f);
        sphere_mat_ = reactor_create_material_simple(0.9f, 0.9f, 0.9f);

        reactor_set_camera_position(0, 8, 15);
        reactor_set_camera_target(0, 0, 0);

        if (!cube_mesh_) return;

        // --- Floor ---
        if (floor_mat_) {
            CMat4 t{};
            t.cols[0][0] = 20; t.cols[1][1] = 0.2f; t.cols[2][2] = 20; t.cols[3][3] = 1;
            t.cols[3][1] = -0.1f;
            reactor_add_object(cube_mesh_, floor_mat_, t);
        }

        // --- Pillars ---
        if (pillar_mat_) {
            float positions[][2] = {{-5, -5}, {5, -5}, {-5, 5}, {5, 5}};
            for (auto& p : positions) {
                CMat4 t{};
                t.cols[0][0] = 0.5f; t.cols[1][1] = 4; t.cols[2][2] = 0.5f; t.cols[3][3] = 1;
                t.cols[3][0] = p[0]; t.cols[3][1] = 2; t.cols[3][2] = p[1];
                reactor_add_object(cube_mesh_, pillar_mat_, t);
            }
        }

        // --- Center sphere (cube proxy) ---
        if (sphere_mat_) {
            CMat4 t{};
            t.cols[0][0] = 2; t.cols[1][1] = 2; t.cols[2][2] = 2; t.cols[3][3] = 1;
            t.cols[3][1] = 1;
            reactor_add_object(cube_mesh_, sphere_mat_, t);
        }

        // --- Directional light (dim ambient sun) ---
        reactor_add_directional_light(0.2f, -1.0f, 0.3f, 0.3f, 0.35f, 0.4f, 0.4f);
        printf("Added directional light (dim sun)\n");

        // --- 4 colored point lights (orbiting) ---
        Vec3 colors[] = {
            {1.0f, 0.2f, 0.1f},  // Red
            {0.1f, 1.0f, 0.2f},  // Green
            {0.2f, 0.3f, 1.0f},  // Blue
            {1.0f, 0.9f, 0.2f},  // Yellow
        };

        for (int i = 0; i < 4; ++i) {
            float angle = (float)i * 1.5708f; // 90 degrees apart
            float x = cosf(angle) * 6.0f;
            float z = sinf(angle) * 6.0f;

            reactor_add_point_light(x, 3, z,
                colors[i].x, colors[i].y, colors[i].z, 3.0f, 12.0f);

            // ECS entity for tracking
            char name[32];
            snprintf(name, sizeof(name), "PointLight_%d", i);
            point_lights_[i] = Entity::create(name);
            point_lights_[i].set_position(Vec3(x, 3, z));

            CLight light{};
            light.light_type = 1; // Point
            light.position = {x, 3, z};
            light.color = {colors[i].x, colors[i].y, colors[i].z};
            light.intensity = 3.0f;
            light.range = 12.0f;
            point_lights_[i].add_light(light);

            printf("Added point light %d: (%.1f, 3, %.1f) color=(%.1f, %.1f, %.1f)\n",
                i, x, z, colors[i].x, colors[i].y, colors[i].z);
        }

        // --- Spot light (flashlight) ---
        reactor_add_spot_light(0, 6, 0, 0, -1, 0, 1, 1, 1, 5.0f, 15.0f, 30.0f);
        spot_entity_ = Entity::create("SpotLight");
        spot_entity_.set_position(Vec3(0, 6, 0));
        CLight spot{};
        spot.light_type = 2; // Spot
        spot.position = {0, 6, 0};
        spot.direction = {0, -1, 0};
        spot.color = {1, 1, 1};
        spot.intensity = 5.0f;
        spot.range = 15.0f;
        spot.inner_angle = 20.0f;
        spot.outer_angle = 30.0f;
        spot_entity_.add_light(spot);
        printf("Added spot light at (0, 6, 0)\n");

        printf("\nTotal lights: %u\n", reactor_light_count());
        printf("Light entities: %zu\n", ECS::query(COMPONENT_LIGHT).size());
        printf("\nControls: ESC = Exit\n\n");
    }

    void on_update(float dt) override {
        time_ += dt;

        // Orbit point lights around center
        for (int i = 0; i < 4; ++i) {
            float base_angle = (float)i * 1.5708f;
            float angle = base_angle + time_ * 0.5f;
            float radius = 6.0f + sinf(time_ * 0.3f + i) * 1.5f;
            float y = 2.0f + sinf(time_ * 0.7f + i * 0.8f) * 1.5f;
            float x = cosf(angle) * radius;
            float z = sinf(angle) * radius;
            point_lights_[i].set_position(Vec3(x, y, z));
        }

        // Animate spot light direction
        float spot_angle = time_ * 0.3f;
        float sx = sinf(spot_angle) * 0.5f;
        float sz = cosf(spot_angle) * 0.5f;
        CLight spot{};
        spot.light_type = 2;
        spot.position = {sx * 3.0f, 6, sz * 3.0f};
        spot.direction = {-sx, -1, -sz};
        spot.color = {1, 1, 1};
        spot.intensity = 5.0f + sinf(time_ * 2.0f) * 2.0f;
        spot.range = 15.0f;
        spot.inner_angle = 20.0f;
        spot.outer_angle = 30.0f;
        spot_entity_.set_light(spot);
        spot_entity_.set_position(Vec3(sx * 3.0f, 6, sz * 3.0f));

        if (reactor_get_frame_count() % 120 == 0) {
            printf("\rFPS: %.1f | Lights: %u    ", Time::fps(), reactor_light_count());
            fflush(stdout);
        }

        if (Input::key_pressed(Input::KEY_ESCAPE())) Window::request_close();
    }

    void on_render() override {}

    void on_shutdown() override {
        for (auto& l : point_lights_) l.destroy();
        spot_entity_.destroy();
        printf("\nLighting demo shutdown.\n");
    }
};

int main() {
    LightingDemo app;
    return app.run();
}
