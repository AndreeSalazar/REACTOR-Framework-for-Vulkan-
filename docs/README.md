# REACTOR Framework — Documentación v1.0.5

Bienvenido a la documentación oficial de **REACTOR Framework**.

## Guías Disponibles

| Guía | Descripción |
| ---- | ----------- |
| [Manual General](manual.md) | Manual corto y completo para uso general |
| [Guía Rust](rust-guide.md) | Desarrollo de juegos con Rust |
| [Guía C++](cpp-guide.md) | Desarrollo de juegos con C++ (ECS, PBR, FrameGraph, 9 ejemplos) |
| [Arquitectura](architecture.md) | Diagrama de sistema, C ABI, ownership model |
| [Roadmap C++](cpp_editor_parity_roadmap.md) | Estado de paridad C++ con Rust core |
| [Cómo Compilar](../HOW_BUILD.md) | Guía paso a paso para compilar Rust + C++ |

## Inicio Rápido

### Rust

```rust
use reactor::prelude::*;

fn main() {
    reactor::run(MiJuego::default());
}
```

### C++

```cpp
#include <reactor/application.hpp>
using namespace reactor;

class MiJuego : public Application {
    Config config() override { return Config("Mi Juego", 1280, 720); }
    void on_init() override {
        Camera::set_position({0, 2, 5});
        Lighting::add_directional({-0.5f, -1, -0.3f}, {1, 1, 1}, 1.0f);
    }
    void on_update(float dt) override {
        if (Input::key_pressed(Input::KEY_ESCAPE())) Window::request_close();
    }
};

int main() { return MiJuego().run(); }
```

## Versión Actual: 1.0.5

### Novedades principales

- **C ABI completo** — 3300+ líneas de funciones `extern "C"`
- **C++ SDK** — 1477 líneas header-only con wrappers RAII
- **ECS** — Entity/Component CRUD, queries con bitmask
- **PBR Materials** — Metallic/roughness, instances, emissive
- **FrameGraph** — Render passes declarativos, forward/deferred presets
- **Telemetry** — GPU stats, memory budget, VRAM real desde Vulkan
- **PlayMode** — Play-in-editor bridge (enter/exit/pause)
- **Scene Serialization** — Export a JSON
- **9 Ejemplos C++** — Cada uno en carpeta propia con escenario único
- **Editor REACTOR** — egui + egui_dock (Viewport, Hierarchy, Inspector, Console)
- **ADead-GPU** — ISR, SDF, Ray Marching, Hybrid Rendering

## Arquitectura

```text
C++ Game / Editor
        |
   C++ SDK (application.hpp — 1477 líneas)
        |
   C ABI (core.hpp — 646 declarations)
        |
   reactor_c_api.dll (Rust — 3300+ líneas)
        |
   Rust Core (Vulkan context, ECS, PBR, FrameGraph)
        |
   Vulkan 1.3 → GPU
```

## Licencia

MIT License — **Powered by Salazar-interactive**
