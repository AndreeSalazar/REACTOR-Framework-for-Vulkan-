// =============================================================================
// hello_cpp — Example C++ game using REACTOR
// =============================================================================
// This demonstrates the ONE CALL ReactorApp() pattern with Scene API.
// Ultra-simple, ultra-productive, ultra-powerful.
//
// Build:
//   1. cargo build --release -p reactor-c-api
//   2. cmake -B build && cmake --build build
// =============================================================================

#include <reactor/reactor.hpp>
#include <cstdio>

// =============================================================================
// Example 1: Class-based with Scene API (recommended for larger games)
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
        
        // Setup lighting using Lighting API
        reactor::Lighting::add_directional(
            {-0.5f, -1.0f, -0.3f},  // direction
            {1.0f, 0.98f, 0.95f},   // warm white color
            1.0f                     // intensity
        );
        reactor::Lighting::add_point(
            {2.0f, 2.0f, 2.0f},     // position
            {0.3f, 0.5f, 1.0f},     // blue color
            0.5f,                    // intensity
            10.0f                    // range
        );
        printf("  Lights added: %u\n", reactor::Lighting::count());
        
        // Test SDF functions
        float sphere_dist = reactor::SDF::sphere({0.5f, 0, 0}, 1.0f);
        float box_dist = reactor::SDF::box({0.3f, 0.3f, 0.3f}, {0.5f, 0.5f, 0.5f});
        
        printf("  SDF sphere at (0.5,0,0): %.3f\n", sphere_dist);
        printf("  SDF box at (0.3,0.3,0.3): %.3f\n", box_dist);
        
        // Set initial camera
        reactor::Camera::set_position(camera_pos_);
        reactor::Camera::set_target({0, 0, 0});
        
        // Scene info
        printf("  Scene objects: %u\n", reactor::Scene::object_count());
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
        
        // Update object transforms in scene (if any)
        uint32_t obj_count = reactor::Scene::object_count();
        for (uint32_t i = 0; i < obj_count; ++i) {
            auto transform = reactor::Mat4::RotationY(rotation_ + i * 0.5f);
            reactor::Scene::set_transform(i, transform);
        }
        
        // Show FPS every 60 frames
        if (reactor::Time::frame_count() % 60 == 0) {
            printf("\rFPS: %.1f  Camera: (%.1f, %.1f, %.1f)  Objects: %u    ",
                reactor::Time::fps(),
                camera_pos_.x, camera_pos_.y, camera_pos_.z,
                obj_count);
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
        
        // Scene is rendered automatically by REACTOR
    }

    void on_shutdown() override {
        printf("\n");
        reactor::Log::info("HelloReactor shutdown!");
        
        // Clear scene and lights
        reactor::Scene::clear();
        reactor::Lighting::clear();
    }
};

// =============================================================================
// Example 2: Functional style with Scene API
// =============================================================================

void run_functional_example() {
    float rotation = 0.0f;
    
    reactor::ReactorApp(
        reactor::Config("Functional REACTOR").with_size(800, 600),
        
        // on_init
        []() {
            reactor::Log::info("Functional example initialized!");
            
            // Add lighting
            reactor::Lighting::add_directional({0, -1, 0}, {1, 1, 1}, 1.0f);
        },
        
        // on_update
        [&rotation](float dt) {
            rotation += dt;
            
            // Update all objects in scene
            for (uint32_t i = 0; i < reactor::Scene::object_count(); ++i) {
                reactor::Scene::set_transform(i, reactor::Mat4::RotationY(rotation));
            }
            
            if (reactor::Input::key_pressed(reactor::Input::KEY_ESCAPE())) {
                reactor::Window::request_close();
            }
        },
        
        // on_render
        [&rotation]() {
            // Scene rendered automatically
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
    printf("║       REACTOR C++ SDK — Scene API Example                    ║\n");
    printf("╚══════════════════════════════════════════════════════════════╝\n");
    printf("\n");
    printf("Features demonstrated:\n");
    printf("  - Scene API: object management, transforms\n");
    printf("  - Lighting API: directional, point, spot lights\n");
    printf("  - Camera API: position, target, view-projection\n");
    printf("  - Input API: keyboard, mouse\n");
    printf("  - SDF API: signed distance functions\n");
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
