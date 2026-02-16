// =============================================================================
// REACTOR 3D — Basic Working Example
// =============================================================================
// This uses only the absolute minimum C API functions.
// Demonstrates ReactorApp() pattern with Vulkan rendering.
// =============================================================================

#include <cstdint>
#include <cstdio>

extern "C" {
    // Callback types
    typedef void (*InitCallback)();
    typedef void (*UpdateCallback)(float);
    typedef void (*RenderCallback)();
    
    // Core functions
    int32_t reactor_run_simple(
        const char* title,
        uint32_t width,
        uint32_t height,
        InitCallback on_init,
        UpdateCallback on_update,
        RenderCallback on_render
    );
    
    // Info functions
    const char* reactor_get_gpu_name();
    uint32_t reactor_get_msaa_samples();
    float reactor_get_fps();
    uint64_t reactor_get_frame_count();
    
    // Input functions
    int32_t reactor_key_pressed(uint32_t key);
    uint32_t reactor_key_escape();
    
    // Window functions
    void reactor_request_close();
    
    // Camera functions
    void reactor_set_camera_position(float x, float y, float z);
    void reactor_set_camera_target(float x, float y, float z);
    
    // Lighting functions
    int32_t reactor_add_directional_light(float dx, float dy, float dz, float r, float g, float b, float intensity);
    
    // Mesh functions
    void* reactor_create_cube();
    void reactor_destroy_mesh(void* mesh);
    
    // Material functions
    void* reactor_create_material_simple(float r, float g, float b);
    void reactor_destroy_material(void* material);
    
    // Scene functions
    struct CMat4 { float cols[4][4]; };
    int32_t reactor_add_object(void* mesh, void* material, CMat4 transform);
    void reactor_set_object_transform(uint32_t index, CMat4 transform);
    uint32_t reactor_object_count();
}

// Global state for callbacks
static float g_rotation = 0.0f;

void on_init() {
    printf("+==============================================================+\n");
    printf("|           REACTOR 3D - C++ Vulkan Example                    |\n");
    printf("+==============================================================+\n");
    printf("\n");
    printf("GPU: %s\n", reactor_get_gpu_name());
    printf("MSAA: %ux\n", reactor_get_msaa_samples());
    printf("\n");
    printf("Controles:\n");
    printf("  ESC - Salir\n");
    printf("\n");
    
    // Setup camera
    reactor_set_camera_position(0.0f, 3.0f, 8.0f);
    reactor_set_camera_target(0.0f, 0.0f, 0.0f);
    
    // Setup lighting
    reactor_add_directional_light(-0.5f, -1.0f, -0.3f, 1.0f, 0.98f, 0.95f, 1.0f);
    
    printf("REACTOR inicializado!\n");
}

void on_update(float dt) {
    g_rotation += dt;
    
    // Exit on ESC
    if (reactor_key_pressed(reactor_key_escape())) {
        reactor_request_close();
    }
    
    // FPS display every 60 frames
    if (reactor_get_frame_count() % 60 == 0) {
        printf("\rFPS: %.1f    ", reactor_get_fps());
        fflush(stdout);
    }
}

void on_render() {
    // Scene is rendered automatically by REACTOR
}

int main() {
    printf("\n");
    printf("Starting REACTOR 3D...\n");
    printf("\n");
    
    // THE ONE CALL - ReactorApp() pattern
    return reactor_run_simple(
        "REACTOR 3D",    // title
        1280,            // width
        720,             // height
        on_init,         // init callback
        on_update,       // update callback
        on_render        // render callback
    );
}
