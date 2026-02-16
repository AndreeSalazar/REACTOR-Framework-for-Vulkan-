# REACTOR C++ Integration

**ONE CALL — Ultra-Simple, Ultra-Productive, Ultra-Powerful**

This folder contains the C/C++ interop layer for REACTOR Framework.
Use `ReactorApp()` to initialize everything with a single call.

## Quick Start

### The Simplest Way (ONE CALL)
```cpp
#include <reactor/reactor.hpp>

int main() {
    return reactor::ReactorApp("My Game");
}
```

### With Callbacks (Functional Style)
```cpp
#include <reactor/reactor.hpp>

int main() {
    float rotation = 0.0f;
    
    return reactor::ReactorApp("My Game", 1280, 720,
        []() { /* on_init */ },
        [&](float dt) { rotation += dt; },
        []() { /* on_render */ }
    );
}
```

### Class-Based (Recommended for Larger Games)
```cpp
#include <reactor/reactor.hpp>

class MyGame : public reactor::Application {
    float rotation_ = 0.0f;

public:
    reactor::Config config() override {
        return reactor::Config("My Game")
            .with_size(1280, 720)
            .with_vsync(true);
    }

    void on_init() override {
        reactor::Log::info("Game initialized!");
        reactor::Camera::set_position({0, 2, 5});
    }

    void on_update(float dt) override {
        rotation_ += dt;
        
        // Input handling
        if (reactor::Input::key_down(reactor::Input::KEY_W())) {
            // Move forward
        }
        if (reactor::Input::key_pressed(reactor::Input::KEY_ESCAPE())) {
            reactor::Window::request_close();
        }
    }

    void on_render() override {
        auto vp = reactor::Camera::view_projection();
        auto model = reactor::Mat4::RotationY(rotation_);
        // Draw with vp * model
    }
};

int main() {
    return MyGame().run();
}
```

## Architecture

```
Rust Core (reactor crate)
    ↓ extern "C"
C API (reactor_c_api/) ← Produces reactor_c_api.dll
    ↓ C++ wrapper
C++ SDK (reactor_cpp/) ← Header-only, RAII classes
    ↓ ONE CALL
Your Game (ReactorApp() or class MyGame : public reactor::Application)
```

## Structure

```
cpp/
├── reactor_c_api/           # Rust → C bridge
│   ├── Cargo.toml
│   ├── src/lib.rs           # extern "C" exports (~1000 lines)
│   └── cbindgen.toml
│
├── reactor_cpp/             # C++ SDK (header-only)
│   ├── CMakeLists.txt
│   └── include/reactor/
│       ├── reactor.hpp      # Main header (include this)
│       ├── core.hpp         # C API declarations
│       ├── types.hpp        # Vec2, Vec3, Vec4, Mat4, Transform
│       └── application.hpp  # Application, Input, Time, Camera, SDF, Log
│
└── examples/
    └── hello_cpp/
        ├── main.cpp         # Complete example
        └── CMakeLists.txt
```

## Building

### Step 1: Build the Rust C API
```bash
cd cpp/reactor_c_api
cargo build --release
# Produces: target/release/reactor_c_api.dll (Windows)
# Produces: target/release/libreactor_c_api.so (Linux)
# Produces: target/release/libreactor_c_api.dylib (macOS)
```

### Step 2: Build C++ Example
```bash
cd cpp/examples/hello_cpp
cmake -B build
cmake --build build
./build/hello_cpp  # or build\Debug\hello_cpp.exe on Windows
```

## API Reference

### Core Classes

| Class | Description |
|-------|-------------|
| `reactor::Application` | Base class for games. Override `on_init()`, `on_update()`, `on_render()` |
| `reactor::Config` | Configuration builder with fluent API |
| `reactor::Input` | Keyboard and mouse state |
| `reactor::Time` | Frame timing (delta, fps, total time) |
| `reactor::Window` | Window state and control |
| `reactor::Camera` | Built-in camera control |
| `reactor::SDF` | Signed Distance Functions (ADead-GPU) |
| `reactor::Log` | Debug logging |

### Math Types

| Type | Description |
|------|-------------|
| `Vec2` | 2D vector with operators |
| `Vec3` | 3D vector with dot, cross, normalize |
| `Vec4` / `Color` | 4D vector / RGBA color |
| `Mat4` | 4x4 matrix with Perspective, LookAt, Translation, Rotation, Scale |
| `Transform` | Position + Rotation + Scale with matrix() |

### Input

```cpp
// Keyboard
reactor::Input::key_down(reactor::Input::KEY_W())   // Held down
reactor::Input::key_pressed(reactor::Input::KEY_SPACE())  // Just pressed

// Mouse
reactor::Vec2 pos = reactor::Input::mouse_position();
reactor::Vec2 delta = reactor::Input::mouse_delta();
bool left = reactor::Input::mouse_left();
```

### Time

```cpp
float dt = reactor::Time::delta();      // Seconds since last frame
float fps = reactor::Time::fps();       // Current FPS
float total = reactor::Time::total();   // Total time since start
uint64_t frame = reactor::Time::frame_count();
```

### Camera

```cpp
reactor::Camera::set_position({0, 5, 10});
reactor::Camera::set_target({0, 0, 0});
reactor::Mat4 vp = reactor::Camera::view_projection();
```

### SDF (ADead-GPU)

```cpp
float d1 = reactor::SDF::sphere({0,0,0}, 1.0f);
float d2 = reactor::SDF::box({0,0,0}, {0.5f, 0.5f, 0.5f});
float d3 = reactor::SDF::op_smooth_union(d1, d2, 0.1f);
```

### Matrices

```cpp
auto identity = reactor::Mat4::Identity();
auto translate = reactor::Mat4::Translation(1, 2, 3);
auto rotate = reactor::Mat4::RotationY(reactor::PI / 4);
auto scale = reactor::Mat4::Scale(2.0f);
auto perspective = reactor::Mat4::Perspective(60, 16.0f/9.0f, 0.1f, 1000.0f);
auto lookAt = reactor::Mat4::LookAt({0,5,10}, {0,0,0});

auto mvp = perspective * lookAt * translate * rotate * scale;
```

## Features

- **ONE CALL**: `ReactorApp()` initializes Vulkan, window, input, everything
- **Header-only C++ SDK**: Just `#include <reactor/reactor.hpp>`
- **Zero overhead**: Direct calls to Rust via C API
- **Full Vulkan power**: Ray tracing, compute, MSAA, all from Rust core
- **ADead-GPU**: SDF, ray marching, hybrid rendering exposed to C++
- **Modern C++17**: RAII, lambdas, fluent builders
- **Cross-platform**: Windows, Linux, macOS
