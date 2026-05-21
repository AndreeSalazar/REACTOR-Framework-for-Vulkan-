# 🔧 How to Build — Guía de Construcción REACTOR (Rust puro)

> **REACTOR v1.1.0+ es 100 % Rust.** No requiere C, C++, CMake ni vcpkg.

## 📋 Requisitos Previos

### Windows
```powershell
# 1. Instalar Rust
winget install Rustlang.Rustup

# 2. Instalar Vulkan SDK
# https://vulkan.lunarg.com/sdk/home
winget install KhronosGroup.VulkanSDK

# 3. Verificar
rustc --version    # 1.70+
cargo --version
vulkaninfo
```

### Linux (Ubuntu/Debian)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
sudo apt install vulkan-tools libvulkan-dev vulkan-validationlayers
vulkaninfo
```

### macOS
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
brew install molten-vk
vulkaninfo
```

---

## 🚀 Construcción Rápida

```bash
git clone https://github.com/user/REACTOR-Framework-for-Vulkan-.git
cd REACTOR-Framework-for-Vulkan-

cargo run --example cube              # Debug
cargo run --example cube --release    # Release (recomendado)
```

### Compilar Librería
```bash
cargo build              # Debug
cargo build --release    # Release
```

Los shaders se compilan automáticamente vía `build.rs` (GLSL → SPIR-V).

---

## 🎮 Crear Tu Propio Juego

### Opción A — Forma SUPER corta (1 archivo, 1 función)

```rust
use reactor::prelude::*;

fn main() {
    reactor::quick("Mi Juego", 1280, 720, |ctx| {
        // update cada frame — listo
        ctx.camera.position.x = (ctx.time.elapsed()).sin() * 5.0;
    });
}
```

### Opción B — Trait `ReactorApp` (control total)

```rust
use reactor::prelude::*;

struct MiJuego { rotacion: f32 }

impl ReactorApp for MiJuego {
    fn config(&self) -> ReactorConfig {
        ReactorConfig::new("Mi Juego")
            .with_size(1920, 1080)
            .with_vsync(true)
            .with_msaa(4)
    }

    fn init(&mut self, ctx: &mut ReactorContext) {
        ctx.camera.position = Vec3::new(0.0, 2.0, 4.0);
    }

    fn update(&mut self, ctx: &mut ReactorContext) {
        self.rotacion += ctx.time.delta();
    }
}

fn main() { reactor::run(MiJuego { rotacion: 0.0 }); }
```

### Opción C — Macro `reactor::game!` (mínimo absoluto)

```rust
use reactor::prelude::*;

reactor::game! {
    title: "Mi Juego",
    size: (1280, 720),
    update: |ctx| {
        ctx.camera.position.x = ctx.time.elapsed().sin() * 5.0;
    }
}
```

### Opción D — Proyecto Separado

```bash
cargo new mi_juego
cd mi_juego
```

`Cargo.toml`:
```toml
[dependencies]
reactor = { path = "../REACTOR-Framework-for-Vulkan-" }
```

`src/main.rs` — usa cualquiera de las opciones A, B o C.

---

## 📁 Estructura de Archivos

```text
REACTOR-Framework-for-Vulkan-/
├── README.md               # Documentación principal
├── HOW_BUILD.md            # Esta guía
├── Fases.md                # Roadmap completo del SDK
├── Cargo.toml              # Crate principal
│
├── src/                    # Rust Core (lib.rs + módulos)
├── examples/               # 5 demos Rust
├── shaders/                # GLSL + SPIR-V autocompilados
├── assets/                 # Modelos y texturas
├── docs/                   # Manual + guías + Tareas
└── Editor-REACTOR/         # Editor visual (egui)
```

---

## ⚙️ Opciones de `ReactorConfig`

```rust
ReactorConfig::new("Título")
    .with_size(1920, 1080)
    .with_fullscreen(true)
    .with_resizable(true)
    .with_vsync(true)
    .with_msaa(4)                              // 1, 2, 4, 8
    .with_renderer(RendererMode::Forward)      // Forward | Deferred | RayTracing
    .with_physics_hz(60)
    .with_scene("assets/level.gltf")
```

---

## 🎯 Callbacks de `ReactorApp`

```rust
impl ReactorApp for MiJuego {
    fn config(&self) -> ReactorConfig { ... }    // opcional
    fn init(&mut self, ctx: &mut ReactorContext);
    fn update(&mut self, ctx: &mut ReactorContext);

    fn render(&mut self, ctx: &mut ReactorContext) { ctx.render_scene(); }
    fn fixed_update(&mut self, ctx: &mut ReactorContext, dt: f32) {}
    fn on_resize(&mut self, ctx: &mut ReactorContext, w: u32, h: u32) {}
    fn on_exit(&mut self, ctx: &mut ReactorContext) {}
}
```

---

## 🐛 Solución de Problemas

### "Vulkan not found"
```powershell
vulkaninfo
echo %VULKAN_SDK%   # Windows
```

### "No suitable GPU found"
- Drivers actualizados + GPU compatible con Vulkan 1.3.

### "Shader compilation failed"
```bash
cd shaders
glslc shader.vert -o vert.spv
glslc shader.frag -o frag.spv
```
> Normalmente `build.rs` lo hace solo en cada `cargo build`.

### Validation layer errors
- Aparecen sólo en debug. No afectan release.

---

## 📚 Recursos

- [Vulkan Tutorial](https://vulkan-tutorial.com/)
- [Ash (Rust Vulkan bindings)](https://github.com/ash-rs/ash)
- [Fases del SDK](Fases.md)
- [Manual general](docs/manual.md)
- [Guía Rust](docs/rust-guide.md)

---

## 📄 Licencia

MIT — Powered by Salazar-interactive.
