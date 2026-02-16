// =============================================================================
// hello_cpp — Example C++ game using REACTOR
// =============================================================================
// This demonstrates the ONE CALL ReactorApp() pattern.
// Ultra-simple, ultra-productive, ultra-powerful.
//
// Build:
//   1. cargo build --release -p reactor-c-api
//   2. cmake -B build && cmake --build build
// =============================================================================

#include <reactor/reactor.hpp>
#include <cstdio>

// =============================================================================
// Example 1: Class-based (recommended for larger games)
// =============================================================================

class HelloReactor : public reactor::Application {
    float rotation_ = 0.0f;
    reactor::Vec3 camera_pos_{0, 2, 5};

public:
    reactor::Config config() override {
        return reactor::Config("Hello REACTOR C++")
            .with_size(1280, 720)
            .with_vsync(true);
    }

    void on_init() override {
        reactor::Log::info("HelloReactor initialized!");
        
        // Test SDF functions
        float sphere_dist = reactor::SDF::sphere({0.5f, 0, 0}, 1.0f);
        float box_dist = reactor::SDF::box({0.3f, 0.3f, 0.3f}, {0.5f, 0.5f, 0.5f});
        
        printf("  SDF sphere at (0.5,0,0): %.3f\n", sphere_dist);
        printf("  SDF box at (0.3,0.3,0.3): %.3f\n", box_dist);
        
        // Set initial camera
        reactor::Camera::set_position(camera_pos_);
        reactor::Camera::set_target({0, 0, 0});
    }

    void on_update(float dt) override {
        rotation_ += dt;
        
        // Camera movement with WASD
        float speed = 5.0f * dt;
        
        if (reactor::Input::key_down(reactor::Input::KEY_W())) {
            camera_pos_.z -= speed;
        }
        if (reactor::Input::key_down(reactor::Input::KEY_S())) {
            camera_pos_.z += speed;
        }
        if (reactor::Input::key_down(reactor::Input::KEY_A())) {
            camera_pos_.x -= speed;
        }
        if (reactor::Input::key_down(reactor::Input::KEY_D())) {
            camera_pos_.x += speed;
        }
        if (reactor::Input::key_down(reactor::Input::KEY_SPACE())) {
            camera_pos_.y += speed;
        }
        if (reactor::Input::key_down(reactor::Input::KEY_SHIFT())) {
            camera_pos_.y -= speed;
        }
        
        // Exit on Escape
        if (reactor::Input::key_pressed(reactor::Input::KEY_ESCAPE())) {
            reactor::Window::request_close();
        }
        
        // Update camera
        reactor::Camera::set_position(camera_pos_);
        
        // Show FPS every 60 frames
        if (reactor::Time::frame_count() % 60 == 0) {
            printf("\rFPS: %.1f  Camera: (%.1f, %.1f, %.1f)    ",
                reactor::Time::fps(),
                camera_pos_.x, camera_pos_.y, camera_pos_.z);
            fflush(stdout);
        }
    }

    void on_render() override {
        // Get view-projection matrix
        auto vp = reactor::Camera::view_projection();
        
        // Create model matrix with rotation
        auto model = reactor::Mat4::RotationY(rotation_);
        
        // MVP = VP * Model
        auto mvp = vp * model;
        
        // In full integration: draw meshes with mvp
    }

    void on_shutdown() override {
        printf("\n");
        reactor::Log::info("HelloReactor shutdown!");
    }
};

// =============================================================================
// Example 2: Functional style (ultra-simple for small demos)
// =============================================================================

void run_functional_example() {
    float rotation = 0.0f;
    
    reactor::ReactorApp(
        reactor::Config("Functional REACTOR").with_size(800, 600),
        
        // on_init
        []() {
            reactor::Log::info("Functional example initialized!");
        },
        
        // on_update
        [&rotation](float dt) {
            rotation += dt;
            
            if (reactor::Input::key_pressed(reactor::Input::KEY_ESCAPE())) {
                reactor::Window::request_close();
            }
        },
        
        // on_render
        [&rotation]() {
            auto model = reactor::Mat4::RotationY(rotation);
            // Draw with model matrix...
        }
    );
}

// =============================================================================
// Example 3: Minimal (THE ONE CALL)
// =============================================================================

void run_minimal_example() {
    // This is the absolute minimum — just opens a window
    reactor::ReactorApp("Minimal REACTOR");
}

// =============================================================================
// Main — Choose which example to run
// =============================================================================

int main() {
    printf("╔══════════════════════════════════════════════════════════════╗\n");
    printf("║           REACTOR C++ SDK — Hello World Example              ║\n");
    printf("╚══════════════════════════════════════════════════════════════╝\n");
    printf("\n");
    printf("Controls:\n");
    printf("  WASD      - Move camera\n");
    printf("  Space     - Move up\n");
    printf("  Shift     - Move down\n");
    printf("  Escape    - Exit\n");
    printf("\n");
    
    // Run the class-based example (most complete)
    HelloReactor game;
    return game.run();
    
    // Alternative: run functional example
    // run_functional_example();
    // return 0;
    
    // Alternative: run minimal example
    // run_minimal_example();
    // return 0;
}
