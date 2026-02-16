// =============================================================================
// REACTOR 3D — The Ultimate Simple C++ Example
// =============================================================================
// This is the DEFINITIVE example of how to use REACTOR in C++.
// ONE CALL. ONE FILE. ZERO BOILERPLATE.
//
// Usage:
//   1. Include reactor.hpp
//   2. Call ReactorApp() or inherit from Application
//   3. That's it. You're rendering 3D with Vulkan.
//
// Build:
//   cmake -B build && cmake --build build
//   ./build/reactor_3d
// =============================================================================

#include <reactor/reactor.hpp>
#include <cstdio>

using namespace reactor;

// =============================================================================
// OPTION 1: THE ONE CALL (Absolute Minimum)
// =============================================================================
// Just call ReactorApp() and you have a Vulkan window.
// Perfect for quick prototypes and testing.

void example_one_call() {
    ReactorApp("REACTOR 3D - One Call");
}

// =============================================================================
// OPTION 2: LAMBDA STYLE (Quick Prototyping)
// =============================================================================
// Add callbacks for init, update, render without creating a class.
// Perfect for small demos and experiments.

void example_lambda() {
    float rotation = 0.0f;
    Vec3 camera_pos{0, 3, 8};
    
    ReactorApp(
        Config("REACTOR 3D - Lambda Style")
            .with_size(1280, 720)
            .with_vsync(true)
            .with_msaa(4),
        
        // ON INIT
        [&]() {
            printf("╔══════════════════════════════════════════════════════════════╗\n");
            printf("║           REACTOR 3D — Lambda Style Demo                     ║\n");
            printf("╚══════════════════════════════════════════════════════════════╝\n");
            printf("\n");
            printf("Controls:\n");
            printf("  WASD      - Move camera\n");
            printf("  Space     - Move up\n");
            printf("  Shift     - Move down\n");
            printf("  ESC       - Exit\n");
            printf("\n");
            
            // Setup lighting
            Lighting::add_directional({-0.5f, -1.0f, -0.3f}, {1.0f, 0.98f, 0.95f}, 1.0f);
            Lighting::add_point({3, 2, 3}, {0.2f, 0.5f, 1.0f}, 0.5f, 15.0f);
            
            // Setup camera
            Camera::set_position(camera_pos);
            Camera::set_target({0, 0, 0});
            
            // Enable post-processing
            PostProcess::set_bloom(true, 0.8f, 1.0f);
            PostProcess::set_tonemapping(true, 1.2f);
            
            // Print GPU info
            printf("GPU: %s\n", GPUInfo::name());
            printf("MSAA: %ux\n", GPUInfo::msaa_samples());
            printf("Ray Tracing: %s\n", GPUInfo::raytracing_supported() ? "Yes" : "No");
            printf("\n");
        },
        
        // ON UPDATE
        [&](float dt) {
            rotation += dt;
            
            // Camera movement
            float speed = 5.0f * dt;
            if (Input::key_down(Input::KEY_W())) camera_pos.z -= speed;
            if (Input::key_down(Input::KEY_S())) camera_pos.z += speed;
            if (Input::key_down(Input::KEY_A())) camera_pos.x -= speed;
            if (Input::key_down(Input::KEY_D())) camera_pos.x += speed;
            if (Input::key_down(Input::KEY_SPACE())) camera_pos.y += speed;
            if (Input::key_down(Input::KEY_SHIFT())) camera_pos.y -= speed;
            
            Camera::set_position(camera_pos);
            
            // Update scene objects
            for (uint32_t i = 0; i < Scene::object_count(); ++i) {
                Scene::set_transform(i, Mat4::RotationY(rotation + i * 0.5f));
            }
            
            // Exit on ESC
            if (Input::key_pressed(Input::KEY_ESCAPE())) {
                Window::request_close();
            }
            
            // FPS display
            if (Time::frame_count() % 60 == 0) {
                printf("\rFPS: %.1f | Camera: (%.1f, %.1f, %.1f) | Objects: %u    ",
                    Time::fps(), camera_pos.x, camera_pos.y, camera_pos.z,
                    Scene::object_count());
                fflush(stdout);
            }
        },
        
        // ON RENDER
        [&]() {
            // Scene is rendered automatically by REACTOR
            // You can add debug visualization here
            Debug::grid(20.0f, 20, 0.3f, 0.3f, 0.3f);
        }
    );
}

// =============================================================================
// OPTION 3: CLASS STYLE (Full Control)
// =============================================================================
// Inherit from Application for maximum control and organization.
// Perfect for larger projects and games.

class Reactor3D : public Application {
    float rotation_ = 0.0f;
    Vec3 camera_pos_{0, 3, 8};
    float yaw_ = 0.0f;
    float pitch_ = -0.3f;
    bool mouse_captured_ = false;

public:
    Config config() override {
        return Config("REACTOR 3D — Full Control")
            .with_size(1280, 720)
            .with_vsync(true)
            .with_msaa(4);
    }

