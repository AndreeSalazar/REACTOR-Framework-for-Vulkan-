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
#include <vector>
#include <cmath>
#include <algorithm>

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

// (GPU, Lighting, Mesh, Material, Scene structs defined below with full implementations)

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
        CMat4 c = transform;
        reactor_set_object_transform(index, c);
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
// Mesh — RAII wrapper for GPU meshes
// =============================================================================

class Mesh {
private:
    MeshHandle* handle_ = nullptr;
    uint32_t vertex_count_ = 0;
    uint32_t index_count_ = 0;

public:
    Mesh() = default;
    explicit Mesh(MeshHandle* handle) : handle_(handle) {}

    /// Create a cube mesh (built-in primitive)
    static Mesh cube() {
        Mesh m;
        m.handle_ = reactor_create_cube();
        m.vertex_count_ = 24;
        m.index_count_ = 36;
        return m;
    }

    /// Create mesh from vertex and index data
    static Mesh from_data(const std::vector<CVertex>& vertices, const std::vector<uint32_t>& indices) {
        Mesh m;
        m.handle_ = reactor_create_mesh(
            vertices.data(), static_cast<uint32_t>(vertices.size()),
            indices.data(), static_cast<uint32_t>(indices.size())
        );
        m.vertex_count_ = static_cast<uint32_t>(vertices.size());
        m.index_count_ = static_cast<uint32_t>(indices.size());
        return m;
    }

    /// Create a simple quad mesh
    static Mesh quad(float size = 1.0f) {
        float h = size * 0.5f;
        std::vector<CVertex> vertices = {
            {{-h, 0, -h}, {0, 1, 0}, {0, 0}},
            {{ h, 0, -h}, {0, 1, 0}, {1, 0}},
            {{ h, 0,  h}, {0, 1, 0}, {1, 1}},
            {{-h, 0,  h}, {0, 1, 0}, {0, 1}},
        };
        std::vector<uint32_t> indices = {0, 1, 2, 2, 3, 0};
        return from_data(vertices, indices);
    }

    /// Create a plane mesh with subdivisions
    static Mesh plane(float width, float depth, uint32_t subdivisions = 1) {
        std::vector<CVertex> vertices;
        std::vector<uint32_t> indices;
        
        float hw = width * 0.5f;
        float hd = depth * 0.5f;
        uint32_t segs = subdivisions + 1;
        
        for (uint32_t z = 0; z <= segs; ++z) {
            for (uint32_t x = 0; x <= segs; ++x) {
                float px = -hw + (width * x / segs);
                float pz = -hd + (depth * z / segs);
                float u = static_cast<float>(x) / segs;
                float v = static_cast<float>(z) / segs;
                vertices.push_back({{px, 0, pz}, {0, 1, 0}, {u, v}});
            }
        }
        
        for (uint32_t z = 0; z < segs; ++z) {
            for (uint32_t x = 0; x < segs; ++x) {
                uint32_t i = z * (segs + 1) + x;
                indices.push_back(i);
                indices.push_back(i + segs + 1);
                indices.push_back(i + 1);
                indices.push_back(i + 1);
                indices.push_back(i + segs + 1);
                indices.push_back(i + segs + 2);
            }
        }
        
        return from_data(vertices, indices);
    }

    ~Mesh() {
        if (handle_) {
            reactor_destroy_mesh(handle_);
        }
    }

    // Move semantics
    Mesh(Mesh&& other) noexcept 
        : handle_(other.handle_), vertex_count_(other.vertex_count_), index_count_(other.index_count_) {
        other.handle_ = nullptr;
        other.vertex_count_ = 0;
        other.index_count_ = 0;
    }

    Mesh& operator=(Mesh&& other) noexcept {
        if (this != &other) {
            if (handle_) reactor_destroy_mesh(handle_);
            handle_ = other.handle_;
            vertex_count_ = other.vertex_count_;
            index_count_ = other.index_count_;
            other.handle_ = nullptr;
            other.vertex_count_ = 0;
            other.index_count_ = 0;
        }
        return *this;
    }

