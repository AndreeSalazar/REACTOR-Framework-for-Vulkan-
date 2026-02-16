# REACTOR 3D — C++ Vulkan Example

The **definitive** example of how to use REACTOR in C++.

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

## 🎯 Features Demonstrated

- **Input**: Keyboard + Mouse
- **Camera**: Position, Target, FPS-style look
- **Lighting**: Directional + Point lights
- **Post-Processing**: Bloom, Tone Mapping, Vignette, FXAA
- **Debug**: Grid, Axes, Lines
- **GPU Info**: Hardware detection
- **Animation**: Update system
- **Audio**: Ready to use

## 📋 Controls

| Key | Action |
|-----|--------|
| WASD | Move camera |
| Space | Move up |
| Shift | Move down / Sprint |
| Mouse | Look around (click to capture) |
| ESC | Release mouse / Exit |

## 🔧 Requirements

- C++17 compiler
- Vulkan SDK
- REACTOR C API library (`reactor_c_api.dll` / `.so`)

## 📁 Files

```
3D/
├── main.cpp          # Example code (3 styles)
├── CMakeLists.txt    # Build configuration
└── README.md         # This file
```

## 🌟 Why REACTOR?

| Feature | REACTOR | Others |
|---------|---------|--------|
| Setup | 1 line | 100+ lines |
| Vulkan | Automatic | Manual |
| RAII | Perfect | Manual |
| Performance | Native | Overhead |
| Control | Total | Limited |

**REACTOR = Vulkan power with zero boilerplate.**
