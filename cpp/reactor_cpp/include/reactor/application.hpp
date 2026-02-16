// =============================================================================
// REACTOR C++ SDK — Application Base Class
// =============================================================================
// Users inherit from reactor::Application to build games in C++.
// ONE CALL: ReactorApp() initializes everything ultra-intelligently.
//
// Architecture:
//   class MyGame : public reactor::Application
//       → reactor::Application (C++ RAII wrapper)
//           → reactor_c_api.dll (extern "C")
//               → Rust Reactor
//                   → VulkanContext
//                       → GPU
// =============================================================================

#pragma once

#include "types.hpp"
#include <string>
#include <functional>

namespace reactor {

// =============================================================================
// Input — Keyboard and Mouse state
// =============================================================================

struct Input {
    static bool key_down(uint32_t key) { return reactor_key_down(key); }
    static bool key_pressed(uint32_t key) { return reactor_key_pressed(key); }

    static Vec2 mouse_position() { return Vec2(reactor_mouse_position()); }
    static Vec2 mouse_delta() { return Vec2(reactor_mouse_delta()); }
    static bool mouse_button(uint32_t button) { return reactor_mouse_button(button); }
    static bool mouse_left() { return mouse_button(0); }
    static bool mouse_right() { return mouse_button(1); }
    static bool mouse_middle() { return mouse_button(2); }

    // Key codes
    static uint32_t KEY_W() { return reactor_key_w(); }
    static uint32_t KEY_A() { return reactor_key_a(); }
    static uint32_t KEY_S() { return reactor_key_s(); }
    static uint32_t KEY_D() { return reactor_key_d(); }
    static uint32_t KEY_Q() { return reactor_key_q(); }
    static uint32_t KEY_E() { return reactor_key_e(); }
    static uint32_t KEY_SPACE() { return reactor_key_space(); }
    static uint32_t KEY_SHIFT() { return reactor_key_shift(); }
    static uint32_t KEY_CTRL() { return reactor_key_ctrl(); }
    static uint32_t KEY_ESCAPE() { return reactor_key_escape(); }
    static uint32_t KEY_ENTER() { return reactor_key_enter(); }
    static uint32_t KEY_TAB() { return reactor_key_tab(); }
    static uint32_t KEY_UP() { return reactor_key_up(); }
    static uint32_t KEY_DOWN() { return reactor_key_arrow_down(); }
    static uint32_t KEY_LEFT() { return reactor_key_left(); }
    static uint32_t KEY_RIGHT() { return reactor_key_right(); }
};

// =============================================================================
// Time — Frame timing
// =============================================================================

struct Time {
    static float delta() { return reactor_get_delta_time(); }
    static float total() { return reactor_get_total_time(); }
    static float fps() { return reactor_get_fps(); }
    static uint64_t frame_count() { return reactor_get_frame_count(); }
};

// =============================================================================
// Window — Window state
// =============================================================================

struct Window {
    static uint32_t width() { return reactor_get_width(); }
    static uint32_t height() { return reactor_get_height(); }
    static float aspect_ratio() { return reactor_get_aspect_ratio(); }
    static bool should_close() { return reactor_should_close(); }
    static void request_close() { reactor_request_close(); }
};

// =============================================================================
// Camera — Built-in camera control
// =============================================================================

struct Camera {
    static void set_position(const Vec3& pos) {
        reactor_set_camera_position(pos.x, pos.y, pos.z);
    }
    static void set_target(const Vec3& target) {
        reactor_set_camera_target(target.x, target.y, target.z);
    }
    static Vec3 position() {
        return Vec3(reactor_get_camera_position());
    }
    static Mat4 view_projection() {
        return Mat4(reactor_get_view_projection());
    }
};

// =============================================================================
// SDF — Signed Distance Functions (ADead-GPU)
// =============================================================================

struct SDF {
    static float sphere(const Vec3& p, float radius) {
        return reactor_sdf_sphere(p.x, p.y, p.z, radius);
    }
    static float box(const Vec3& p, const Vec3& b) {
        return reactor_sdf_box(p.x, p.y, p.z, b.x, b.y, b.z);
    }
    static float cylinder(const Vec3& p, float h, float r) {
        return reactor_sdf_cylinder(p.x, p.y, p.z, h, r);
    }
    static float torus(const Vec3& p, float r1, float r2) {
        return reactor_sdf_torus(p.x, p.y, p.z, r1, r2);
    }
    static float capsule(const Vec3& p, float h, float r) {
        return reactor_sdf_capsule(p.x, p.y, p.z, h, r);
    }