    // No copy
    Mesh(const Mesh&) = delete;
    Mesh& operator=(const Mesh&) = delete;

    /// Check if mesh is valid
    bool valid() const { return handle_ != nullptr; }
    explicit operator bool() const { return valid(); }

    /// Get vertex/index counts
    uint32_t vertex_count() const { return vertex_count_; }
    uint32_t index_count() const { return index_count_; }

    /// Get raw handle
    MeshHandle* raw() const { return handle_; }
};

// =============================================================================
// Texture — RAII wrapper for textures
// =============================================================================

class Texture {
    TextureHandle* handle_ = nullptr;
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
    TextureHandle* raw() const { return handle_; }
};

// =============================================================================
// Material — RAII wrapper for GPU materials
// =============================================================================

class Material {
private:
    MaterialHandle* handle_ = nullptr;

public:
    Material() = default;
    explicit Material(MaterialHandle* handle) : handle_(handle) {}

    /// Create a basic material from SPIR-V shader code
    static Material from_shaders(const std::vector<uint32_t>& vert_spv, const std::vector<uint32_t>& frag_spv) {
        MaterialHandle* handle = reactor_create_material(
            vert_spv.data(), static_cast<uint32_t>(vert_spv.size()),
            frag_spv.data(), static_cast<uint32_t>(frag_spv.size())
        );
        return Material(handle);
    }

    /// Create a textured material from SPIR-V shader code and texture
    static Material from_texture(const std::vector<uint32_t>& vert_spv, const std::vector<uint32_t>& frag_spv, const Texture& texture) {
        MaterialHandle* handle = reactor_create_textured_material(
            vert_spv.data(), static_cast<uint32_t>(vert_spv.size()),
            frag_spv.data(), static_cast<uint32_t>(frag_spv.size()),
            texture.raw()
        );
        return Material(handle);
    }

    ~Material() {
        if (handle_) {
            reactor_destroy_material(handle_);
        }
    }

    // Move semantics
    Material(Material&& other) noexcept : handle_(other.handle_) {
        other.handle_ = nullptr;
    }

    Material& operator=(Material&& other) noexcept {
        if (this != &other) {
            if (handle_) reactor_destroy_material(handle_);
            handle_ = other.handle_;
            other.handle_ = nullptr;
        }
        return *this;
    }

    // No copy
    Material(const Material&) = delete;
    Material& operator=(const Material&) = delete;

    /// Check if material is valid
    bool valid() const { return handle_ != nullptr; }
    explicit operator bool() const { return valid(); }

    /// Get raw handle (for advanced use)
    MaterialHandle* raw() const { return handle_; }
};

// =============================================================================
// Model — OBJ model loading and info
// =============================================================================

struct ObjInfo {
    uint32_t vertex_count = 0;
    uint32_t index_count = 0;
    uint32_t triangle_count = 0;
    bool valid = false;

    /// Load OBJ file info (does not create GPU resources)
    static ObjInfo load(const std::string& path) {
        CObjData data = reactor_load_obj_info(path.c_str());
        ObjInfo info;
        info.vertex_count = data.vertex_count;
        info.index_count = data.index_count;
        info.triangle_count = data.triangle_count;
        info.valid = data.success;
        return info;
    }

    explicit operator bool() const { return valid; }
};

// =============================================================================
// GameObject — Represents an object in the scene
// =============================================================================

class GameObject {
private:
    uint32_t index_ = UINT32_MAX;
    bool valid_ = false;

public:
    GameObject() = default;
    explicit GameObject(uint32_t index) : index_(index), valid_(true) {}

    /// Check if valid
    bool valid() const { return valid_ && index_ != UINT32_MAX; }
    explicit operator bool() const { return valid(); }

    /// Get scene index
    uint32_t index() const { return index_; }

    /// Set transform
    void set_transform(const Mat4& transform) {
        if (valid()) { CMat4 c = transform; reactor_set_object_transform(index_, c); }
    }

    /// Get transform
    Mat4 transform() const {
        if (valid()) return Mat4(reactor_get_object_transform(index_));
        return Mat4::Identity();
    }

