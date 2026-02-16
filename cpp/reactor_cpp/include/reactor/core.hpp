// =============================================================================
// REACTOR C++ SDK — Core C API Declarations
// =============================================================================
// Auto-generated declarations for the Rust C API.
// This file declares all extern "C" functions from reactor_c_api.dll
// =============================================================================

#pragma once

#include <cstdint>

// =============================================================================
// C API Types (must match Rust repr(C) types)
// =============================================================================

extern "C" {

// Math types
struct CVec2 { float x, y; };
struct CVec3 { float x, y, z; };
struct CVec4 { float x, y, z, w; };
struct CMat4 { float cols[4][4]; };

struct CVertex {
    CVec3 position;
    CVec3 normal;
    CVec2 uv;
    CVec4 color;
};

struct CTransform {
    CVec3 position;
    CVec3 rotation;
    CVec3 scale;
};

struct CLight {
    uint32_t light_type;
    CVec3 position;
    CVec3 direction;
    CVec3 color;
    float intensity;
    float range;
    float inner_angle;
    float outer_angle;
};

// Renderer mode enum
enum CRendererMode {
    Forward = 0,
    Deferred = 1,
    RayTracing = 2,
};

struct CConfig {
    const char* title;
    uint32_t width;
    uint32_t height;
    bool vsync;
    uint32_t msaa_samples;
    bool fullscreen;
    bool resizable;
    uint32_t physics_hz;
    CRendererMode renderer;
    const char* scene;  // Path to auto-load scene (glTF, etc.)
};

// Callback types
typedef void (*InitCallback)();
typedef void (*UpdateCallback)(float);
typedef void (*RenderCallback)();
typedef void (*ShutdownCallback)();
typedef void (*ResizeCallback)(uint32_t, uint32_t);

struct CCallbacks {
    InitCallback on_init;
    UpdateCallback on_update;
    RenderCallback on_render;
    ShutdownCallback on_shutdown;
    ResizeCallback on_resize;
};

// =============================================================================
// Version & Info
// =============================================================================

const char* reactor_version();
const char* reactor_engine_name();
uint32_t reactor_get_version_major();
uint32_t reactor_get_version_minor();
uint32_t reactor_get_version_patch();

// =============================================================================
// Core API — The ONE CALL entry point
// =============================================================================

int32_t reactor_run(CConfig config, CCallbacks callbacks);
int32_t reactor_run_simple(
    const char* title,
    uint32_t width,
    uint32_t height,
    InitCallback on_init,
    UpdateCallback on_update,
    RenderCallback on_render
);

// =============================================================================
// Time & Frame Info
// =============================================================================

float reactor_get_delta_time();
float reactor_get_total_time();
float reactor_get_fps();
uint64_t reactor_get_frame_count();

// =============================================================================
// Window API
// =============================================================================

uint32_t reactor_get_width();
uint32_t reactor_get_height();
float reactor_get_aspect_ratio();
bool reactor_should_close();
void reactor_request_close();

// =============================================================================
// Input API
// =============================================================================

bool reactor_key_down(uint32_t key_code);
bool reactor_key_pressed(uint32_t key_code);
CVec2 reactor_mouse_position();
CVec2 reactor_mouse_delta();
bool reactor_mouse_button(uint32_t button);

// Key codes
uint32_t reactor_key_w();
uint32_t reactor_key_a();
uint32_t reactor_key_s();
uint32_t reactor_key_d();
uint32_t reactor_key_q();
uint32_t reactor_key_e();
uint32_t reactor_key_space();
uint32_t reactor_key_shift();
uint32_t reactor_key_ctrl();
uint32_t reactor_key_escape();
uint32_t reactor_key_enter();
uint32_t reactor_key_tab();
uint32_t reactor_key_up();
uint32_t reactor_key_arrow_down();
uint32_t reactor_key_left();
uint32_t reactor_key_right();

// =============================================================================
// Camera API
// =============================================================================

void reactor_set_camera_position(float x, float y, float z);
void reactor_set_camera_target(float x, float y, float z);
CVec3 reactor_get_camera_position();
CMat4 reactor_get_view_projection();

// =============================================================================
// Math utilities
// =============================================================================

CMat4 reactor_mat4_identity();
CMat4 reactor_mat4_perspective(float fov_degrees, float aspect, float near_plane, float far_plane);
CMat4 reactor_mat4_look_at(CVec3 eye, CVec3 target, CVec3 up);
CMat4 reactor_mat4_mul(CMat4 a, CMat4 b);
CMat4 reactor_mat4_translation(float x, float y, float z);
CMat4 reactor_mat4_rotation_x(float angle_radians);
CMat4 reactor_mat4_rotation_y(float angle_radians);
CMat4 reactor_mat4_rotation_z(float angle_radians);
CMat4 reactor_mat4_scale(float x, float y, float z);
CMat4 reactor_mat4_inverse(CMat4 m);
CMat4 reactor_mat4_transpose(CMat4 m);

CVec3 reactor_vec3_add(CVec3 a, CVec3 b);
CVec3 reactor_vec3_sub(CVec3 a, CVec3 b);
CVec3 reactor_vec3_mul(CVec3 a, float s);
float reactor_vec3_dot(CVec3 a, CVec3 b);
CVec3 reactor_vec3_cross(CVec3 a, CVec3 b);
float reactor_vec3_length(CVec3 v);
CVec3 reactor_vec3_normalize(CVec3 v);
CVec3 reactor_vec3_lerp(CVec3 a, CVec3 b, float t);

// =============================================================================
// SDF (ADead-GPU)
// =============================================================================

float reactor_sdf_sphere(float px, float py, float pz, float radius);
float reactor_sdf_box(float px, float py, float pz, float bx, float by, float bz);
float reactor_sdf_cylinder(float px, float py, float pz, float h, float r);
float reactor_sdf_torus(float px, float py, float pz, float r1, float r2);
float reactor_sdf_capsule(float px, float py, float pz, float h, float r);
float reactor_sdf_union(float d1, float d2);
float reactor_sdf_subtract(float d1, float d2);
float reactor_sdf_intersect(float d1, float d2);
float reactor_sdf_smooth_union(float d1, float d2, float k);

// =============================================================================
// Utility functions
// =============================================================================

float reactor_lerp(float a, float b, float t);
float reactor_clamp(float v, float min, float max);
float reactor_smoothstep(float edge0, float edge1, float x);
float reactor_deg_to_rad(float degrees);
float reactor_rad_to_deg(float radians);

// =============================================================================
// Debug logging
// =============================================================================

void reactor_log_info(const char* msg);
void reactor_log_warn(const char* msg);
void reactor_log_error(const char* msg);

// =============================================================================
// Error Handling API
// =============================================================================

uint32_t reactor_get_last_error();
const char* reactor_get_error_message();
bool reactor_has_error();
void reactor_clear_error();
const char* reactor_error_description(uint32_t code);

// =============================================================================
// Scene API — Global scene management
// =============================================================================

uint32_t reactor_object_count();
int32_t reactor_add_object(void* mesh, void* material, CMat4 transform);
void reactor_set_object_transform(uint32_t index, CMat4 transform);
CMat4 reactor_get_object_transform(uint32_t index);
void reactor_set_object_visible(uint32_t index, bool visible);
void reactor_clear_scene();

// =============================================================================
// Scene Handle API — For custom scenes
// =============================================================================

void* reactor_scene_create();
void reactor_scene_destroy(void* scene);
uint32_t reactor_scene_object_count(const void* scene);
void reactor_scene_clear(void* scene);
int32_t reactor_scene_add_object(void* scene, void* mesh, void* material, CMat4 transform);
void reactor_scene_set_transform(void* scene, uint32_t index, CMat4 transform);
CMat4 reactor_scene_get_transform(const void* scene, uint32_t index);
void reactor_scene_set_visible(void* scene, uint32_t index, bool visible);
bool reactor_scene_is_visible(const void* scene, uint32_t index);
bool reactor_scene_remove_object(void* scene, uint32_t index);

// =============================================================================
// Mesh API
// =============================================================================

void* reactor_create_mesh(const CVertex* vertices, uint32_t vertex_count, const uint32_t* indices, uint32_t index_count);
void* reactor_create_cube();
void reactor_destroy_mesh(void* mesh);

// =============================================================================
// Material API
// =============================================================================

void reactor_destroy_material(void* material);

// =============================================================================
// Texture API
// =============================================================================

void* reactor_load_texture(const char* path);
void* reactor_load_texture_bytes(const uint8_t* data, uint32_t len);
void* reactor_create_solid_texture(uint8_t r, uint8_t g, uint8_t b, uint8_t a);
uint32_t reactor_texture_width(const void* texture);
uint32_t reactor_texture_height(const void* texture);
void reactor_destroy_texture(void* texture);

// =============================================================================
// Lighting API
// =============================================================================

int32_t reactor_add_directional_light(float dir_x, float dir_y, float dir_z, float r, float g, float b, float intensity);
int32_t reactor_add_point_light(float x, float y, float z, float r, float g, float b, float intensity, float range);
int32_t reactor_add_spot_light(float pos_x, float pos_y, float pos_z, float dir_x, float dir_y, float dir_z, float r, float g, float b, float intensity, float range, float angle_degrees);
uint32_t reactor_light_count();
void reactor_clear_lights();

// =============================================================================
// Camera Handle API — For custom cameras
// =============================================================================

void* reactor_camera_create_perspective(float fov, float aspect, float near_plane, float far_plane);
void reactor_camera_destroy(void* camera);
void reactor_camera_set_position(void* camera, float x, float y, float z);
void reactor_camera_set_target(void* camera, float x, float y, float z);
CMat4 reactor_camera_get_view_projection(const void* camera);
CMat4 reactor_camera_get_view(const void* camera);
CMat4 reactor_camera_get_projection(const void* camera);

} // extern "C"
