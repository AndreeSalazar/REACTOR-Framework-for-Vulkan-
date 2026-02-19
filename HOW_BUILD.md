# 🔧 How to Build — Guía de Construcción REACTOR

## 📋 Requisitos Previos

### Windows
```powershell
# 1. Instalar Rust
winget install Rustlang.Rustup

# 2. Instalar Vulkan SDK
# Descargar de: https://vulkan.lunarg.com/sdk/home
# O con winget:
winget install KhronosGroup.VulkanSDK

# 3. Verificar instalación
rustc --version    # Debe ser 1.70+
cargo --version
vulkaninfo         # Debe mostrar info de tu GPU
```

### Linux (Ubuntu/Debian)
```bash
# 1. Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Instalar Vulkan
sudo apt install vulkan-tools libvulkan-dev vulkan-validationlayers

# 3. Verificar
vulkaninfo
```

### macOS
```bash
# 1. Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Instalar MoltenVK (Vulkan para macOS)
brew install molten-vk

# 3. Verificar
vulkaninfo
```

---

## 🚀 Construcción Rápida

### Clonar y Ejecutar
```bash
git clone https://github.com/user/REACTOR-Framework-for-Vulkan-.git
cd REACTOR-Framework-for-Vulkan-

# Ejecutar ejemplo simple
cargo run --example cube

# Ejecutar con optimizaciones
cargo run --example cube --release
```

### Compilar Librería
```bash
# Debug (rápido de compilar, lento de ejecutar)
cargo build

# Release (lento de compilar, rápido de ejecutar)
cargo build --release
```

---

## 🎮 Crear Tu Propio Juego

### Opción 1: Nuevo Archivo en `examples/`

Crea `examples/mi_juego.rs`:

```rust
use reactor::prelude::*;
use std::sync::Arc;

struct MiJuego {
    rotacion: f32,
}

impl ReactorApp for MiJuego {
    fn config(&self) -> ReactorConfig {
        ReactorConfig::new("Mi Juego")
            .with_size(1920, 1080)
            .with_vsync(true)
            .with_msaa(4)
    }

    fn init(&mut self, ctx: &mut ReactorContext) {
        ctx.camera.position = Vec3::new(0.0, 2.0, 4.0);
        // Tu código de inicialización aquí...
    }

    fn update(&mut self, ctx: &mut ReactorContext) {
        self.rotacion += ctx.time.delta();
        // Tu lógica de juego aquí...
    }
}

fn main() {
    reactor::run(MiJuego { rotacion: 0.0 });
}
```

Agrega a `Cargo.toml`:
```toml
[[example]]
name = "mi_juego"
path = "examples/mi_juego.rs"
```

Ejecuta:
```bash
cargo run --example mi_juego
```

### Opción 2: Proyecto Separado

```bash
# Crear nuevo proyecto
cargo new mi_juego
cd mi_juego

# Agregar REACTOR como dependencia en Cargo.toml
```

En `Cargo.toml`:
```toml
[package]
name = "mi_juego"
version = "0.1.0"
edition = "2021"

[dependencies]
reactor = { path = "../REACTOR-Framework-for-Vulkan-" }
# O desde git:
# reactor = { git = "https://github.com/user/REACTOR-Framework-for-Vulkan-.git" }
```

En `src/main.rs`:
```rust
use reactor::prelude::*;

struct MiJuego;

impl ReactorApp for MiJuego {
    fn config(&self) -> ReactorConfig {
        ReactorConfig::new("Mi Juego").with_size(1280, 720)
    }
    fn init(&mut self, _ctx: &mut ReactorContext) {}
    fn update(&mut self, _ctx: &mut ReactorContext) {}
}

fn main() { reactor::run(MiJuego); }
```

---

## 🔨 Compilar C++ SDK

### Requisitos Adicionales

- CMake 3.16+
- Compilador C++17 (MSVC 2022, GCC 11+, Clang 14+)

### Paso 1: Compilar DLL de Rust

```bash
cd cpp/reactor_c_api
cargo build --release

# La DLL estará en: target/release/reactor_c_api.dll (Windows)
#                   target/release/libreactor_c_api.so (Linux)
#                   target/release/libreactor_c_api.dylib (macOS)
```

### Paso 2: Compilar los 9 Ejemplos C++

```bash
cd cpp/examples/3D
cmake -B build
cmake --build build --config Release
```

### Paso 3: Ejecutar Ejemplos

```bash
# Windows
.\build\Release\reactor_3d.exe              # Cubo básico
.\build\Release\reactor_ecs_scene.exe       # ECS entity/component CRUD
.\build\Release\reactor_pbr_materials.exe   # PBR materials system
.\build\Release\reactor_frame_graph.exe     # FrameGraph render passes
.\build\Release\reactor_fps_controller.exe  # FPS controller + physics
.\build\Release\reactor_lighting.exe        # Multi-light showcase
.\build\Release\reactor_telemetry.exe       # GPU stats + telemetry
.\build\Release\reactor_play_mode.exe       # Play-in-editor bridge
.\build\Release\reactor_multi_object.exe    # 225 objects scene

# Linux/macOS
./build/reactor_3d
./build/reactor_ecs_scene
./build/reactor_lighting
# ... etc
```

