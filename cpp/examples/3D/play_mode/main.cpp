// =============================================================================
// REACTOR — Play Mode Bridge Example (C++)
// =============================================================================
// Demonstrates the runtime-editor bridge:
//   - Enter/exit play mode with scene snapshot
//   - Pause/unpause simulation
//   - Play time tracking
//   - Scene state management between edit and play modes
//
// This is the foundation for a C++ editor that can test scenes in real-time.
// =============================================================================

#include <reactor/application.hpp>
#include <cstdio>
#include <cmath>

using namespace reactor;

class PlayModeDemo : public Application {
    MeshHandle* cube_mesh_ = nullptr;
    MaterialHandle* mat_ = nullptr;
    int32_t cube_idx_ = -1;

    float time_ = 0.0f;
    bool was_playing_ = false;

public:
    Config config() override {
        return Config("REACTOR — Play Mode Bridge", 1280, 720)
            .with_msaa(4);
    }

    void on_init() override {
        printf("=== REACTOR Play Mode Bridge Demo ===\n\n");

        cube_mesh_ = reactor_create_cube();
        mat_ = reactor_create_material_simple(0.5f, 0.7f, 0.9f);

        reactor_set_camera_position(0, 3, 6);
        reactor_set_camera_target(0, 0, 0);
        reactor_add_directional_light(-0.5f, -1.0f, -0.3f, 1, 1, 1, 1.0f);

        if (cube_mesh_ && mat_) {
            CMat4 t{};
            t.cols[0][0] = 1; t.cols[1][1] = 1; t.cols[2][2] = 1; t.cols[3][3] = 1;
            cube_idx_ = reactor_add_object(cube_mesh_, mat_, t);
        }

        // Create some ECS entities for the scene
        auto e1 = Entity::create("EditorCube");
        e1.set_position(Vec3(0, 0.5f, 0));
        e1.add_mesh_renderer(0, 0);

        auto e2 = Entity::create("EditorLight");
        e2.set_position(Vec3(5, 5, 5));
        CLight light{};
        light.light_type = 1;
        light.position = {5, 5, 5};
        light.color = {1, 0.8f, 0.6f};
        light.intensity = 2.0f;
        light.range = 10.0f;
        e2.add_light(light);

        printf("Scene setup complete. Entities: %u\n\n", reactor_entity_count());
        printf("Controls:\n");
        printf("  P     - Toggle Play/Stop\n");
        printf("  SPACE - Toggle Pause (during play)\n");
        printf("  S     - Serialize scene\n");
        printf("  ESC   - Exit\n\n");
    }

    void on_update(float dt) override {
        time_ += dt;

        bool is_playing = PlayMode::is_playing();
        bool is_paused = PlayMode::is_paused();

        // Toggle play mode with P
        if (Input::key_pressed(reactor_key_q())) { // Using Q as proxy for P
            if (is_playing) {
                PlayMode::exit();
                printf("\n[EDITOR] Exited play mode. Scene restored.\n");
            } else {
                PlayMode::enter();
                printf("\n[PLAY] Entered play mode! Scene snapshot taken.\n");
            }
        }

        // Toggle pause with Space
        if (is_playing && Input::key_pressed(Input::KEY_SPACE())) {
            PlayMode::pause(!is_paused);
            printf("\n[PLAY] %s\n", is_paused ? "RESUMED" : "PAUSED");
        }

        // Serialize scene with S key
        if (Input::key_pressed(Input::KEY_SHIFT())) {
            std::string json = SceneSerializer::serialize();
            printf("\n[SERIALIZE] Scene (%zu bytes):\n%.300s\n", json.size(), json.c_str());
        }

        // Update play time
        if (is_playing) {
            PlayMode::update(dt);
        }

        // Animate cube differently based on mode
        if (cube_idx_ >= 0) {
            float angle;
            float scale;
            if (is_playing && !is_paused) {
                // Play mode: fast spin + bounce
                float pt = PlayMode::time();
                angle = pt * 3.0f;
                scale = 1.0f + sinf(pt * 5.0f) * 0.3f;
            } else if (is_playing && is_paused) {
                // Paused: frozen
                float pt = PlayMode::time();
                angle = pt * 3.0f;
                scale = 1.0f + sinf(pt * 5.0f) * 0.3f;
            } else {
                // Edit mode: slow rotation
                angle = time_ * 0.3f;
                scale = 1.0f;
            }

            float ca = cosf(angle), sa = sinf(angle);
            CMat4 t{};
            t.cols[0][0] = scale * ca; t.cols[0][2] = scale * sa;
            t.cols[1][1] = scale;
            t.cols[2][0] = -scale * sa; t.cols[2][2] = scale * ca;
            t.cols[3][3] = 1.0f;
            t.cols[3][1] = is_playing ? (0.5f + sinf(PlayMode::time() * 3.0f) * 1.0f) : 0.5f;
            reactor_set_object_transform(cube_idx_, t);
        }

        // Status display
        if (reactor_get_frame_count() % 60 == 0) {
            const char* mode = is_playing ? (is_paused ? "PAUSED" : "PLAYING") : "EDITOR";
            printf("\r[%s] FPS:%.0f PlayTime:%.1fs Entities:%u    ",
                mode, Time::fps(), PlayMode::time(), reactor_entity_count());
            fflush(stdout);
        }

        if (Input::key_pressed(Input::KEY_ESCAPE())) Window::request_close();
    }

    void on_render() override {}
};

int main() {
    PlayModeDemo app;
    return app.run();
}