    /// Set position (convenience)
    void set_position(const Vec3& pos) {
        Mat4 t = transform();
        t.cols[3][0] = pos.x;
        t.cols[3][1] = pos.y;
        t.cols[3][2] = pos.z;
        set_transform(t);
    }

    /// Get position
    Vec3 position() const {
        Mat4 t = transform();
        return Vec3(t.cols[3][0], t.cols[3][1], t.cols[3][2]);
    }

    /// Set visibility
    void set_visible(bool visible) {
        if (valid()) reactor_set_object_visible(index_, visible);
    }

    /// Translate
    void translate(const Vec3& delta) {
        set_position(position() + delta);
    }

    /// Set rotation (Euler angles in radians)
    void set_rotation(float pitch, float yaw, float roll) {
        Vec3 pos = position();
        // Build rotation matrix
        float cp = std::cos(pitch), sp = std::sin(pitch);
        float cy = std::cos(yaw), sy = std::sin(yaw);
        float cr = std::cos(roll), sr = std::sin(roll);
        
        Mat4 rot = Mat4::Identity();
        rot.cols[0][0] = cy * cr;
        rot.cols[0][1] = cy * sr;
        rot.cols[0][2] = -sy;
        rot.cols[1][0] = sp * sy * cr - cp * sr;
        rot.cols[1][1] = sp * sy * sr + cp * cr;
        rot.cols[1][2] = sp * cy;
        rot.cols[2][0] = cp * sy * cr + sp * sr;
        rot.cols[2][1] = cp * sy * sr - sp * cr;
        rot.cols[2][2] = cp * cy;
        rot.cols[3][0] = pos.x;
        rot.cols[3][1] = pos.y;
        rot.cols[3][2] = pos.z;
        set_transform(rot);
    }

    /// Set scale (uniform)
    void set_scale(float scale) {
        set_scale(Vec3(scale, scale, scale));
    }

    /// Set scale (non-uniform)
    void set_scale(const Vec3& scale) {
        Mat4 t = transform();
        // Extract position, apply scale to rotation part
        Vec3 pos(t.cols[3][0], t.cols[3][1], t.cols[3][2]);
        t.cols[0][0] *= scale.x; t.cols[0][1] *= scale.x; t.cols[0][2] *= scale.x;
        t.cols[1][0] *= scale.y; t.cols[1][1] *= scale.y; t.cols[1][2] *= scale.y;
        t.cols[2][0] *= scale.z; t.cols[2][1] *= scale.z; t.cols[2][2] *= scale.z;
        set_transform(t);
    }
};

// =============================================================================
// Debug — Debug drawing utilities
// =============================================================================

struct Debug {
    /// Draw a line (for one frame)
    static void line(const Vec3& start, const Vec3& end, const Vec3& color = Vec3(1, 1, 1)) {
        // TODO: Implement when C ABI is ready
        (void)start; (void)end; (void)color;
    }

    /// Draw a wire box
    static void wire_box(const Vec3& center, const Vec3& size, const Vec3& color = Vec3(1, 1, 1)) {
        Vec3 h = size * 0.5f;
        Vec3 corners[8] = {
            center + Vec3(-h.x, -h.y, -h.z),
            center + Vec3( h.x, -h.y, -h.z),
            center + Vec3( h.x, -h.y,  h.z),
            center + Vec3(-h.x, -h.y,  h.z),
            center + Vec3(-h.x,  h.y, -h.z),
            center + Vec3( h.x,  h.y, -h.z),
            center + Vec3( h.x,  h.y,  h.z),
            center + Vec3(-h.x,  h.y,  h.z),
        };
        // Bottom
        line(corners[0], corners[1], color);
        line(corners[1], corners[2], color);
        line(corners[2], corners[3], color);
        line(corners[3], corners[0], color);
        // Top
        line(corners[4], corners[5], color);
        line(corners[5], corners[6], color);
        line(corners[6], corners[7], color);
        line(corners[7], corners[4], color);
        // Verticals
        line(corners[0], corners[4], color);
        line(corners[1], corners[5], color);
        line(corners[2], corners[6], color);
        line(corners[3], corners[7], color);
    }