    static float op_union(float d1, float d2) { return reactor_sdf_union(d1, d2); }
    static float op_subtract(float d1, float d2) { return reactor_sdf_subtract(d1, d2); }
    static float op_intersect(float d1, float d2) { return reactor_sdf_intersect(d1, d2); }
    static float op_smooth_union(float d1, float d2, float k) { return reactor_sdf_smooth_union(d1, d2, k); }
};

// =============================================================================
// Log — Debug logging
// =============================================================================

struct Log {
    static void info(const char* msg) { reactor_log_info(msg); }
    static void warn(const char* msg) { reactor_log_warn(msg); }
    static void error(const char* msg) { reactor_log_error(msg); }
};

// =============================================================================
// Error — Error handling system
// =============================================================================

/// Error codes matching Rust ErrorCode enum
enum class ErrorCode : uint32_t {
    None = 0,
    
    // Vulkan errors (100-199)
    VulkanInstanceCreation = 100,
    VulkanDeviceCreation = 101,
    VulkanSurfaceCreation = 102,
    VulkanSwapchainCreation = 103,
    VulkanRenderPassCreation = 104,
    VulkanPipelineCreation = 105,
    VulkanBufferCreation = 106,
    VulkanImageCreation = 107,
    VulkanMemoryAllocation = 108,
    VulkanCommandBuffer = 109,
    VulkanSynchronization = 110,
    VulkanShaderCompilation = 111,
    VulkanDescriptorSet = 112,
    VulkanValidation = 113,
    
    // Resource errors (200-299)
    FileNotFound = 200,
    InvalidFormat = 201,
    TextureLoadFailed = 202,
    ModelLoadFailed = 203,
    ShaderLoadFailed = 204,
    AssetNotFound = 205,
    
    // Window errors (300-399)
    WindowCreation = 300,
    EventLoopError = 301,
    
    // System errors (400-499)
    OutOfMemory = 400,
    InvalidParameter = 401,
    NotInitialized = 402,
    AlreadyInitialized = 403,
    NotSupported = 404,
    InternalError = 405,
    
    // Scene errors (500-599)
    InvalidObjectIndex = 500,
    InvalidMeshHandle = 501,
    InvalidMaterialHandle = 502,
    
    Unknown = 999,
};

struct Error {
    /// Get the last error code (None = no error)
    static ErrorCode code() {
        return static_cast<ErrorCode>(reactor_get_last_error());
    }
    
    /// Get the last error message (nullptr if no error)
    static const char* message() {
        return reactor_get_error_message();
    }
    
    /// Check if there's a pending error
    static bool has_error() {
        return reactor_has_error();
    }
    
    /// Clear the last error
    static void clear() {
        reactor_clear_error();
    }
    
    /// Get a human-readable description for an error code
    static const char* description(ErrorCode code) {
        return reactor_error_description(static_cast<uint32_t>(code));
    }
    
    /// Check and log any pending error, returns true if there was an error
    static bool check_and_log() {
        if (has_error()) {
            const char* msg = message();
            if (msg) {
                Log::error(msg);
            }
            return true;
        }
        return false;
    }
};

// =============================================================================
// Scene — Global scene management
// =============================================================================

struct Scene {
    /// Get object count in the global scene
    static uint32_t object_count() { return reactor_object_count(); }

    /// Set transform for an object
    static void set_transform(uint32_t index, const Mat4& transform) {
        reactor_set_object_transform(index, transform.to_c());
    }

    /// Get transform for an object
    static Mat4 get_transform(uint32_t index) {
        return Mat4(reactor_get_object_transform(index));
    }

    /// Set visibility for an object
    static void set_visible(uint32_t index, bool visible) {
        reactor_set_object_visible(index, visible);
    }

    /// Clear all objects from the scene
    static void clear() { reactor_clear_scene(); }
};

// =============================================================================
// Lighting — Light management
// =============================================================================

struct Lighting {
    /// Add a directional light
    static int32_t add_directional(const Vec3& direction, const Vec3& color, float intensity) {
        return reactor_add_directional_light(
            direction.x, direction.y, direction.z,
            color.x, color.y, color.z,
            intensity
        );
    }

    /// Add a point light
    static int32_t add_point(const Vec3& position, const Vec3& color, float intensity, float range) {
        return reactor_add_point_light(
            position.x, position.y, position.z,
            color.x, color.y, color.z,
            intensity, range
        );
    }

    /// Add a spot light
    static int32_t add_spot(
        const Vec3& position, const Vec3& direction,
        const Vec3& color, float intensity, float range, float angle_degrees
    ) {
        return reactor_add_spot_light(
            position.x, position.y, position.z,
            direction.x, direction.y, direction.z,
            color.x, color.y, color.z,
            intensity, range, angle_degrees
        );
    }

    /// Get light count
    static uint32_t count() { return reactor_light_count(); }

