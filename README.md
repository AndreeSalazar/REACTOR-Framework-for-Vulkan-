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
- **B (Reactor)**: Abstracciones seguras con RAII automático.
- **C (Game)**: API de alto nivel para lógica de juego.

## ✨ Características

| Módulo | Características |
|--------|-----------------|
| **Core** | VulkanContext, Device, Allocator, CommandManager |
| **Graphics** | Swapchain, Pipeline, RenderPass, Framebuffer, Buffer, Image, Sampler, Descriptors, DepthBuffer, MSAA |
| **Ray Tracing** | RayTracingContext, AccelerationStructure, RayTracingPipeline, ShaderBindingTable |
| **Compute** | ComputePipeline, ComputeDispatch |
| **Resources** | Mesh, Material, Texture, Vertex, Model |
| **Systems** | Input, ECS (World/Entity/Component), Scene, Camera, Transform |
| **Utils** | GPUDetector, CPUDetector, ResolutionDetector, Time |

## 🚀 Quick Start

### Requisitos
- [Rust](https://rustup.rs/) (1.70+)
- Vulkan SDK (1.3+)

### Ejecutar Sandbox
```bash
cargo run --example sandbox
```

### Código de Ejemplo (Layer C)

```rust
use reactor::prelude::*;
use winit::event_loop::EventLoop;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    // ... crear ventana ...
    
    // Una línea para inicializar TODO Vulkan
    let reactor = Reactor::init(&window).expect("Failed to init Vulkan");
    
    // Crear recursos fácilmente
    let mesh = reactor.create_mesh(&vertices, &indices)?;
    let material = reactor.create_material(&vert_spv, &frag_spv)?;
    
    // Renderizar escena
    reactor.draw_scene(&scene, &view_projection)?;
}
```

## 📁 Estructura del Proyecto

```
src/
├── core/           # Núcleo de Vulkan
│   ├── context.rs      # VulkanContext (Instance, Device, Queue)
│   ├── device.rs       # DeviceInfo
│   ├── allocator.rs    # MemoryAllocator (gpu-allocator)
│   ├── command.rs      # CommandManager
│   └── surface.rs      # SurfaceInfo
│
├── graphics/       # Renderizado
│   ├── swapchain.rs    # Swapchain
│   ├── pipeline.rs     # Graphics Pipeline + Config
│   ├── render_pass.rs  # RenderPass + Config
│   ├── framebuffer.rs  # Framebuffer + FramebufferSet
│   ├── buffer.rs       # Buffer (Vertex, Index, Uniform, Storage)
│   ├── image.rs        # Image + Transitions
│   ├── sampler.rs      # Sampler + Config
│   ├── descriptors.rs  # DescriptorPool, Layout, Set
│   ├── depth.rs        # DepthBuffer
│   └── msaa.rs         # MSAA Target
│
├── raytracing/     # Ray Tracing (RTX)
│   ├── context.rs              # RayTracingContext
│   ├── acceleration_structure.rs # BLAS/TLAS
│   ├── pipeline.rs             # RT Pipeline
│   └── shader_binding_table.rs # SBT
│
├── compute/        # Compute Shaders
│   ├── pipeline.rs     # ComputePipeline
│   └── dispatch.rs     # ComputeDispatch + Barriers
│
├── resources/      # Assets del Juego
│   ├── vertex.rs       # Vertex, VertexPBR, InstanceData
│   ├── mesh.rs         # Mesh + Primitives (Cube, Quad)
│   ├── material.rs     # Material + MaterialBuilder
│   ├── texture.rs      # Texture + Mipmaps
│   └── model.rs        # Model + ModelBatch
│
├── systems/        # Sistemas de Juego
│   ├── input.rs        # Input (Keyboard, Mouse)
│   ├── ecs.rs          # World, Entity, Component
│   ├── scene.rs        # Scene + SceneObject
│   ├── camera.rs       # Camera (3D/2D)
│   └── transform.rs    # Transform
│
├── utils/          # Utilidades
│   ├── gpu_detector.rs       # GPUDetector + GPUInfo
│   ├── cpu_detector.rs       # CPUDetector + CPUInfo
│   ├── resolution_detector.rs # ResolutionDetector
│   └── time.rs               # Time + FixedTimestep
│
├── lib.rs          # Exports + Prelude
└── reactor.rs      # Fachada principal
```

## 🎮 Uso Avanzado

### Prelude (Importar todo lo común)
```rust
use reactor::prelude::*;
```

### Crear Material con Config
```rust
let material = MaterialBuilder::new(vert_code, frag_code)
    .no_cull()
    .blend()
    .build(&ctx, render_pass, width, height)?;
```

### Sistema ECS
```rust
let mut world = World::new();
let entity = world.create_entity();
world.add_component(entity, Transform::from_position(Vec3::ZERO));
world.add_component(entity, Velocity { x: 1.0, y: 0.0 });

for (entity, transform) in world.query::<Transform>() {
    // ...
}
```

### Cámara 3D
```rust
let camera = Camera::perspective(45.0, aspect, 0.1, 1000.0)
    .look_at(eye, target, Vec3::Y);
let vp = camera.view_projection_matrix();
```

## 📊 Comparación: Vulkan Puro vs REACTOR

| Tarea | Vulkan Puro | REACTOR |
|-------|-------------|---------|
| Inicializar Vulkan | ~300 líneas | 1 línea |
| Crear Buffer | ~50 líneas | 1 línea |
| Crear Pipeline | ~200 líneas | 1 línea |
| Renderizar Escena | ~100 líneas | 1 línea |
| **Total típico** | **800-1500 líneas** | **~50 líneas** |

## 📄 Licencia
MIT License