    /// Draw a wire sphere (approximation)
    static void wire_sphere(const Vec3& center, float radius, const Vec3& color = Vec3(1, 1, 1)) {
        const int segments = 16;
        for (int i = 0; i < segments; ++i) {
            float a1 = (float)i / segments * 6.28318f;
            float a2 = (float)(i + 1) / segments * 6.28318f;
            // XY circle
            line(center + Vec3(std::cos(a1), std::sin(a1), 0) * radius,
                 center + Vec3(std::cos(a2), std::sin(a2), 0) * radius, color);
            // XZ circle
            line(center + Vec3(std::cos(a1), 0, std::sin(a1)) * radius,
                 center + Vec3(std::cos(a2), 0, std::sin(a2)) * radius, color);
            // YZ circle
            line(center + Vec3(0, std::cos(a1), std::sin(a1)) * radius,
                 center + Vec3(0, std::cos(a2), std::sin(a2)) * radius, color);
        }
    }

    /// Draw a grid on the XZ plane
    static void grid(float size, int divisions, const Vec3& color = Vec3(0.5f, 0.5f, 0.5f)) {
        float half = size * 0.5f;
        float step = size / divisions;
        for (int i = 0; i <= divisions; ++i) {
            float pos = -half + i * step;
            line(Vec3(pos, 0, -half), Vec3(pos, 0, half), color);
            line(Vec3(-half, 0, pos), Vec3(half, 0, pos), color);
        }
    }

    /// Draw coordinate axes
    static void axes(const Vec3& origin, float length = 1.0f) {
        line(origin, origin + Vec3(length, 0, 0), Vec3(1, 0, 0)); // X = Red
        line(origin, origin + Vec3(0, length, 0), Vec3(0, 1, 0)); // Y = Green
        line(origin, origin + Vec3(0, 0, length), Vec3(0, 0, 1)); // Z = Blue
    }

    /// Draw a ray
    static void ray(const Vec3& origin, const Vec3& direction, float length = 10.0f, const Vec3& color = Vec3(1, 1, 0)) {
        line(origin, origin + direction.normalized() * length, color);
    }
};

// =============================================================================
// CharacterController — FPS-style physics controller
// =============================================================================

class CharacterController {
private:
    CCharacterController data_;

public:
    CharacterController() : data_(reactor_character_controller_create(0, 1, 0)) {}
    CharacterController(const Vec3& position) 
        : data_(reactor_character_controller_create(position.x, position.y, position.z)) {}

    /// Update physics (call every frame)
    void update(float dt, const Vec3& move_input, bool jump, float ground_y = 0.0f) {
        reactor_character_controller_update(&data_, dt, move_input.x, move_input.z, jump, ground_y);
    }

    /// Get eye position (for camera)
    Vec3 eye_position() const {
        float x, y, z;
        reactor_character_controller_eye_position(&data_, &x, &y, &z);
        return Vec3(x, y, z);
    }

    /// Get/set position
    Vec3 position() const { return Vec3(data_.position_x, data_.position_y, data_.position_z); }
    void set_position(const Vec3& pos) { data_.position_x = pos.x; data_.position_y = pos.y; data_.position_z = pos.z; }

    /// Get/set velocity
    Vec3 velocity() const { return Vec3(data_.velocity_x, data_.velocity_y, data_.velocity_z); }
    void set_velocity(const Vec3& vel) { data_.velocity_x = vel.x; data_.velocity_y = vel.y; data_.velocity_z = vel.z; }

    /// Properties
    float height() const { return data_.height; }
    void set_height(float h) { data_.height = h; }

    float radius() const { return data_.radius; }
    void set_radius(float r) { data_.radius = r; }

    float move_speed() const { return data_.move_speed; }
    void set_move_speed(float s) { data_.move_speed = s; }

    float jump_force() const { return data_.jump_force; }
    void set_jump_force(float f) { data_.jump_force = f; }

    float gravity() const { return data_.gravity; }
    void set_gravity(float g) { data_.gravity = g; }

    bool is_grounded() const { return data_.is_grounded; }
};

