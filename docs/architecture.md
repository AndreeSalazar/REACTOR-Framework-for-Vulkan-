# REACTOR Framework — Architecture

## System Diagram

```text
+------------------------------------------------------------------+
|                    C++ Game / App / Editor                        |
|  9 examples: ecs_scene, pbr_materials, frame_graph, fps_ctrl...  |
|  Editor-REACTOR (egui + egui_dock)                               |
+------------------------------------------------------------------+
        |                    |                    |
        | reactor_initialize | reactor_run_simple | reactor_shutdown
        | reactor_begin_frame| reactor_end_frame  |
        v                    v                    v
+------------------------------------------------------------------+
|              C++ SDK (application.hpp — 1477 lines)              |
|                                                                  |
|  reactor::Application, Entity, ECS, PBRMaterial, FrameGraph      |
|  RenderStats, PlayMode, SceneSerializer, GPUInfo, Error          |
|  Scene, Lighting, Camera, Input, Time, Window, Config            |
+------------------------------------------------------------------+
        |
        | #include <reactor/core.hpp>  (646 C declarations)
        v
+------------------------------------------------------------------+
|              Stable C ABI Contract (core.hpp)                    |
|                                                                  |
|  3300+ extern "C" functions                                      |
|  Opaque Handles:  MeshHandle*, MaterialHandle*, SceneHandle*     |
|  Error Model:     ReactorResult enum (no exceptions)             |
|  Ownership:       Rust creates -> Rust destroys                  |
|  Lifecycle:       initialize -> run -> shutdown                  |
|  Frame:           begin_frame -> [update/render] -> end_frame    |
|  ECS:             entity_create/destroy, component CRUD, queries |
|  PBR:             pbr_create/destroy, instances, parameters      |
|  FrameGraph:      create/add_pass/compile, forward/deferred      |
|  Telemetry:       render_stats, memory_budget, gpu_info          |
|  PlayMode:        enter/exit/pause, scene snapshot               |
+------------------------------------------------------------------+
        |
        | extern "C" fn reactor_*()
        | reactor_c_api.dll / .so  (3300+ lines Rust)
        v
+------------------------------------------------------------------+
|                   Rust Core (lib.rs)                             |
|                                                                  |
|  ReactorState (global singleton, Mutex-protected)                |
|  - Reactor (Vulkan context)                                      |
|  - ECS World (entities, components, queries)                     |
|  - PBR Material registry (base + instances)                      |
|  - FrameGraph (passes, resources, barriers)                      |
|  - Scene, Camera, Lighting, Physics, Culling                     |
|  - Input state (keys, mouse)                                     |
|  - Time, frame tracking, render stats                            |
|  - SPIR-V shaders (embedded via include_bytes!)                  |
|  - PlayMode bridge (snapshot, pause, time)                       |
+------------------------------------------------------------------+
        |
        | Reactor::init(), draw_scene(), handle_event()
        v
+------------------------------------------------------------------+
|                  Vulkan 1.3 Backend                              |
|                                                                  |
|  VulkanContext  — Instance, Device, Queues                       |
|  Swapchain     — Triple buffering                                |
|  RenderPass    — MSAA 4x + Depth (D32_SFLOAT)                   |
|  Pipeline      — Vertex + Fragment shaders                       |
|  Ray Tracing   — Auto-detected (VK_KHR_ray_tracing_pipeline)    |
|  Memory        — gpu-allocator (Vulkan Memory Allocator)         |
+------------------------------------------------------------------+
        |
        v
+------------------------------------------------------------------+
|                     GPU Hardware                                 |
|  NVIDIA RTX 3060 12GB — 3000+ FPS @ 1280x720                    |
+------------------------------------------------------------------+
```

## Lifecycle

```
main()
  |
  +-- reactor_initialize()          // Prepare global state
  |
  +-- reactor_run_simple()          // Enter main loop
  |     |
  |     +-- EventLoop::new()        // winit event loop
  |     +-- Window::create()        // OS window (no GLFW)
  |     +-- Reactor::init()         // Vulkan setup
  |     |
  |     +-- [loop]
  |     |     +-- handle_event()    // Input, resize, close
  |     |     +-- time.update()     // Delta time
  |     |     +-- on_update(dt)     // User logic
  |     |     +-- draw_scene()      // Vulkan render
  |     |     +-- on_render()       // User custom render
  |     |     +-- request_redraw()  // Next frame
  |     |
  |     +-- device_wait_idle()      // GPU sync before exit
  |
  +-- reactor_shutdown()            // Release all resources
  |
  +-- return 0
```