    void on_init() override {
        printf("╔══════════════════════════════════════════════════════════════╗\n");
        printf("║           REACTOR 3D — Class Style Demo                      ║\n");
        printf("╚══════════════════════════════════════════════════════════════╝\n");
        printf("\n");
        printf("Controls:\n");
        printf("  WASD       - Move camera\n");
        printf("  Space      - Move up\n");
        printf("  Shift      - Move down / Sprint\n");
        printf("  Mouse      - Look around (click to capture)\n");
        printf("  ESC        - Release mouse / Exit\n");
        printf("  1-4        - Toggle post-processing effects\n");
        printf("\n");
        
        // Lighting
        Lighting::add_directional({-0.5f, -1.0f, -0.3f}, {1.0f, 0.98f, 0.95f}, 1.0f);
        Lighting::add_point({5, 2, 0}, {1.0f, 0.3f, 0.1f}, 0.8f, 10.0f);
        Lighting::add_point({-5, 2, 0}, {0.1f, 0.3f, 1.0f}, 0.8f, 10.0f);
        Lighting::add_point({0, 2, 5}, {0.1f, 1.0f, 0.3f}, 0.8f, 10.0f);
        
        // Camera
        Camera::set_position(camera_pos_);
        Camera::set_target({0, 0, 0});
        
        // Post-processing
        PostProcess::set_bloom(true, 1.0f, 0.8f);
        PostProcess::set_tonemapping(true, 1.0f);
        PostProcess::set_vignette(true, 0.3f);
        PostProcess::set_fxaa(true);
        
        // GPU Info
        printf("GPU: %s\n", GPUInfo::name());
        printf("MSAA: %ux\n", GPUInfo::msaa_samples());
        
        uint32_t major, minor, patch;
        GPUInfo::vulkan_version(major, minor, patch);
        printf("Vulkan: %u.%u.%u\n", major, minor, patch);
        printf("\n");
        
        Log::info("REACTOR 3D initialized!");
    }

    void on_update(float dt) override {
        rotation_ += dt;
        
        // Mouse capture toggle
        if (Input::mouse_left() && !mouse_captured_) {
            mouse_captured_ = true;
        }
        
        // ESC handling
        if (Input::key_pressed(Input::KEY_ESCAPE())) {
            if (mouse_captured_) {
                mouse_captured_ = false;
            } else {
                Window::request_close();
            }
        }
        
        // Mouse look
        if (mouse_captured_) {
            Vec2 delta = Input::mouse_delta();
            yaw_ -= delta.x * 0.002f;
            pitch_ -= delta.y * 0.002f;
            pitch_ = std::max(-1.4f, std::min(1.4f, pitch_));
        }
        
        // Camera movement
        float speed = 5.0f * dt;
        if (Input::key_down(Input::KEY_SHIFT())) speed *= 2.0f;
        
        Vec3 forward{-sinf(yaw_), 0, -cosf(yaw_)};
        Vec3 right{cosf(yaw_), 0, -sinf(yaw_)};
        
        if (Input::key_down(Input::KEY_W())) camera_pos_ = camera_pos_ + forward * speed;
        if (Input::key_down(Input::KEY_S())) camera_pos_ = camera_pos_ - forward * speed;
        if (Input::key_down(Input::KEY_D())) camera_pos_ = camera_pos_ + right * speed;
        if (Input::key_down(Input::KEY_A())) camera_pos_ = camera_pos_ - right * speed;
        if (Input::key_down(Input::KEY_SPACE())) camera_pos_.y += speed;
        if (Input::key_down(Input::KEY_SHIFT())) camera_pos_.y -= speed * 0.5f;
        
        Camera::set_position(camera_pos_);
        Vec3 look_target = camera_pos_ + Vec3{
            -sinf(yaw_) * cosf(pitch_),
            sinf(pitch_),
            -cosf(yaw_) * cosf(pitch_)
        };
        Camera::set_target(look_target);
        
        // Post-processing toggles
        static bool bloom = true, tone = true, vignette = true, fxaa = true;
        // Note: Would need key_pressed for 1-4 keys
        
        // Update scene objects
        for (uint32_t i = 0; i < Scene::object_count(); ++i) {
            float offset = i * 0.5f;
            Scene::set_transform(i, 
                Mat4::RotationY(rotation_ + offset) * 
                Mat4::RotationX(rotation_ * 0.7f + offset)
            );
        }
        
        // Animation update
        Animation::update(dt);
        
        // FPS display
        if (Time::frame_count() % 60 == 0) {
            printf("\rFPS: %.1f | Pos: (%.1f, %.1f, %.1f) | Mouse: %s    ",
                Time::fps(), 
                camera_pos_.x, camera_pos_.y, camera_pos_.z,
                mouse_captured_ ? "Captured" : "Free");
            fflush(stdout);
        }
    }

    void on_render() override {
        // Debug visualization
        Debug::grid(20.0f, 20, 0.2f, 0.2f, 0.2f);
        
        // Debug axes at origin
        Debug::line(0, 0, 0, 2, 0, 0, 1, 0, 0); // X = Red
        Debug::line(0, 0, 0, 0, 2, 0, 0, 1, 0); // Y = Green
        Debug::line(0, 0, 0, 0, 0, 2, 0, 0, 1); // Z = Blue
    }

    void on_shutdown() override {
        printf("\n");
        Log::info("REACTOR 3D shutdown!");
        Scene::clear();
        Lighting::clear();
        Debug::clear();
    }
};

// =============================================================================
// MAIN — Choose your style
// =============================================================================

int main(int argc, char** argv) {
    printf("\n");
    printf("╔══════════════════════════════════════════════════════════════╗\n");
    printf("║              REACTOR 3D — C++ Vulkan Framework               ║\n");
    printf("║                      Version 1.0.5                           ║\n");
    printf("╚══════════════════════════════════════════════════════════════╝\n");
    printf("\n");
    printf("Select example mode:\n");
    printf("  1 = One Call (absolute minimum)\n");
    printf("  2 = Lambda Style (quick prototyping)\n");
    printf("  3 = Class Style (full control) [default]\n");
    printf("\n");
    
    int mode = 3;
    if (argc > 1) {
        mode = atoi(argv[1]);
    }
    
    switch (mode) {
        case 1:
            printf("Running: One Call Example\n\n");
            example_one_call();
            break;
        case 2:
            printf("Running: Lambda Style Example\n\n");
            example_lambda();
            break;
        case 3:
        default:
            printf("Running: Class Style Example\n\n");
            return Reactor3D().run();
    }
    
    return 0;
}