    /// Clear all lights
    static void clear() { reactor_clear_lights(); }
};

// =============================================================================
// Texture — RAII wrapper for textures
// =============================================================================

class Texture {
    void* handle_ = nullptr;
    uint32_t width_ = 0;
    uint32_t height_ = 0;

public:
    Texture() = default;
    
    /// Load from file (PNG, JPG, BMP, etc.)
    explicit Texture(const std::string& path) {
        handle_ = reactor_load_texture(path.c_str());
        if (handle_) {
            width_ = reactor_texture_width(handle_);
            height_ = reactor_texture_height(handle_);
        }
    }

    /// Load from memory
    Texture(const uint8_t* data, uint32_t len) {
        handle_ = reactor_load_texture_bytes(data, len);
        if (handle_) {
            width_ = reactor_texture_width(handle_);
            height_ = reactor_texture_height(handle_);
        }
    }

    /// Create solid color texture
    static Texture solid(uint8_t r, uint8_t g, uint8_t b, uint8_t a = 255) {
        Texture tex;
        tex.handle_ = reactor_create_solid_texture(r, g, b, a);
        if (tex.handle_) {
            tex.width_ = 1;
            tex.height_ = 1;
        }
        return tex;
    }

    /// Create white texture (default diffuse)
    static Texture white() { return solid(255, 255, 255, 255); }

    /// Create black texture
    static Texture black() { return solid(0, 0, 0, 255); }

    /// Create default normal map (flat surface)
    static Texture default_normal() { return solid(128, 128, 255, 255); }

    ~Texture() {
        if (handle_) {
            reactor_destroy_texture(handle_);
        }
    }

    // Move semantics
    Texture(Texture&& other) noexcept 
        : handle_(other.handle_), width_(other.width_), height_(other.height_) {
        other.handle_ = nullptr;
        other.width_ = 0;
        other.height_ = 0;
    }

    Texture& operator=(Texture&& other) noexcept {
        if (this != &other) {
            if (handle_) reactor_destroy_texture(handle_);
            handle_ = other.handle_;
            width_ = other.width_;
            height_ = other.height_;
            other.handle_ = nullptr;
            other.width_ = 0;
            other.height_ = 0;
        }
        return *this;
    }

    // No copy
    Texture(const Texture&) = delete;
    Texture& operator=(const Texture&) = delete;

    /// Check if texture is valid
    bool valid() const { return handle_ != nullptr; }
    explicit operator bool() const { return valid(); }

    /// Get dimensions
    uint32_t width() const { return width_; }
    uint32_t height() const { return height_; }

    /// Get raw handle (for advanced use)
    void* raw() const { return handle_; }
};

// =============================================================================
// Config — Application configuration
// =============================================================================

/// Renderer mode for the engine
enum class RendererMode {
    Forward = 0,
    Deferred = 1,
    RayTracing = 2,
};

struct Config {
    std::string title = "REACTOR Application";
    uint32_t width = 1280;
    uint32_t height = 720;
    bool vsync = true;
    uint32_t msaa_samples = 4;
    bool fullscreen = false;
    bool resizable = true;
    uint32_t physics_hz = 60;
    RendererMode renderer = RendererMode::Forward;
    std::string scene = "";  // Path to auto-load scene (glTF, etc.)

    Config() = default;
    Config(const std::string& t) : title(t) {}
    Config(const std::string& t, uint32_t w, uint32_t h) : title(t), width(w), height(h) {}

    Config& with_size(uint32_t w, uint32_t h) { width = w; height = h; return *this; }
    Config& with_vsync(bool v) { vsync = v; return *this; }
    Config& with_msaa(uint32_t samples) { msaa_samples = samples; return *this; }
    Config& with_fullscreen(bool f) { fullscreen = f; return *this; }
    Config& with_resizable(bool r) { resizable = r; return *this; }
    Config& with_physics_hz(uint32_t hz) { physics_hz = hz; return *this; }
    Config& with_renderer(RendererMode mode) { renderer = mode; return *this; }
    Config& with_scene(const std::string& path) { scene = path; return *this; }
    
    /// Convert to C API config
    CConfig to_c() const {
        return CConfig{
            title.c_str(),
            width,
            height,
            vsync,
            msaa_samples,
            fullscreen,
            resizable,
            physics_hz,
            static_cast<CRendererMode>(renderer),
            scene.empty() ? nullptr : scene.c_str()
        };
    }
};

// =============================================================================
// Application — Base class for games (THE ONE CALL pattern)
// =============================================================================

class Application {
public:
    virtual ~Application() = default;

    /// Get configuration (override to customize)
    virtual Config config() { return Config(); }

    /// Called once after initialization
    virtual void on_init() {}

    /// Called every frame for game logic
    virtual void on_update(float dt) { (void)dt; }

    /// Called every frame for rendering
    virtual void on_render() {}