// =============================================================================
// Physics — Static physics utilities
// =============================================================================

struct Physics {
    /// Raycast against AABB, returns hit distance or -1 if no hit
    static float raycast_aabb(const Vec3& origin, const Vec3& direction, const Vec3& aabb_min, const Vec3& aabb_max) {
        float t;
        bool hit = reactor_raycast_aabb(
            origin.x, origin.y, origin.z,
            direction.x, direction.y, direction.z,
            aabb_min.x, aabb_min.y, aabb_min.z,
            aabb_max.x, aabb_max.y, aabb_max.z,
            &t
        );
        return hit ? t : -1.0f;
    }

    /// Test AABB-AABB intersection
    static bool aabb_intersects(const Vec3& a_min, const Vec3& a_max, const Vec3& b_min, const Vec3& b_max) {
        return reactor_aabb_intersects(
            a_min.x, a_min.y, a_min.z, a_max.x, a_max.y, a_max.z,
            b_min.x, b_min.y, b_min.z, b_max.x, b_max.y, b_max.z
        );
    }

    /// Test sphere-sphere intersection
    static bool sphere_intersects(const Vec3& a_center, float a_radius, const Vec3& b_center, float b_radius) {
        float dist_sq = (b_center - a_center).length_squared();
        float radius_sum = a_radius + b_radius;
        return dist_sq <= radius_sum * radius_sum;
    }

    /// Test point inside AABB
    static bool point_in_aabb(const Vec3& point, const Vec3& aabb_min, const Vec3& aabb_max) {
        return point.x >= aabb_min.x && point.x <= aabb_max.x &&
               point.y >= aabb_min.y && point.y <= aabb_max.y &&
               point.z >= aabb_min.z && point.z <= aabb_max.z;
    }

    /// Test point inside sphere
    static bool point_in_sphere(const Vec3& point, const Vec3& center, float radius) {
        return (point - center).length_squared() <= radius * radius;
    }

    /// Linear interpolation
    static float lerp(float a, float b, float t) { return a + (b - a) * t; }
    static Vec3 lerp(const Vec3& a, const Vec3& b, float t) { return a + (b - a) * t; }

    /// Smoothstep interpolation
    static float smoothstep(float edge0, float edge1, float x) {
        float t = std::clamp((x - edge0) / (edge1 - edge0), 0.0f, 1.0f);
        return t * t * (3.0f - 2.0f * t);
    }
};

// (GPUInfo defined below with real C ABI calls)

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

// =============================================================================
// ECS — Entity Component System (Full CRUD)
// =============================================================================

using EntityId = uint32_t;
constexpr EntityId INVALID_ENTITY = 0;

/// Component mask bits for queries
enum ComponentMask : uint32_t {
    COMPONENT_ALL           = 0,
    COMPONENT_MESH_RENDERER = 1,
    COMPONENT_LIGHT         = 2,
    COMPONENT_CAMERA        = 4,
    COMPONENT_RIGIDBODY     = 8,
};

class Entity {
    EntityId id_ = INVALID_ENTITY;
public:
    Entity() = default;
    explicit Entity(EntityId id) : id_(id) {}

    /// Create a new entity
    static Entity create(const std::string& name = "Entity") {
        return Entity(reactor_entity_create(name.c_str()));
    }

    /// Destroy this entity
    bool destroy() {
        if (id_ == INVALID_ENTITY) return false;
        bool ok = reactor_entity_destroy(id_);
        id_ = INVALID_ENTITY;
        return ok;
    }

    bool valid() const { return id_ != INVALID_ENTITY && reactor_entity_exists(id_); }
    explicit operator bool() const { return valid(); }
    EntityId id() const { return id_; }

    // --- Active state ---
    void set_active(bool active) { reactor_entity_set_active(id_, active); }
    bool is_active() const { return reactor_entity_is_active(id_); }

