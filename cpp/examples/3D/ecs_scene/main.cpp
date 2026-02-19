// =============================================================================
// REACTOR — ECS Scene Example (C++)
// =============================================================================
// Demonstrates the full Entity-Component System:
//   - Entity creation/destruction
//   - Transform, MeshRenderer, Light, Camera, RigidBody components
//   - Component queries with bitmask filters
//   - Entity active/inactive toggling
//
// This shows how C++ and Rust share the same ECS backend seamlessly.
// =============================================================================

#include <reactor/application.hpp>
#include <cstdio>

using namespace reactor;

class ECSDemo : public Application {
    Entity player_;
    Entity ground_;
    Entity sun_light_;
    Entity camera_entity_;
    Entity cubes_[5];
    
    MeshHandle* cube_mesh_ = nullptr;
    MaterialHandle* cube_mat_ = nullptr;
    float time_ = 0.0f;

public:
    Config config() override {
        return Config("REACTOR — ECS Scene Demo", 1280, 720)
            .with_msaa(4)
            .with_vsync(true);
    }

    void on_init() override {
        printf("=== REACTOR ECS Scene Demo ===\n\n");

        // Create GPU resources
        cube_mesh_ = reactor_create_cube();
        cube_mat_ = reactor_create_material_simple(0.8f, 0.3f, 0.2f);

        // --- Player entity ---
        player_ = Entity::create("Player");
        player_.set_position(Vec3(0, 1, 0));
        player_.set_scale(Vec3(1, 2, 1));
        player_.add_mesh_renderer(0, 0);
        player_.add_rigidbody(80.0f, true);
        printf("Created Player (id=%u)\n", player_.id());

        // --- Ground entity ---
        ground_ = Entity::create("Ground");
        ground_.set_position(Vec3(0, -0.5f, 0));
        ground_.set_scale(Vec3(20, 1, 20));
        ground_.add_mesh_renderer(0, 0);
        printf("Created Ground (id=%u)\n", ground_.id());

        // --- Sun light entity ---
        sun_light_ = Entity::create("SunLight");
        sun_light_.set_position(Vec3(10, 20, 10));
        CLight sun{};
        sun.light_type = 0; // Directional
        sun.direction = {-0.5f, -1.0f, -0.3f};
        sun.color = {1.0f, 0.95f, 0.9f};
        sun.intensity = 1.2f;
        sun_light_.add_light(sun);
        printf("Created SunLight (id=%u)\n", sun_light_.id());

        // --- Camera entity ---
        camera_entity_ = Entity::create("MainCamera");
        camera_entity_.set_position(Vec3(0, 5, 10));
        camera_entity_.add_camera(60.0f, 0.1f, 1000.0f, true);
        printf("Created MainCamera (id=%u)\n", camera_entity_.id());

        // --- Cube array ---
        for (int i = 0; i < 5; ++i) {
            char name[32];
            snprintf(name, sizeof(name), "Cube_%d", i);
            cubes_[i] = Entity::create(name);
            cubes_[i].set_position(Vec3(-4.0f + i * 2.0f, 0.5f, -3.0f));
            cubes_[i].set_scale(Vec3(0.8f));
            cubes_[i].add_mesh_renderer(0, 0);
            printf("Created %s (id=%u)\n", name, cubes_[i].id());
        }

        // Add cube to Vulkan scene
        if (cube_mesh_ && cube_mat_) {
            CMat4 identity{};
            identity.cols[0][0] = 1; identity.cols[1][1] = 1;
            identity.cols[2][2] = 1; identity.cols[3][3] = 1;
            reactor_add_object(cube_mesh_, cube_mat_, identity);
        }

        // Setup Vulkan camera
        reactor_set_camera_position(0, 5, 10);
        reactor_set_camera_target(0, 0, 0);
        reactor_add_directional_light(-0.5f, -1.0f, -0.3f, 1, 0.95f, 0.9f, 1.2f);

        // --- Print entity stats ---
        printf("\n--- Entity Stats ---\n");
        printf("Total entities: %u\n", reactor_entity_count());

        // Query by component
        auto renderers = ECS::query(COMPONENT_MESH_RENDERER);
        printf("Entities with MeshRenderer: %zu\n", renderers.size());

        auto lights = ECS::query(COMPONENT_LIGHT);
        printf("Entities with Light: %zu\n", lights.size());

        auto cameras = ECS::query(COMPONENT_CAMERA);
        printf("Entities with Camera: %zu\n", cameras.size());

        auto bodies = ECS::query(COMPONENT_RIGIDBODY);
        printf("Entities with RigidBody: %zu\n", bodies.size());

        printf("\nControls: ESC = Exit\n\n");
    }

    void on_update(float dt) override {
        time_ += dt;

        // Animate cubes
        for (int i = 0; i < 5; ++i) {
            float y = 0.5f + sinf(time_ * 2.0f + i * 1.2f) * 0.5f;
            cubes_[i].set_position(Vec3(-4.0f + i * 2.0f, y, -3.0f));
            cubes_[i].set_rotation(Vec3(0, time_ + i * 0.5f, 0));
        }

        // Apply force to player every 3 seconds
        if (fmodf(time_, 3.0f) < dt) {
            player_.apply_force(Vec3(0, 500, 0));
        }

        // Toggle cube 2 visibility every 2 seconds
        bool visible = fmodf(time_, 2.0f) < 1.0f;
        cubes_[2].set_active(visible);

        // Print stats periodically
        if (reactor_get_frame_count() % 120 == 0) {
            printf("\rFPS: %.1f | Entities: %u | Player vel: (%.1f, %.1f, %.1f)    ",
                Time::fps(), reactor_entity_count(),
                player_.velocity().x, player_.velocity().y, player_.velocity().z);
            fflush(stdout);
        }

        if (Input::key_pressed(Input::KEY_ESCAPE())) {
            Window::request_close();
        }
    }

    void on_render() override {}

    void on_shutdown() override {
        // Destroy all entities
        player_.destroy();
        ground_.destroy();
        sun_light_.destroy();
        camera_entity_.destroy();
        for (auto& c : cubes_) c.destroy();

        printf("\n\nAll entities destroyed. Final count: %u\n", reactor_entity_count());
    }
};

int main() {
    ECSDemo app;
    return app.run();
}
