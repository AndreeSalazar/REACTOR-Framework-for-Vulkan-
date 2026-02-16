// =============================================================================
// simple_cube.cpp — THE ONE CALL Pattern Demo (C++)
// =============================================================================
// The absolute minimum code to render a 3D cube with REACTOR in C++.
// Mirrors the Rust simple_cube.rs example.
//
// PATTERN: Inherit → Override → Run
//   1. Create class inheriting from reactor::Application
//   2. Override config(), on_init(), on_update()
//   3. Call run() in main()
//
// That's it. No boilerplate. No Vulkan. No window management.
// =============================================================================

#include <reactor/reactor.hpp>

using namespace reactor;

// =============================================================================
// YOUR GAME — Just state + logic
// =============================================================================

class SimpleCube : public Application {
    float rotation = 0.0f;

public:
    // -------------------------------------------------------------------------
    // CONFIG — One place to configure everything
    // -------------------------------------------------------------------------
    Config config() override {
        return Config("🎲 Simple Cube C++")
            .with_size(1280, 720)
            .with_vsync(true)
            .with_msaa(4);
    }

    // -------------------------------------------------------------------------
    // INIT — Setup your scene once
    // -------------------------------------------------------------------------
    void on_init() override {
        // Camera
        Camera::set_position({0.0f, 2.0f, 4.0f});
        Camera::look_at({0.0f, 0.0f, 0.0f});

        // Light
        Lighting::add_directional(
            Vec3(-0.5f, -1.0f, -0.3f).normalized(),
            Vec3(1.0f, 1.0f, 1.0f),
            1.0f
        );

        // TODO: Add cube mesh when mesh creation API is exposed
        // For now, this demonstrates the pattern
        Log::info("SimpleCube initialized!");
    }

    // -------------------------------------------------------------------------
    // UPDATE — Game logic every frame
    // -------------------------------------------------------------------------
    void on_update(float dt) override {
        rotation += dt * 1.5f;
        
        // Update cube transform
        if (Scene::object_count() > 0) {
            Scene::set_transform(0, Mat4::rotation_y(rotation) * Mat4::rotation_x(rotation * 0.7f));
        }

        // ESC to exit
        if (Input::key_down(Input::KEY_ESCAPE())) {
            Window::request_close();
        }
    }
};

// =============================================================================
// MAIN — THE ONE CALL
// =============================================================================

int main() {
    return SimpleCube().run();
}

// =============================================================================
// ALTERNATIVE: Ultra-simple lambda pattern (future)
// =============================================================================
// 
// int main() {
//     return ReactorApp({
//         .title = "Simple Cube",
//         .width = 1280,
//         .height = 720,
//         .on_update = [](float dt) {
//             static float rotation = 0;
//             rotation += dt * 1.5f;
//             Scene::set_transform(0, Mat4::rotation_y(rotation));
//         }
//     });
// }