    // --- Transform ---
    void set_position(const Vec3& p) { reactor_entity_set_position(id_, p.x, p.y, p.z); }
    Vec3 position() const { return Vec3(reactor_entity_get_position(id_)); }
    void set_rotation(const Vec3& r) { reactor_entity_set_rotation(id_, r.x, r.y, r.z); }
    void set_scale(const Vec3& s) { reactor_entity_set_scale(id_, s.x, s.y, s.z); }

    void set_transform(const Vec3& pos, const Vec3& rot, const Vec3& scl) {
        CTransform t;
        t.position = {pos.x, pos.y, pos.z};
        t.rotation = {rot.x, rot.y, rot.z};
        t.scale = {scl.x, scl.y, scl.z};
        reactor_entity_set_transform(id_, t);
    }

    // --- Mesh Renderer ---
    bool add_mesh_renderer(int32_t mesh_idx, int32_t mat_idx) {
        return reactor_entity_add_mesh_renderer(id_, mesh_idx, mat_idx);
    }
    bool remove_mesh_renderer() { return reactor_entity_remove_mesh_renderer(id_); }
    bool has_mesh_renderer() const { return reactor_entity_has_mesh_renderer(id_); }

    // --- Light ---
    bool add_light(const CLight& light) { return reactor_entity_add_light(id_, light); }
    bool remove_light() { return reactor_entity_remove_light(id_); }
    bool has_light() const { return reactor_entity_has_light(id_); }
    void set_light(const CLight& l) { reactor_entity_set_light(id_, l); }

    // --- Camera ---
    bool add_camera(float fov = 60.f, float near_p = 0.1f, float far_p = 1000.f, bool is_main = false) {
        return reactor_entity_add_camera(id_, fov, near_p, far_p, is_main);
    }
    bool remove_camera() { return reactor_entity_remove_camera(id_); }
    bool has_camera() const { return reactor_entity_has_camera(id_); }

    // --- RigidBody ---
    bool add_rigidbody(float mass = 1.f, bool gravity = true) {
        return reactor_entity_add_rigidbody(id_, mass, gravity);
    }
    bool remove_rigidbody() { return reactor_entity_remove_rigidbody(id_); }
    void apply_force(const Vec3& f) { reactor_entity_apply_force(id_, f.x, f.y, f.z); }
    void set_velocity(const Vec3& v) { reactor_entity_set_velocity(id_, v.x, v.y, v.z); }
    Vec3 velocity() const { return Vec3(reactor_entity_get_velocity(id_)); }
};

struct ECS {
    static EntityId create_entity() { return reactor_ecs_create_entity(); }
    static void destroy_entity(EntityId e) { reactor_ecs_destroy_entity(e); }
    static uint32_t entity_count() { return reactor_ecs_entity_count(); }

    /// Query entities by component mask. Returns vector of entity IDs.
    static std::vector<EntityId> query(uint32_t mask, uint32_t max_results = 256) {
        std::vector<EntityId> buf(max_results);
        uint32_t count = reactor_query_entities(mask, buf.data(), max_results);
        buf.resize(count);
        return buf;
    }
};

// =============================================================================
// PBRMaterial — PBR material system with instances
// =============================================================================

class PBRMaterial {
    uint32_t id_ = 0;
public:
    PBRMaterial() = default;
    explicit PBRMaterial(uint32_t id) : id_(id) {}

    /// Create with default parameters
    static PBRMaterial create() { return PBRMaterial(reactor_pbr_create_default()); }

    /// Create with custom parameters
    static PBRMaterial create(const Vec4& base_color, float metallic, float roughness) {
        CPBRMaterial p{};
        p.base_color = {base_color.x, base_color.y, base_color.z, base_color.w};
        p.metallic = metallic;
        p.roughness = roughness;
        p.ao = 1.0f;
        p.emissive = {0, 0, 0};
        p.emissive_strength = 0;
        p.alpha_cutoff = 0.5f;
        p.normal_scale = 1.0f;
        p.double_sided = false;
        p.alpha_mode = 0;
        return PBRMaterial(reactor_pbr_create(p));
    }

    /// Create an instance that inherits from this material
    PBRMaterial create_instance() const { return PBRMaterial(reactor_pbr_create_instance(id_)); }