### Ejemplo C++ Mínimo

```cpp
#include <reactor/application.hpp>
using namespace reactor;

class MiJuego : public Application {
    float rotacion = 0.0f;

    Config config() override {
        return Config("Mi Juego C++", 1920, 1080).with_msaa(4);
    }

    void on_init() override {
        Camera::set_position({0, 2, 5});
        Lighting::add_directional({-0.5f, -1, -0.3f}, {1, 1, 1}, 1.0f);
    }

    void on_update(float dt) override {
        rotacion += dt;
        if (Input::key_pressed(Input::KEY_ESCAPE())) Window::request_close();
    }
};

int main() { return MiJuego().run(); }
```

---

## 📁 Estructura de Archivos

```text
REACTOR-Framework-for-Vulkan-/
├── image.svg               # Logo REACTOR (Salazar-interactive)
├── README.md               # Documentación principal
├── HOW_BUILD.md            # Esta guía
├── Cargo.toml              # Proyecto Rust (v1.0.5)
│
├── src/                    # Rust Core
│   ├── lib.rs              # Exports + Prelude
│   ├── reactor.rs          # Vulkan rendering
│   ├── core/               # VulkanContext, Device, Allocator
│   ├── graphics/           # Swapchain, Pipeline, MSAA, Depth
│   ├── raytracing/         # RT Context, BLAS/TLAS, Pipeline
│   ├── compute/            # ComputePipeline, Dispatch
│   ├── resources/          # Mesh, Material, Texture, Vertex
│   ├── systems/            # Input, ECS, Scene, Camera
│   └── utils/              # GPUDetector, Time
│
├── examples/               # Ejemplos Rust (5)
│   ├── cube.rs             # ← EMPIEZA AQUÍ
│   ├── textured_cube.rs    # Cubo con textura
│   ├── sandbox.rs          # Sandbox experimental
│   ├── physics_camera.rs   # Cámara con física
│   └── obj_loader_demo.rs  # Carga de modelos OBJ
│
├── shaders/                # Shaders GLSL + SPIR-V
│
├── cpp/                    # C++ SDK completo
│   ├── reactor_c_api/      # Rust → C ABI (3300+ líneas)
│   ├── reactor_cpp/        # C++ SDK headers (1477 líneas)
│   └── examples/3D/        # 9 ejemplos C++ (ver arriba)
│
├── docs/                   # Documentación
│
└── Editor-REACTOR/         # Editor visual (egui + egui_dock)
```

---

## ⚙️ Opciones de ReactorConfig

```rust
ReactorConfig::new("Título")
    // Ventana
    .with_size(1920, 1080)      // Resolución
    .with_fullscreen(true)       // Pantalla completa
    .with_resizable(true)        // Ventana redimensionable
    
    // Renderizado
    .with_vsync(true)            // Sincronización vertical
    .with_msaa(4)                // Anti-aliasing (1, 2, 4, 8)
    .with_renderer(RendererMode::Forward)  // Forward, Deferred, RayTracing
    
    // Sistemas
    .with_physics_hz(60)         // Frecuencia de física
    .with_scene("assets/level.gltf")  // Auto-cargar escena
```

---

## 🎯 Callbacks Disponibles

```rust
impl ReactorApp for MiJuego {
    // REQUERIDOS
    fn config(&self) -> ReactorConfig;
    fn init(&mut self, ctx: &mut ReactorContext);
    fn update(&mut self, ctx: &mut ReactorContext);
    
    // OPCIONALES
    fn render(&mut self, ctx: &mut ReactorContext) { /* automático */ }
    fn shutdown(&mut self) { /* cleanup */ }
    fn on_resize(&mut self, width: u32, height: u32) { /* resize */ }
}
```

---

## 🐛 Solución de Problemas

### "Vulkan not found"
```bash
# Verificar que Vulkan SDK está instalado
vulkaninfo

# En Windows, verificar variable de entorno
echo %VULKAN_SDK%
```

### "No suitable GPU found"
- Asegúrate de tener drivers actualizados
- Verifica que tu GPU soporta Vulkan 1.3

### "Shader compilation failed"
```bash
# Recompilar shaders
cd shaders
glslc shader.vert -o vert.spv
glslc shader.frag -o frag.spv
```

### "Validation layer errors"
- Los errores de validación solo aparecen en debug builds
- Son útiles para debugging pero no afectan release builds

---

## 📚 Recursos

- [Vulkan Tutorial](https://vulkan-tutorial.com/)
- [Ash (Rust Vulkan bindings)](https://github.com/ash-rs/ash)
- [REACTOR Issues](https://github.com/user/REACTOR-Framework-for-Vulkan-/issues)

---

## 📄 Licencia

MIT License — Usa REACTOR para lo que quieras.
