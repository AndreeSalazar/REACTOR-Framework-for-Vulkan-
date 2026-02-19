# REACTOR Framework — Guía de Desarrollo en Rust

**Versión 1.0.5** | Para desarrolladores Rust | Powered by Salazar-interactive

## Introducción

Esta guía te enseña a crear juegos con REACTOR usando Rust directamente.

## Requisitos

- Rust 1.70+
- Vulkan SDK 1.3+
- GPU compatible con Vulkan

## Configuracion del Proyecto

### Cargo.toml

```toml
[package]
name = "mi-juego"
version = "0.1.0"
edition = "2021"

[dependencies]
reactor = { path = "../REACTOR-Framework-for-Vulkan-" }
```

## Patron ReactorApp

El patron principal es implementar el trait `ReactorApp`:

```rust
use reactor::prelude::*;

struct MiJuego {
    rotacion: f32,
    velocidad: f32,
}

impl Default for MiJuego {
    fn default() -> Self {
        Self {
            rotacion: 0.0,
            velocidad: 1.0,
        }
    }
}

impl ReactorApp for MiJuego {
    fn config(&self) -> ReactorConfig {
        ReactorConfig::new("Mi Juego")
            .with_size(1920, 1080)
            .with_vsync(true)
            .with_msaa(4)
    }

    fn init(&mut self, ctx: &mut ReactorContext) {
        // Configurar camara
        ctx.camera.position = Vec3::new(0.0, 2.0, 5.0);
        ctx.camera.look_at(Vec3::ZERO);
        
        // Agregar luz
        ctx.lighting.add_light(Light::directional(
            Vec3::new(-0.5, -1.0, -0.3),
            Vec3::ONE,
            1.0,
        ));
    }

    fn update(&mut self, ctx: &mut ReactorContext) {
        self.rotacion += ctx.time.delta() * self.velocidad;
        
        // Input
        if ctx.input.key_pressed(KeyCode::Escape) {
            ctx.request_close();
        }
        
        if ctx.input.key_down(KeyCode::Space) {
            self.velocidad = 3.0;
        } else {
            self.velocidad = 1.0;
        }
    }
}

fn main() {
    reactor::run(MiJuego::default());
}
```

## Sistemas Disponibles

### Camara

```rust
// Perspectiva
let camera = Camera::perspective(60.0, aspect, 0.1, 1000.0);

// Posicion y orientacion
camera.position = Vec3::new(0.0, 5.0, 10.0);
camera.look_at(Vec3::ZERO);

// Controles FPS
camera.rotate_yaw(mouse_dx * sensitivity);
camera.rotate_pitch(mouse_dy * sensitivity);
camera.move_forward(speed * dt);
```

### Iluminacion

```rust
let mut lighting = LightingSystem::new();

// Luz direccional (sol)
lighting.add_light(Light::directional(
    Vec3::new(-0.5, -1.0, -0.3),
    Vec3::new(1.0, 0.98, 0.95),
    1.0,
));

// Luz puntual
lighting.add_light(Light::point(
    Vec3::new(0.0, 5.0, 0.0),
    Vec3::new(1.0, 0.8, 0.6),
    2.0,  // intensidad
    20.0, // rango
));

// Spotlight
lighting.add_light(Light::spot(
    Vec3::new(0.0, 10.0, 0.0),
    Vec3::NEG_Y,
    Vec3::ONE,
    5.0,  // intensidad
    30.0, // rango
    45.0, // angulo
));
```

### Input

```rust
// Teclas
if ctx.input.key_pressed(KeyCode::Space) { /* una vez */ }
if ctx.input.key_down(KeyCode::W) { /* mientras presionado */ }

// Mouse
let (mx, my) = ctx.input.mouse_position();
let (dx, dy) = ctx.input.mouse_delta();
if ctx.input.mouse_button_down(MouseButton::Left) { /* click */ }
```

### Escena

```rust
// Crear mesh
let cube_mesh = Mesh::cube(&ctx.vulkan, &ctx.allocator)?;

// Crear material
let material = Material::new(&ctx.vulkan, render_pass, vert_spv, frag_spv)?;

// Agregar a escena
ctx.scene.add_object(Arc::new(cube_mesh), Arc::new(material), Mat4::IDENTITY);

// Modificar transform
ctx.scene.objects[0].transform = Mat4::from_rotation_y(rotacion);
```

### Primitivas

```rust
let (vertices, indices) = Primitives::cube();
let (vertices, indices) = Primitives::sphere(32, 16);
let (vertices, indices) = Primitives::plane(10);
let (vertices, indices) = Primitives::cylinder(32, 2.0, 0.5);
```

### Fisica

```rust
let mut physics = PhysicsWorld::new();
physics.gravity = Vec3::new(0.0, -9.81, 0.0);

// Raycasting
let ray = Ray::new(origin, direction);
if let Some(t) = ray.intersects_aabb(&aabb) {
    let hit_point = ray.point_at(t);
}
```

### Post-Processing

```rust
let post = PostProcessPipeline::with_preset(PostProcessPreset::Cinematic);

// Manual
let mut settings = PostProcessSettings::default();
settings.enable_effect(PostProcessEffect::Bloom);
settings.bloom_intensity = 0.5;
```

## ADead-GPU

### ISR (Intelligent Shading Rate)

```rust
let mut isr = IntelligentShadingRate::new(1920, 1080);
isr.config = IntelligentShadingRate::preset_performance();

let importance = isr.calculate_importance(world_pos, normal, prev_pos, camera_pos, sdf);
let pixel_size = isr.get_adaptive_pixel_size(screen_x, screen_y);
```

### SDF Anti-Aliasing

```rust
let aa = SDFAntiAliasing::new();
let alpha = aa.compute_aa(sdf_value, screen_derivative);
```

## Ejemplos

Ejecutar ejemplos incluidos:

```bash
cargo run --example cube              # Cubo 3D con controles
cargo run --example textured_cube     # Cubo con textura
cargo run --example sandbox           # Sandbox experimental
cargo run --example physics_camera    # Cámara con física
cargo run --example obj_loader_demo   # Carga de modelos OBJ
```

## Estructura de Proyecto Recomendada

```text
mi-juego/
  Cargo.toml
  src/
    main.rs
    game.rs
    player.rs
    world.rs
  assets/
    shaders/
    textures/
    models/
```

## Tips

1. **Usa `prelude::*`** para importar todo lo común
2. **RAII automático** — no necesitas limpiar recursos manualmente
3. **Delta time** — siempre multiplica movimiento por `ctx.time.delta()`
4. **Ownership** — Rust garantiza seguridad de memoria
5. **C++ interop** — también puedes usar REACTOR desde C++ vía el C ABI (ver `docs/cpp-guide.md`)

## Licencia

MIT License — **Powered by Salazar-interactive**