    void destroy() { if (id_) { reactor_pbr_destroy(id_); id_ = 0; } }

    bool valid() const { return id_ != 0; }
    uint32_t id() const { return id_; }

    void set_base_color(const Vec4& c) { reactor_pbr_set_base_color(id_, c.x, c.y, c.z, c.w); }
    void set_metallic_roughness(float m, float r) { reactor_pbr_set_metallic_roughness(id_, m, r); }
    void set_emissive(const Vec3& c, float strength) { reactor_pbr_set_emissive(id_, c.x, c.y, c.z, strength); }

    static uint32_t count() { return reactor_pbr_count(); }
};

// =============================================================================
// FrameGraph — Render graph configuration
// =============================================================================

/// Resource types for FrameGraph
enum class FGResourceType : uint32_t {
    Texture = 0, Buffer = 1, DepthBuffer = 2, RenderTarget = 3, Swapchain = 4,
};

/// Resource formats for FrameGraph
enum class FGFormat : uint32_t {
    RGBA8 = 0, RGBA16F = 1, RGBA32F = 2, R8 = 3, R16F = 4, R32F = 5,
    Depth32F = 6, Depth24Stencil8 = 7,
};

class FrameGraph {
    CFrameGraphHandle* handle_ = nullptr;
public:
    FrameGraph() { handle_ = reactor_frame_graph_create(); }
    ~FrameGraph() { if (handle_) reactor_frame_graph_destroy(handle_); }

    /// Create pre-built forward rendering graph
    static FrameGraph forward(uint32_t w, uint32_t h) {
        FrameGraph fg;
        if (fg.handle_) reactor_frame_graph_destroy(fg.handle_);
        fg.handle_ = reactor_frame_graph_create_forward(w, h);
        return fg;
    }

    /// Create pre-built deferred rendering graph
    static FrameGraph deferred(uint32_t w, uint32_t h) {
        FrameGraph fg;
        if (fg.handle_) reactor_frame_graph_destroy(fg.handle_);
        fg.handle_ = reactor_frame_graph_create_deferred(w, h);
        return fg;
    }

    uint32_t create_resource(const std::string& name, FGResourceType type,
                             uint32_t w, uint32_t h, FGFormat fmt, bool persistent = false) {
        return reactor_frame_graph_create_resource(handle_, name.c_str(),
            static_cast<uint32_t>(type), w, h, static_cast<uint32_t>(fmt), persistent);
    }

    uint32_t add_pass(const std::string& name,
                      const std::vector<uint32_t>& reads,
                      const std::vector<uint32_t>& writes,
                      int32_t order = 0) {
        return reactor_frame_graph_add_pass(handle_, name.c_str(),
            reads.data(), static_cast<uint32_t>(reads.size()),
            writes.data(), static_cast<uint32_t>(writes.size()), order);
    }

    bool compile() { return reactor_frame_graph_compile(handle_); }

    CFrameGraphStats stats() const { return reactor_frame_graph_get_stats(handle_); }

    // Move only
    FrameGraph(FrameGraph&& o) noexcept : handle_(o.handle_) { o.handle_ = nullptr; }
    FrameGraph& operator=(FrameGraph&& o) noexcept {
        if (this != &o) { if (handle_) reactor_frame_graph_destroy(handle_); handle_ = o.handle_; o.handle_ = nullptr; }
        return *this;
    }
    FrameGraph(const FrameGraph&) = delete;
    FrameGraph& operator=(const FrameGraph&) = delete;
};

// =============================================================================
// RenderStats — Real-time rendering statistics
// =============================================================================

struct RenderStats {
    static CRenderStats get() { return reactor_get_render_stats(); }
    static CMemoryBudget memory_budget() { return reactor_get_memory_budget(); }

    /// Print stats to stdout
    static void print() {
        auto s = get();
        printf("FPS: %.1f | Frame: %.2fms | Draw: %u | Tris: %u | Objects: %u/%u | VRAM: %uMB\n",
            s.fps, s.frame_time_ms, s.draw_calls, s.triangles,
            s.visible_objects, s.scene_objects, s.vram_total_mb);
    }
};

