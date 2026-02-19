// =============================================================================
// REACTOR C++ SDK — Core C API Declarations
// =============================================================================
// Stable ABI contract for the Rust C API (reactor_c_api.dll).
// All types are repr(C) compatible. All handles are opaque pointers.
//
// Ownership Rule:
//   Rust creates → Rust destroys.
//   C++ uses handles but NEVER calls delete/free on them.
//   Use reactor_destroy_*() functions to release resources.
//
// Lifecycle:
//   reactor_initialize()
//   reactor_run() or manual loop with begin_frame/end_frame
//   reactor_shutdown()
// =============================================================================

#pragma once

#include <cstdint>

extern "C" {

// =============================================================================
// ReactorResult — ABI-safe error codes (no exceptions across FFI)
// =============================================================================

enum ReactorResult : int32_t {
    REACTOR_OK                       = 0,
    REACTOR_ERROR_NOT_INITIALIZED    = 1,
    REACTOR_ERROR_ALREADY_INITIALIZED = 2,
    REACTOR_ERROR_VULKAN_INIT        = 3,
    REACTOR_ERROR_WINDOW_CREATION    = 4,
    REACTOR_ERROR_SHADER_COMPILATION = 5,
    REACTOR_ERROR_MESH_CREATION      = 6,
    REACTOR_ERROR_MATERIAL_CREATION  = 7,
    REACTOR_ERROR_INVALID_HANDLE     = 8,
    REACTOR_ERROR_OUT_OF_MEMORY      = 9,
    REACTOR_ERROR_INVALID_ARGUMENT   = 10,
    REACTOR_ERROR_FRAME_NOT_ACTIVE   = 11,
    REACTOR_ERROR_FRAME_ALREADY_ACTIVE = 12,
    REACTOR_ERROR_UNKNOWN            = 255,
};

const char* reactor_result_string(ReactorResult result);

// =============================================================================
// Opaque Handles — C++ NEVER dereferences these, only passes them around
// =============================================================================

typedef struct ReactorHandle ReactorHandle;
typedef struct MeshHandle MeshHandle;
typedef struct MaterialHandle MaterialHandle;
typedef struct TextureHandle TextureHandle;
typedef struct SceneHandle SceneHandle;
typedef struct CameraHandle CameraHandle;

// =============================================================================
// C API Types (must match Rust repr(C) types)
// =============================================================================

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
// Global Lifecycle — Initialize / Shutdown
// =============================================================================

ReactorResult reactor_initialize();
ReactorResult reactor_shutdown();
bool reactor_is_initialized();

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
// Frame Lifecycle — Command submission boundary
// =============================================================================

ReactorResult reactor_begin_frame();
ReactorResult reactor_end_frame();
bool reactor_is_frame_active();

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
int32_t reactor_add_object(MeshHandle* mesh, MaterialHandle* material, CMat4 transform);
void reactor_set_object_transform(uint32_t index, CMat4 transform);
CMat4 reactor_get_object_transform(uint32_t index);
void reactor_set_object_visible(uint32_t index, bool visible);
void reactor_clear_scene();

// =============================================================================
// Scene Handle API — For custom scenes (opaque handles)
// =============================================================================

SceneHandle* reactor_scene_create();
void reactor_scene_destroy(SceneHandle* scene);
uint32_t reactor_scene_object_count(const SceneHandle* scene);
void reactor_scene_clear(SceneHandle* scene);
int32_t reactor_scene_add_object(SceneHandle* scene, MeshHandle* mesh, MaterialHandle* material, CMat4 transform);
void reactor_scene_set_transform(SceneHandle* scene, uint32_t index, CMat4 transform);
CMat4 reactor_scene_get_transform(const SceneHandle* scene, uint32_t index);
void reactor_scene_set_visible(SceneHandle* scene, uint32_t index, bool visible);
bool reactor_scene_is_visible(const SceneHandle* scene, uint32_t index);
bool reactor_scene_remove_object(SceneHandle* scene, uint32_t index);

// =============================================================================
// Mesh API (Rust owns memory — use reactor_destroy_mesh to free)
// =============================================================================

MeshHandle* reactor_create_mesh(const CVertex* vertices, uint32_t vertex_count, const uint32_t* indices, uint32_t index_count);
MeshHandle* reactor_create_cube();
void reactor_destroy_mesh(MeshHandle* mesh);

// =============================================================================
// Material API (Rust owns memory — use reactor_destroy_material to free)
// =============================================================================

MaterialHandle* reactor_create_material(const uint32_t* vert_spv, uint32_t vert_len, const uint32_t* frag_spv, uint32_t frag_len);
MaterialHandle* reactor_create_material_simple(float r, float g, float b);
MaterialHandle* reactor_create_textured_material(const uint32_t* vert_spv, uint32_t vert_len, const uint32_t* frag_spv, uint32_t frag_len, const TextureHandle* texture);
void reactor_destroy_material(MaterialHandle* material);

// =============================================================================
// Model Loading API (OBJ)
// =============================================================================

struct CObjData {
    uint32_t vertex_count;
    uint32_t index_count;
    uint32_t triangle_count;
    bool success;
};

CObjData reactor_load_obj_info(const char* path);

// =============================================================================
// Physics API
// =============================================================================

struct CCharacterController {
    float position_x, position_y, position_z;
    float velocity_x, velocity_y, velocity_z;
    float height;
    float radius;
    float move_speed;
    float jump_force;
    float gravity;
    bool is_grounded;
};

CCharacterController reactor_character_controller_create(float x, float y, float z);
void reactor_character_controller_update(CCharacterController* controller, float dt, float move_x, float move_z, bool jump, float ground_y);
void reactor_character_controller_eye_position(const CCharacterController* controller, float* out_x, float* out_y, float* out_z);
bool reactor_raycast_aabb(float ray_ox, float ray_oy, float ray_oz, float ray_dx, float ray_dy, float ray_dz,
                          float aabb_min_x, float aabb_min_y, float aabb_min_z, float aabb_max_x, float aabb_max_y, float aabb_max_z, float* out_t);
bool reactor_aabb_intersects(float a_min_x, float a_min_y, float a_min_z, float a_max_x, float a_max_y, float a_max_z,
                             float b_min_x, float b_min_y, float b_min_z, float b_max_x, float b_max_y, float b_max_z);

// =============================================================================
// Texture API (Rust owns memory — use reactor_destroy_texture to free)
// =============================================================================

TextureHandle* reactor_load_texture(const char* path);
TextureHandle* reactor_load_texture_bytes(const uint8_t* data, uint32_t len);
TextureHandle* reactor_create_solid_texture(uint8_t r, uint8_t g, uint8_t b, uint8_t a);
uint32_t reactor_texture_width(const TextureHandle* texture);
uint32_t reactor_texture_height(const TextureHandle* texture);
void reactor_destroy_texture(TextureHandle* texture);

// =============================================================================
// Lighting API
// =============================================================================

int32_t reactor_add_directional_light(float dir_x, float dir_y, float dir_z, float r, float g, float b, float intensity);
int32_t reactor_add_point_light(float x, float y, float z, float r, float g, float b, float intensity, float range);
int32_t reactor_add_spot_light(float pos_x, float pos_y, float pos_z, float dir_x, float dir_y, float dir_z, float r, float g, float b, float intensity, float range, float angle_degrees);
uint32_t reactor_light_count();
void reactor_clear_lights();

// =============================================================================
// Camera Handle API — For custom cameras (opaque handles)
// =============================================================================

CameraHandle* reactor_camera_create_perspective(float fov, float aspect, float near_plane, float far_plane);
void reactor_camera_destroy(CameraHandle* camera);
void reactor_camera_set_position(CameraHandle* camera, float x, float y, float z);
void reactor_camera_set_target(CameraHandle* camera, float x, float y, float z);
CMat4 reactor_camera_get_view_projection(const CameraHandle* camera);
CMat4 reactor_camera_get_view(const CameraHandle* camera);
CMat4 reactor_camera_get_projection(const CameraHandle* camera);

// =============================================================================
// ECS API
// =============================================================================

uint32_t reactor_ecs_create_entity();
void reactor_ecs_destroy_entity(uint32_t entity);
uint32_t reactor_ecs_entity_count();

// =============================================================================
// Debug Draw API
// =============================================================================

void reactor_debug_line(float x1, float y1, float z1, float x2, float y2, float z2, float r, float g, float b);
void reactor_debug_aabb(float min_x, float min_y, float min_z, float max_x, float max_y, float max_z, float r, float g, float b);
void reactor_debug_sphere(float cx, float cy, float cz, float radius, float r, float g, float b);
void reactor_debug_grid(float size, uint32_t divisions, float r, float g, float b);
void reactor_debug_clear();

// =============================================================================
// Animation API
// =============================================================================

uint32_t reactor_animation_create_clip(const char* name);
void reactor_animation_add_position_keyframe(uint32_t clip, float time, float x, float y, float z);
void reactor_animation_add_rotation_keyframe(uint32_t clip, float time, float x, float y, float z, float w);
void reactor_animation_play(uint32_t clip, bool looping);
void reactor_animation_stop(uint32_t clip);
void reactor_animation_update(float dt);

// =============================================================================
// Audio API
// =============================================================================

uint32_t reactor_audio_load(const char* path);
uint32_t reactor_audio_create_source();
void reactor_audio_play(uint32_t source, uint32_t clip);
void reactor_audio_stop(uint32_t source);
void reactor_audio_set_volume(uint32_t source, float volume);
void reactor_audio_set_position(uint32_t source, float x, float y, float z);
void reactor_audio_set_master_volume(float volume);

// =============================================================================
// Post-Processing API
// =============================================================================

void reactor_postprocess_set_bloom(bool enabled, float intensity, float threshold);
void reactor_postprocess_set_tonemapping(bool enabled, float exposure);
void reactor_postprocess_set_vignette(bool enabled, float intensity);
void reactor_postprocess_set_fxaa(bool enabled);

// =============================================================================
// GPU Info API
// =============================================================================

const char* reactor_get_gpu_name();
uint32_t reactor_get_vram_mb();
uint32_t reactor_get_msaa_samples();
bool reactor_is_raytracing_supported();
void reactor_get_vulkan_version(uint32_t* major, uint32_t* minor, uint32_t* patch);

// =============================================================================
// ECS Entity/Component CRUD (Phase 1A)
// =============================================================================

struct CMeshRenderer {
    int32_t mesh_index;
    int32_t material_index;
    bool cast_shadows;
    bool receive_shadows;
    bool visible;
};

struct CCameraComponent {
    float fov;
    float near_plane;
    float far_plane;
    bool is_main;
    CVec4 clear_color;
};

struct CRigidBodyComponent {
    float mass;
    float drag;
    float angular_drag;
    bool use_gravity;
    bool is_kinematic;
    CVec3 velocity;
    CVec3 angular_velocity;
};

// Entity lifecycle
uint32_t reactor_entity_create(const char* name);
bool reactor_entity_destroy(uint32_t entity);
bool reactor_entity_exists(uint32_t entity);
uint32_t reactor_entity_count();
void reactor_entity_set_active(uint32_t entity, bool active);
bool reactor_entity_is_active(uint32_t entity);

// Transform component (every entity has one)
void reactor_entity_set_transform(uint32_t entity, CTransform transform);
CTransform reactor_entity_get_transform(uint32_t entity);
void reactor_entity_set_position(uint32_t entity, float x, float y, float z);
CVec3 reactor_entity_get_position(uint32_t entity);
void reactor_entity_set_rotation(uint32_t entity, float x, float y, float z);
void reactor_entity_set_scale(uint32_t entity, float x, float y, float z);

// Mesh Renderer component
bool reactor_entity_add_mesh_renderer(uint32_t entity, int32_t mesh_index, int32_t material_index);
bool reactor_entity_remove_mesh_renderer(uint32_t entity);
bool reactor_entity_has_mesh_renderer(uint32_t entity);

// Light component
bool reactor_entity_add_light(uint32_t entity, CLight light);
bool reactor_entity_remove_light(uint32_t entity);
bool reactor_entity_has_light(uint32_t entity);
CLight reactor_entity_get_light(uint32_t entity);
void reactor_entity_set_light(uint32_t entity, CLight light);

// Camera component
bool reactor_entity_add_camera(uint32_t entity, float fov, float near_plane, float far_plane, bool is_main);
bool reactor_entity_remove_camera(uint32_t entity);
bool reactor_entity_has_camera(uint32_t entity);

// RigidBody component
bool reactor_entity_add_rigidbody(uint32_t entity, float mass, bool use_gravity);
bool reactor_entity_remove_rigidbody(uint32_t entity);
void reactor_entity_apply_force(uint32_t entity, float fx, float fy, float fz);
void reactor_entity_set_velocity(uint32_t entity, float vx, float vy, float vz);
CVec3 reactor_entity_get_velocity(uint32_t entity);

// ECS Queries (component_mask: 1=MeshRenderer, 2=Light, 4=Camera, 8=RigidBody, 0=all)
uint32_t reactor_query_entities(uint32_t component_mask, uint32_t* out_entities, uint32_t max_results);

// =============================================================================
// PBR Material System (Phase 1C)
// =============================================================================

struct CPBRMaterial {
    CVec4 base_color;
    float metallic;
    float roughness;
    float ao;
    CVec3 emissive;
    float emissive_strength;
    float alpha_cutoff;
    float normal_scale;
    bool double_sided;
    uint32_t alpha_mode; // 0=Opaque, 1=Mask, 2=Blend
};

uint32_t reactor_pbr_create(CPBRMaterial params);
uint32_t reactor_pbr_create_default();
uint32_t reactor_pbr_create_instance(uint32_t parent_id);
void reactor_pbr_destroy(uint32_t material_id);
CPBRMaterial reactor_pbr_get(uint32_t material_id);
void reactor_pbr_set_base_color(uint32_t material_id, float r, float g, float b, float a);
void reactor_pbr_set_metallic_roughness(uint32_t material_id, float metallic, float roughness);
void reactor_pbr_set_emissive(uint32_t material_id, float r, float g, float b, float strength);
uint32_t reactor_pbr_count();

// =============================================================================
// FrameGraph API (Phase 2A)
// =============================================================================

typedef struct CFrameGraphHandle CFrameGraphHandle;

struct CFrameGraphStats {
    uint32_t total_passes;
    uint32_t enabled_passes;
    uint32_t total_resources;
    uint32_t transient_resources;
    uint32_t barriers_generated;
};

CFrameGraphHandle* reactor_frame_graph_create();
void reactor_frame_graph_destroy(CFrameGraphHandle* fg);
uint32_t reactor_frame_graph_create_resource(CFrameGraphHandle* fg, const char* name,
    uint32_t resource_type, uint32_t width, uint32_t height, uint32_t format, bool persistent);
uint32_t reactor_frame_graph_add_pass(CFrameGraphHandle* fg, const char* name,
    const uint32_t* reads, uint32_t read_count, const uint32_t* writes, uint32_t write_count, int32_t order);
bool reactor_frame_graph_compile(CFrameGraphHandle* fg);
CFrameGraphStats reactor_frame_graph_get_stats(const CFrameGraphHandle* fg);
CFrameGraphHandle* reactor_frame_graph_create_forward(uint32_t width, uint32_t height);
CFrameGraphHandle* reactor_frame_graph_create_deferred(uint32_t width, uint32_t height);

// =============================================================================
// Render Stats & Telemetry (Phase 2B)
// =============================================================================

struct CRenderStats {
    float fps;
    float frame_time_ms;
    uint32_t draw_calls;
    uint32_t triangles;
    uint32_t vertices;
    uint32_t scene_objects;
    uint32_t visible_objects;
    uint32_t vram_used_mb;
    uint32_t vram_total_mb;
    float cpu_frame_ms;
    float gpu_frame_ms;
};

struct CMemoryBudget {
    uint64_t device_local_used;
    uint64_t device_local_budget;
    uint64_t host_visible_used;
    uint64_t host_visible_budget;
    uint32_t total_allocations;
};

CRenderStats reactor_get_render_stats();
CMemoryBudget reactor_get_memory_budget();

// =============================================================================
// Scene Serialization (Phase 2C)
// =============================================================================

uint32_t reactor_scene_serialize(uint8_t* buffer, uint32_t buffer_size);
uint32_t reactor_scene_serialize_size();

// =============================================================================
// Compute Pipeline (Phase 3)
// =============================================================================

typedef struct CComputePipelineHandle CComputePipelineHandle;

CComputePipelineHandle* reactor_compute_create(const uint32_t* spv_code, uint32_t spv_len);
void reactor_compute_destroy(CComputePipelineHandle* pipeline);
bool reactor_compute_dispatch(CComputePipelineHandle* pipeline, uint32_t group_x, uint32_t group_y, uint32_t group_z);

// =============================================================================
// Runtime-Editor Bridge (Play Mode)
// =============================================================================

bool reactor_play_enter();
void reactor_play_exit();
void reactor_play_pause(bool paused);
bool reactor_play_is_playing();
bool reactor_play_is_paused();
float reactor_play_get_time();
void reactor_play_update(float dt);

} // extern "C"
