// =============================================================================
// REACTOR — FPS Controller Example (C++)
// =============================================================================
// Demonstrates the character controller and physics:
//   - WASD movement with CharacterController
//   - Mouse look (right-click drag)
//   - Jump with Space
//   - Gravity and ground collision
//   - ECS rigidbody entities with forces
//
// Physics runs in Rust, controlled from C++ — best of both worlds.
// =============================================================================

#include <reactor/application.hpp>
#include <cstdio>
#include <cmath>

using namespace reactor;

class FPSDemo : public Application {
    CharacterController player_;
    float yaw_ = 0.0f;
    float pitch_ = 0.0f;

    MeshHandle* cube_mesh_ = nullptr;
    MaterialHandle* ground_mat_ = nullptr;
    MaterialHandle* wall_mat_ = nullptr;
    MaterialHandle* crate_mat_ = nullptr;

    Entity physics_cubes_[8];
    float time_ = 0.0f;

public:
    Config config() override {
        return Config("REACTOR — FPS Controller", 1280, 720)
            .with_msaa(4)
            .with_physics_hz(60);
    }

    void on_init() override {
        printf("=== REACTOR FPS Controller Demo ===\n\n");

        cube_mesh_ = reactor_create_cube();
        ground_mat_ = reactor_create_material_simple(0.3f, 0.5f, 0.3f);
        wall_mat_ = reactor_create_material_simple(0.6f, 0.6f, 0.65f);
        crate_mat_ = reactor_create_material_simple(0.7f, 0.5f, 0.2f);

        // Player setup
        player_ = CharacterController(Vec3(0, 1, 5));
        player_.set_move_speed(5.0f);
        player_.set_jump_force(8.0f);
        player_.set_gravity(-20.0f);

        // Build environment
        if (cube_mesh_) {
            // Ground
            if (ground_mat_) {
                CMat4 t{};
                t.cols[0][0] = 30; t.cols[1][1] = 0.2f; t.cols[2][2] = 30; t.cols[3][3] = 1;
                t.cols[3][1] = -0.1f;
                reactor_add_object(cube_mesh_, ground_mat_, t);
            }

            // Walls
            if (wall_mat_) {
                float walls[][6] = {
                    {0, 2, -15, 30, 4, 0.5f},  // Back wall
                    {0, 2, 15, 30, 4, 0.5f},    // Front wall
                    {-15, 2, 0, 0.5f, 4, 30},   // Left wall
                    {15, 2, 0, 0.5f, 4, 30},    // Right wall
                };
                for (auto& w : walls) {
                    CMat4 t{};
                    t.cols[0][0] = w[3]; t.cols[1][1] = w[4]; t.cols[2][2] = w[5]; t.cols[3][3] = 1;
                    t.cols[3][0] = w[0]; t.cols[3][1] = w[1]; t.cols[3][2] = w[2];
                    reactor_add_object(cube_mesh_, wall_mat_, t);
                }
            }

            // Crates (physics cubes in ECS)
            if (crate_mat_) {
                for (int i = 0; i < 8; ++i) {
                    float x = -6.0f + (i % 4) * 4.0f;
                    float z = -3.0f + (i / 4) * 6.0f;

                    CMat4 t{};
                    t.cols[0][0] = 1; t.cols[1][1] = 1; t.cols[2][2] = 1; t.cols[3][3] = 1;
                    t.cols[3][0] = x; t.cols[3][1] = 0.5f; t.cols[3][2] = z;
                    reactor_add_object(cube_mesh_, crate_mat_, t);

                    // Also create ECS entity with rigidbody
                    char name[32];
                    snprintf(name, sizeof(name), "Crate_%d", i);
                    physics_cubes_[i] = Entity::create(name);
                    physics_cubes_[i].set_position(Vec3(x, 0.5f, z));
                    physics_cubes_[i].add_mesh_renderer(0, 0);
                    physics_cubes_[i].add_rigidbody(10.0f, true);
                }
            }
        }

        // Lighting
        reactor_add_directional_light(-0.3f, -1.0f, -0.5f, 1, 0.95f, 0.9f, 1.0f);
        reactor_add_point_light(0, 5, 0, 1, 0.8f, 0.6f, 3.0f, 15.0f);

        printf("Controls:\n");
        printf("  WASD    - Move\n");
        printf("  Mouse   - Look (right-click hold)\n");
        printf("  Space   - Jump\n");
        printf("  ESC     - Exit\n\n");
    }

    void on_update(float dt) override {
        time_ += dt;

        // --- Mouse look ---
        if (Input::mouse_right()) {
            Vec2 delta = Input::mouse_delta();
            yaw_ -= delta.x * 0.003f;
            pitch_ -= delta.y * 0.003f;
            pitch_ = std::clamp(pitch_, -1.4f, 1.4f);
        }

        // --- Movement input ---
        Vec3 move_input(0, 0, 0);
        float cy = cosf(yaw_), sy = sinf(yaw_);

        if (Input::key_down(Input::KEY_W())) { move_input.x -= sy; move_input.z -= cy; }
        if (Input::key_down(Input::KEY_S())) { move_input.x += sy; move_input.z += cy; }
        if (Input::key_down(Input::KEY_A())) { move_input.x -= cy; move_input.z += sy; }
        if (Input::key_down(Input::KEY_D())) { move_input.x += cy; move_input.z -= sy; }

        if (move_input.length_squared() > 0.01f) {
            move_input = move_input.normalized();
        }

        bool jump = Input::key_pressed(Input::KEY_SPACE());
        player_.update(dt, move_input, jump, 0.0f);

        // --- Camera follows player ---
        Vec3 eye = player_.eye_position();
        float look_dist = 100.0f;
        Vec3 forward(
            -sinf(yaw_) * cosf(pitch_),
            sinf(pitch_),
            -cosf(yaw_) * cosf(pitch_)
        );
        Vec3 target = eye + forward * look_dist;
        reactor_set_camera_position(eye.x, eye.y, eye.z);
        reactor_set_camera_target(target.x, target.y, target.z);

        // --- Stats ---
        if (reactor_get_frame_count() % 60 == 0) {
            Vec3 pos = player_.position();
            printf("\rFPS: %.0f | Pos: (%.1f, %.1f, %.1f) | Grounded: %s    ",
                Time::fps(), pos.x, pos.y, pos.z,
                player_.is_grounded() ? "YES" : "NO");
            fflush(stdout);
        }

        if (Input::key_pressed(Input::KEY_ESCAPE())) Window::request_close();
    }

    void on_render() override {}

    void on_shutdown() override {
        for (auto& c : physics_cubes_) c.destroy();
        printf("\nFPS demo shutdown.\n");
    }
};

int main() {
    FPSDemo app;
    return app.run();
}