// =============================================================================
// PlayMode — Runtime-Editor bridge
// =============================================================================

struct PlayMode {
    static bool enter() { return reactor_play_enter(); }
    static void exit() { reactor_play_exit(); }
    static void pause(bool p) { reactor_play_pause(p); }
    static bool is_playing() { return reactor_play_is_playing(); }
    static bool is_paused() { return reactor_play_is_paused(); }
    static float time() { return reactor_play_get_time(); }
    static void update(float dt) { reactor_play_update(dt); }
};

// =============================================================================
// SceneSerializer — Scene import/export
// =============================================================================

struct SceneSerializer {
    /// Serialize current scene to string
    static std::string serialize() {
        uint32_t size = reactor_scene_serialize_size();
        if (size == 0) return "";
        std::vector<uint8_t> buf(size + 1, 0);
        reactor_scene_serialize(buf.data(), size);
        return std::string(reinterpret_cast<char*>(buf.data()), size);
    }
};

// =============================================================================
// Animation — Animation system wrapper
// =============================================================================

using AnimationClip = uint32_t;

struct Animation {
    static AnimationClip create_clip(const std::string& name) {
        return reactor_animation_create_clip(name.c_str());
    }
    
    static void add_position_keyframe(AnimationClip clip, float time, const Vec3& pos) {
        reactor_animation_add_position_keyframe(clip, time, pos.x, pos.y, pos.z);
    }
    
    static void add_rotation_keyframe(AnimationClip clip, float time, float x, float y, float z, float w) {
        reactor_animation_add_rotation_keyframe(clip, time, x, y, z, w);
    }
    
    static void play(AnimationClip clip, bool looping = false) {
        reactor_animation_play(clip, looping);
    }
    
    static void stop(AnimationClip clip) {
        reactor_animation_stop(clip);
    }
    
    static void update(float dt) {
        reactor_animation_update(dt);
    }
};

// =============================================================================
// Audio — Audio system wrapper
// =============================================================================

using AudioClip = uint32_t;
using AudioSource = uint32_t;

struct Audio {
    static AudioClip load(const std::string& path) {
        return reactor_audio_load(path.c_str());
    }
    
    static AudioSource create_source() {
        return reactor_audio_create_source();
    }
    
    static void play(AudioSource source, AudioClip clip) {
        reactor_audio_play(source, clip);
    }
    
    static void stop(AudioSource source) {
        reactor_audio_stop(source);
    }
    
    static void set_volume(AudioSource source, float volume) {
        reactor_audio_set_volume(source, volume);
    }
    
    static void set_position(AudioSource source, const Vec3& pos) {
        reactor_audio_set_position(source, pos.x, pos.y, pos.z);
    }
    
    static void set_master_volume(float volume) {
        reactor_audio_set_master_volume(volume);
    }
};

// =============================================================================
// PostProcess — Post-processing effects
// =============================================================================

struct PostProcess {
    static void set_bloom(bool enabled, float intensity = 1.0f, float threshold = 1.0f) {
        reactor_postprocess_set_bloom(enabled, intensity, threshold);
    }
    
    static void set_tonemapping(bool enabled, float exposure = 1.0f) {
        reactor_postprocess_set_tonemapping(enabled, exposure);
    }
    
    static void set_vignette(bool enabled, float intensity = 0.5f) {
        reactor_postprocess_set_vignette(enabled, intensity);
    }
    
    static void set_fxaa(bool enabled) {
        reactor_postprocess_set_fxaa(enabled);
    }
};

// =============================================================================
// GPUInfo — GPU information
// =============================================================================

struct GPUInfo {
    static const char* name() { return reactor_get_gpu_name(); }
    static uint32_t vram_mb() { return reactor_get_vram_mb(); }
    static uint32_t msaa_samples() { return reactor_get_msaa_samples(); }
    static bool raytracing_supported() { return reactor_is_raytracing_supported(); }
    
    static void vulkan_version(uint32_t& major, uint32_t& minor, uint32_t& patch) {
        reactor_get_vulkan_version(&major, &minor, &patch);
    }
};

} // namespace reactor
