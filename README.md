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
| **Core** | VulkanContext, Device, Allocator, CommandManager, Surface |
| **Graphics** | Swapchain, Pipeline, RenderPass, Framebuffer, Buffer, Image, Sampler, Descriptors, DepthBuffer, MSAA, **UniformBuffers**, **DebugRenderer**, **PostProcessing** |
| **Ray Tracing** | RayTracingContext, AccelerationStructure, RayTracingPipeline, ShaderBindingTable |
| **Compute** | ComputePipeline, ComputeDispatch, Barriers |
| **Resources** | Mesh, Material, Texture, Vertex, Model, **Primitives** (Cube, Sphere, Plane, Cylinder, Cone, Torus) |
| **Systems** | Input, ECS, Scene, Camera, Transform, **Lighting**, **Physics**, **FrustumCulling**, **Animation**, **Particles**, **Audio** |
| **Utils** | GPUDetector, CPUDetector, ResolutionDetector, Time, FixedTimestep |

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

### Sistema de Iluminación
```rust
let mut lighting = LightingSystem::with_sun();

// Agregar luz puntual
lighting.add_light(Light::point(
    Vec3::new(0.0, 5.0, 0.0),  // posición
    Vec3::new(1.0, 0.8, 0.6),  // color
    2.0,                        // intensidad
    20.0,                       // rango
));

// Agregar spotlight
lighting.add_light(Light::spot(
    Vec3::new(0.0, 10.0, 0.0), // posición
    Vec3::NEG_Y,               // dirección
    Vec3::ONE,                 // color
    5.0,                       // intensidad
    30.0,                      // rango
    45.0,                      // ángulo
));
```

### Sistema de Partículas
```rust
// Efecto de fuego predefinido
let mut fire = ParticleSystem::fire();
fire.position = Vec3::new(0.0, 0.0, 0.0);

// Explosión
let mut explosion = ParticleSystem::explosion();
explosion.play();

// Sistema personalizado
let config = ParticleSystemConfig {
    emission_rate: 100.0,
    lifetime: RandomRange::new(1.0, 2.0),
    start_color: Vec4::new(0.0, 0.5, 1.0, 1.0),
    ..Default::default()
};
let custom = ParticleSystem::new(config);
```

### Animaciones y Tweens
```rust
// Tween simple
let mut tween = Tween::new(0.0, 100.0, 2.0)
    .with_easing(EasingFunction::EaseOutElastic);

// En el loop
let value = tween.update(delta_time);
if tween.is_finished() { /* ... */ }

// Sistema de animación completo
let mut player = AnimationPlayer::new();
player.add_clip(walk_animation);
player.play("walk");
let sample = player.update(delta_time);
sample.apply_to_transform(&mut transform);
```

### Física y Colisiones
```rust
// Crear mundo físico
let mut physics = PhysicsWorld::new();
physics.gravity = Vec3::new(0.0, -9.81, 0.0);

// Cuerpo rígido
let mut body = RigidBody::default();
body.add_force(Vec3::new(100.0, 0.0, 0.0));

// Raycasting
let ray = Ray::from_screen(mouse_x, mouse_y, width, height, inv_vp);
if let Some(t) = ray.intersects_aabb(&aabb) {
    let hit_point = ray.point_at(t);
}

// Frustum Culling
let mut culling = CullingSystem::new();
culling.update_frustum(view_projection);
for object in &scene.objects {
    if culling.is_visible_aabb(&object.bounds) {
        // Renderizar
    }
}
println!("Culled: {:.1}%", culling.cull_percentage());
```

### Post-Processing
```rust
// Preset cinematográfico
let post = PostProcessPipeline::with_preset(PostProcessPreset::Cinematic);

// Configuración manual
let mut settings = PostProcessSettings::default();
settings.enable_effect(PostProcessEffect::Bloom);
settings.enable_effect(PostProcessEffect::Vignette);
settings.bloom_intensity = 0.5;
settings.vignette_intensity = 0.3;
```

### Debug Renderer
```rust
let mut debug = DebugRenderer::new();

// Dibujar líneas
debug.line(start, end, Vec4::new(1.0, 0.0, 0.0, 1.0));

// Dibujar AABB
debug.aabb(&DebugAABB { min, max }, Vec4::ONE);

// Dibujar ejes
debug.axes(origin, 1.0);

// Dibujar grid
debug.grid(Vec3::ZERO, 10.0, 10, Vec4::new(0.5, 0.5, 0.5, 1.0));

// Dibujar frustum de cámara
debug.frustum(inv_view_proj, Vec4::new(1.0, 1.0, 0.0, 1.0));
```

### Primitivas Geométricas
```rust
// Generar meshes procedurales
let (vertices, indices) = Primitives::cube();
let (vertices, indices) = Primitives::sphere(32, 16);
let (vertices, indices) = Primitives::plane(10);
let (vertices, indices) = Primitives::cylinder(32, 2.0, 0.5);
let (vertices, indices) = Primitives::cone(32, 2.0, 0.5);
let (vertices, indices) = Primitives::torus(32, 16, 1.0, 0.3);
```

### Cámara 3D
```rust
let camera = Camera::perspective(45.0, aspect, 0.1, 1000.0)
    .look_at(eye, target, Vec3::Y);
let vp = camera.view_projection_matrix();

// Controles FPS
camera.rotate_yaw(mouse_delta.x * sensitivity);
camera.rotate_pitch(mouse_delta.y * sensitivity);
camera.move_forward(speed * delta);
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