## Ownership Model

```
Rule: Rust creates -> Rust destroys. C++ only holds opaque pointers.

  C++ side                          Rust side
  --------                          ---------
  MeshHandle* mesh = nullptr;       Box<MeshHandle> on heap
       |                                 |
       +-- reactor_create_cube() ------->+-- Mesh::new() via Vulkan
       |                                 |   Returns Box::into_raw()
       +-- reactor_add_object(mesh) ---->+-- Arc::new(mesh) into Scene
       |                                 |
       +-- [mesh consumed by scene] ---->+-- Scene owns Arc<Mesh>
       |                                 |
       | DO NOT call delete mesh;        | Rust Drop trait handles cleanup
       | DO NOT dereference mesh;        | VkBuffer freed automatically
```

## Error Model

```
// ABI-safe error codes — no exceptions, no panics across FFI

enum ReactorResult : int32_t {
    REACTOR_OK                        = 0,   // Success
    REACTOR_ERROR_NOT_INITIALIZED     = 1,   // Call reactor_initialize() first
    REACTOR_ERROR_ALREADY_INITIALIZED = 2,   // Already initialized
    REACTOR_ERROR_VULKAN_INIT         = 3,   // Vulkan setup failed
    REACTOR_ERROR_WINDOW_CREATION     = 4,   // OS window failed
    REACTOR_ERROR_SHADER_COMPILATION  = 5,   // SPIR-V error
    REACTOR_ERROR_MESH_CREATION       = 6,   // GPU buffer failed
    REACTOR_ERROR_MATERIAL_CREATION   = 7,   // Pipeline creation failed
    REACTOR_ERROR_INVALID_HANDLE      = 8,   // Null or freed handle
    REACTOR_ERROR_OUT_OF_MEMORY       = 9,   // Allocation failed
    REACTOR_ERROR_INVALID_ARGUMENT    = 10,  // Bad parameter
    REACTOR_ERROR_FRAME_NOT_ACTIVE    = 11,  // end_frame without begin_frame
    REACTOR_ERROR_FRAME_ALREADY_ACTIVE = 12, // begin_frame called twice
    REACTOR_ERROR_UNKNOWN             = 255, // Unclassified
};

// Usage:
ReactorResult r = reactor_initialize();
if (r != REACTOR_OK) {
    printf("Error: %s\n", reactor_result_string(r));
}
```

## Frame Lifecycle (Manual Mode)

```
// For advanced users who want explicit frame control:

reactor_initialize();

while (!reactor_should_close()) {
    reactor_begin_frame();      // Start frame, update time, clear input

    // ... game logic here ...
    // ... modify scene, camera, lighting ...

    reactor_end_frame();        // Submit draw commands, present
}

reactor_shutdown();
```

## C ABI Contract Rules

1. **Handles are opaque** — `typedef struct MeshHandle MeshHandle;`
2. **Rust owns memory** — Never `delete` or `free` a handle
3. **Use destroy functions** — `reactor_destroy_mesh()`, `reactor_destroy_material()`
4. **No exceptions** — All errors via `ReactorResult`
5. **No panics** — Rust catches all errors before FFI boundary
6. **Thread safety** — Global state is `Mutex<Option<ReactorState>>`
7. **ABI stability** — `repr(C)` on all types, `extern "C"` on all functions
8. **Versioned** — `reactor_version()`, `reactor_get_version_major/minor/patch()`

## Layer Summary

| Layer | Language | Responsibility |
|-------|----------|----------------|
| Game | C++ | Gameplay logic, callbacks |
| SDK | C++ | Wrappers, RAII, helpers |
| ABI | C | Stable binary interface |
| Core | Rust | Vulkan, memory, systems |
| Driver | Vulkan 1.3 | GPU commands |
| Hardware | GPU | Rendering |
