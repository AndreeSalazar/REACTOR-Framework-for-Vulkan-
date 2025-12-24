# 🚀 REACTOR (Rust Edition) - Zero-overhead Vulkan Framework

**El Framework de Desarrollo de Juegos más Seguro y Fácil con Vulkan, ahora en Rust**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Vulkan](https://img.shields.io/badge/Vulkan-1.3-red.svg)](https://www.vulkan.org/)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)

**REACTOR** simplifica Vulkan usando el sistema de tipos y ownership de Rust para ofrecer **seguridad de memoria garantizada** y **zero-overhead**.

## 🏗️ Arquitectura A → B → C

```
A (Vulkan/Ash) → B (Reactor) → C (Game)
  Unsafe           Safe           Simple
  Raw bindings     RAII wrappers  ECS / Components
```

- **A (Ash)**: Bindings directos a Vulkan (`unsafe`).
- **B (Reactor)**: Abstracciones seguras (`VulkanContext`, `Device`, `Buffer`). RAII se encarga de `vkDestroy*`.
- **C (Game)**: API de alto nivel para lógica de juego.

## ✨ Ventajas de Rust
- **Memory Safety**: Olvídate de los segfaults y memory leaks de C++.
- **RAII Automático**: Los recursos de Vulkan se liberan automáticamente cuando salen de scope.
- **Cargo**: Gestión de dependencias (ash, winit, shaderc) sin dolor.

## 🚀 Quick Start

### Requisitos
- [Rust](https://rustup.rs/) (instalado)
- Vulkan SDK (instalado y configurado en PATH)

### Ejecutar Sandbox
```bash
cargo run --example sandbox
```

### Código de Ejemplo (Layer C)

```rust
use reactor::Reactor;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    
    // Inicialización segura de Vulkan (Instance, Device, Queue)
    let _reactor = Reactor::init(&window).expect("Failed to init Vulkan");

    event_loop.run(move |event, _, control_flow| {
        // Game loop...
    });
}
```

## � Estructura del Proyecto
- `src/lib.rs`: Punto de entrada de la librería.
- `src/vulkan_context.rs`: Inicialización de Vulkan (Instance, PhysicalDevice, Device).
- `src/reactor.rs`: Fachada principal del framework.
- `examples/`: Ejemplos de uso.

## 📄 Licencia
MIT License
