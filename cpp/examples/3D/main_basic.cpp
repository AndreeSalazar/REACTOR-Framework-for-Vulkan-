// =============================================================================
// REACTOR 3D — Formal Lifecycle Example
// =============================================================================
// Demonstrates the professional REACTOR lifecycle:
//
//   reactor_initialize()         — global init
//   reactor_run() / callbacks    — main loop
//   reactor_shutdown()           — clean teardown
//
// Ownership: Rust creates → Rust destroys (opaque handles only)
// Errors:    ReactorResult enum (no exceptions across FFI)
// =============================================================================

#include <reactor/core.hpp>
#include <cstdio>

// Global state for callbacks
static float g_rotation = 0.0f;

// Opaque handles — C++ never dereferences these
static MeshHandle*     g_cube_mesh     = nullptr;
static MaterialHandle* g_cube_material = nullptr;
static int32_t         g_cube_index    = -1;

void on_init() {
    printf("+==============================================================+\n");
    printf("|           REACTOR 3D - C++ Vulkan Example                    |\n");
    printf("+==============================================================+\n");
    printf("\n");
    printf("Version: %s\n", reactor_version());
    printf("GPU: %s\n", reactor_get_gpu_name());
    printf("MSAA: %ux\n", reactor_get_msaa_samples());
    printf("Initialized: %s\n", reactor_is_initialized() ? "YES" : "NO");
    printf("\n");
    printf("Controles:\n");
    printf("  ESC - Salir\n");
    printf("\n");
    
    // Setup camera
    reactor_set_camera_position(0.0f, 2.0f, 5.0f);
    reactor_set_camera_target(0.0f, 0.0f, 0.0f);
    
    // Setup lighting
    reactor_add_directional_light(-0.5f, -1.0f, -0.3f, 1.0f, 0.98f, 0.95f, 1.0f);
    
    // Create a cube mesh (Rust owns the memory)
    g_cube_mesh = reactor_create_cube();
    if (g_cube_mesh) {
        printf("Cubo creado correctamente!\n");
        
        // Create a simple material (Rust owns the memory)
        g_cube_material = reactor_create_material_simple(1.0f, 0.5f, 0.2f);
        if (g_cube_material) {
            printf("Material creado correctamente!\n");
            
            // Add cube to scene with identity transform
            CMat4 transform = {};
            transform.cols[0][0] = 1.0f;
            transform.cols[1][1] = 1.0f;
            transform.cols[2][2] = 1.0f;
            transform.cols[3][3] = 1.0f;
            
            g_cube_index = reactor_add_object(g_cube_mesh, g_cube_material, transform);
            if (g_cube_index >= 0) {
                printf("Cubo agregado a la escena (index: %d)\n", g_cube_index);
            }
        }
    }
    
    printf("Objetos en escena: %u\n", reactor_object_count());
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

// =============================================================================
// MAIN — Formal Lifecycle
// =============================================================================
//
//   1. reactor_initialize()   — prepare subsystems
//   2. reactor_run_simple()   — enter main loop (creates window, Vulkan, etc.)
//   3. reactor_shutdown()     — release all resources
//
// =============================================================================

int main() {
    printf("\nStarting REACTOR 3D...\n\n");
    
    // 1. Initialize REACTOR subsystems
    ReactorResult result = reactor_initialize();
    if (result != REACTOR_OK) {
        printf("ERROR: reactor_initialize() failed: %s\n", reactor_result_string(result));
        return -1;
    }
    
    // 2. Run the application (blocking — returns when window closes)
    int32_t exit_code = reactor_run_simple(
        "REACTOR 3D",    // title
        1280,            // width
        720,             // height
        on_init,         // init callback
        on_update,       // update callback
        on_render        // render callback
    );
    
    // 3. Shutdown — release all resources
    result = reactor_shutdown();
    if (result != REACTOR_OK) {
        printf("WARN: reactor_shutdown() returned: %s\n", reactor_result_string(result));
    }
    
    printf("\nREACTOR shutdown complete.\n");
    return exit_code;
}