    /// Called before exit
    virtual void on_shutdown() {}

    /// Called when window is resized
    virtual void on_resize(uint32_t width, uint32_t height) { (void)width; (void)height; }

    /// Run the application (blocking) — THE ONE CALL
    int run() {
        // Store 'this' for callbacks
        s_instance = this;

        Config cfg = config();

        CConfig c_config{};
        c_config.title = cfg.title.c_str();
        c_config.width = cfg.width;
        c_config.height = cfg.height;
        c_config.vsync = cfg.vsync;
        c_config.msaa_samples = cfg.msaa_samples;
        c_config.fullscreen = cfg.fullscreen;
        c_config.resizable = cfg.resizable;
        c_config.physics_hz = cfg.physics_hz;

        CCallbacks callbacks{};
        callbacks.on_init = &Application::static_on_init;
        callbacks.on_update = &Application::static_on_update;
        callbacks.on_render = &Application::static_on_render;
        callbacks.on_shutdown = &Application::static_on_shutdown;
        callbacks.on_resize = &Application::static_on_resize;

        return reactor_run(c_config, callbacks);
    }

    /// Convenience: run with custom config
    int run(const Config& cfg) {
        s_instance = this;
        s_config = cfg;

        CConfig c_config{};
        c_config.title = cfg.title.c_str();
        c_config.width = cfg.width;
        c_config.height = cfg.height;
        c_config.vsync = cfg.vsync;
        c_config.msaa_samples = cfg.msaa_samples;
        c_config.fullscreen = cfg.fullscreen;
        c_config.resizable = cfg.resizable;
        c_config.physics_hz = cfg.physics_hz;

        CCallbacks callbacks{};
        callbacks.on_init = &Application::static_on_init;
        callbacks.on_update = &Application::static_on_update;
        callbacks.on_render = &Application::static_on_render;
        callbacks.on_shutdown = &Application::static_on_shutdown;
        callbacks.on_resize = &Application::static_on_resize;

        return reactor_run(c_config, callbacks);
    }

    /// Convenience: run with title and size
    int run(const std::string& title, uint32_t width = 1280, uint32_t height = 720) {
        return run(Config(title, width, height));
    }

private:
    static Application* s_instance;
    static Config s_config;

    static void static_on_init() {
        if (s_instance) s_instance->on_init();
    }
    static void static_on_update(float dt) {
        if (s_instance) s_instance->on_update(dt);
    }
    static void static_on_render() {
        if (s_instance) s_instance->on_render();
    }
    static void static_on_shutdown() {
        if (s_instance) s_instance->on_shutdown();
    }
    static void static_on_resize(uint32_t w, uint32_t h) {
        if (s_instance) s_instance->on_resize(w, h);
    }
};

// Static member definitions (must be in header for header-only)
inline Application* Application::s_instance = nullptr;
inline Config Application::s_config;

// =============================================================================
// ReactorApp — Ultra-simple functional API
// =============================================================================

/// Run REACTOR with lambda callbacks — THE SIMPLEST WAY
inline int ReactorApp(
    const Config& config,
    std::function<void()> on_init = nullptr,
    std::function<void(float)> on_update = nullptr,
    std::function<void()> on_render = nullptr
) {
    static std::function<void()> s_init = on_init;
    static std::function<void(float)> s_update = on_update;
    static std::function<void()> s_render = on_render;

    CConfig c_config{};
    c_config.title = config.title.c_str();
    c_config.width = config.width;
    c_config.height = config.height;
    c_config.vsync = config.vsync;
    c_config.msaa_samples = config.msaa_samples;
    c_config.fullscreen = config.fullscreen;
    c_config.resizable = config.resizable;
    c_config.physics_hz = config.physics_hz;

    CCallbacks callbacks{};
    callbacks.on_init = []() { if (s_init) s_init(); };
    callbacks.on_update = [](float dt) { if (s_update) s_update(dt); };
    callbacks.on_render = []() { if (s_render) s_render(); };

    return reactor_run(c_config, callbacks);
}

/// Run REACTOR with just title — THE ONE CALL
inline int ReactorApp(
    const std::string& title,
    std::function<void()> on_init = nullptr,
    std::function<void(float)> on_update = nullptr,
    std::function<void()> on_render = nullptr
) {
    return ReactorApp(Config(title), on_init, on_update, on_render);
}

/// Run REACTOR with title and size
inline int ReactorApp(
    const std::string& title,
    uint32_t width,
    uint32_t height,
    std::function<void()> on_init = nullptr,
    std::function<void(float)> on_update = nullptr,
    std::function<void()> on_render = nullptr
) {
    return ReactorApp(Config(title, width, height), on_init, on_update, on_render);
}

} // namespace reactor
