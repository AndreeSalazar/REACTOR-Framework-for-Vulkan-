# REACTOR C++ Examples — Vulkan Examples

The **definitive** examples of how to use REACTOR in C++.

## 🚀 Quick Start

```cpp
#include <reactor/reactor.hpp>

int main() {
    return reactor::ReactorApp("My Game");
}
```

**That's it.** One line. You're rendering 3D with Vulkan.

## 📦 Build

```bash
# 1. Build REACTOR C API (from project root)
cargo build --release -p reactor-c-api

# 2. Build this example
cd cpp/examples/3D
cmake -B build
cmake --build build

# 3. Run
./build/reactor_3d
```

## 🎮 Three Styles

### Style 1: One Call (Absolute Minimum)

```cpp
int main() {
    return reactor::ReactorApp("My Game");
}
```

### Style 2: Lambda (Quick Prototyping)

```cpp
int main() {
    float rotation = 0;
    
    return reactor::ReactorApp(
        reactor::Config("My Game").with_size(1280, 720),
        []() { /* init */ },
        [&](float dt) { rotation += dt; },
        []() { /* render */ }
    );
}
```

### Style 3: Class (Full Control)

```cpp
class MyGame : public reactor::Application {
    void on_init() override {
        reactor::Lighting::add_directional({0,-1,0}, {1,1,1}, 1);
    }
    
    void on_update(float dt) override {
        if (reactor::Input::key_down(reactor::Input::KEY_ESCAPE()))
            reactor::Window::request_close();
    }
};

int main() {
    return MyGame().run();
}
```

## 🎯 Examples (9 total)

- **`main_basic.cpp`** — Basic cube lifecycle (original)
- **`ecs_scene/`** — Full ECS: entities, transform, mesh renderer, light, camera, rigidbody, queries
- **`pbr_materials/`** — PBR material system: metallic/roughness, instances, emissive
- **`frame_graph/`** — FrameGraph: custom passes, resources, forward/deferred presets
- **`fps_controller/`** — FPS character controller: WASD, mouse look, jump, gravity
- **`lighting_showcase/`** — Multi-light: directional, point (orbiting), spot (animated)
- **`telemetry_stats/`** — GPU stats, memory budget, scene serialization, stress test
- **`play_mode/`** — Play-in-editor bridge: enter/exit/pause, scene snapshot
- **`multi_object/`** — Large scene: 225 objects, wave animation, visibility toggle, queries

## 🔧 Requirements

- C++17 compiler (MSVC, GCC, Clang)
- Vulkan SDK
- REACTOR C API library (`reactor_c_api.dll` / `.so`)

## 📁 Structure

```text
3D/
├── main_basic.cpp          # Basic cube (original)
├── ecs_scene/main.cpp      # ECS entity/component CRUD
├── pbr_materials/main.cpp  # PBR material system
├── frame_graph/main.cpp    # FrameGraph render passes
├── fps_controller/main.cpp # FPS character controller
├── lighting_showcase/main.cpp # Multi-light showcase
├── telemetry_stats/main.cpp   # GPU stats & telemetry
├── play_mode/main.cpp      # Play-in-editor bridge
├── multi_object/main.cpp   # Large scene management
├── CMakeLists.txt          # Builds all 9 examples
└── README.md               # This file
```

## 🌟 Rust + C++ Synergy

Each example demonstrates how **Rust** and **C++** share their strengths:

- **Rust** handles: Vulkan GPU backend, memory safety, shader pipelines, ECS storage
- **C++** handles: Game logic, scene setup, material tweaking, physics control, UI

**REACTOR = Vulkan power with zero boilerplate. Best of both worlds.**
