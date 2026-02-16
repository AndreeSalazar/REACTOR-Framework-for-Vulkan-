# REACTOR Framework — Tareas para v0.5.0

## 🎯 Objetivo Principal
**ReactorApp() ONE CALL** — Una sola llamada para inicializar todo el engine.
REACTOR = React-Like pero para Vulkan. Heredas, overrideas, modificas desde un solo archivo.

Arquitectura: `Rust Core` → `C ABI (extern "C")` → `C++ SDK` → `Usuario hereda y modifica`

---

## 🎨 TARGET API — Lo que queremos lograr

### Rust — Builder pattern + Trait (IMPLEMENTADO ✅)
```rust
use reactor::prelude::*;

struct MyGame { rotation: f32 }

impl ReactorApp for MyGame {
    fn config(&self) -> ReactorConfig {
        ReactorConfig::new("Mi Juego")
            .with_size(1920, 1080)
            .with_vsync(true)
            .with_renderer(RendererMode::RayTracing)  // ← NUEVO
            .with_scene("assets/level1.gltf")         // ← NUEVO (auto-load)
    }

    fn init(&mut self, ctx: &mut ReactorContext) {
        ctx.camera.position = Vec3::new(0.0, 2.0, 4.0);
        // Scene ya cargada por with_scene() ^
    }

    fn update(&mut self, ctx: &mut ReactorContext) {
        self.rotation += ctx.delta() * 1.5;
        ctx.scene.objects[0].transform = Mat4::from_rotation_y(self.rotation);
    }
    // render() automático — no necesitas override
}

fn main() { reactor::run(MyGame { rotation: 0.0 }); }
```

### C++ — Herencia + Designated Initializers (PENDIENTE ❌)
```cpp
#include <reactor/reactor.hpp>

class MyGame : public reactor::Application {
    float rotation = 0.0f;

    Config config() override {
        return Config("Mi Juego")
            .with_size(1920, 1080)
            .with_vsync(true)
            .with_renderer(Renderer::RayTracing)  // ← FALTA
            .with_scene("assets/level1.gltf");    // ← FALTA
    }

    void on_init() override {
        Camera::set_position({0, 2, 4});
    }

    void on_update(float dt) override {
        rotation += dt * 1.5f;
        Scene::set_transform(0, Mat4::RotationY(rotation));
    }
};

int main() { MyGame().run(); }
```

### C++ Ultra-Simple — Lambda ONE CALL (PENDIENTE ❌)
```cpp
// Sin clase, sin herencia — UNA LLAMADA
ReactorApp({
    .title = "Mi Juego",
    .resolution = {1920, 1080},
    .vsync = true,
    .renderer = RayTracing,
    .scene = "assets/level1.gltf"
});
```

### Python / C# / Cualquier lenguaje — Via C ABI (FUTURO)
```python
import reactor
reactor.run("Mi Juego", width=1920, height=1080, scene="assets/level1.gltf")
```

---

## 📋 Plan de Ejecución — Separado por Capa

---

### 🦀 PARTE 1: RUST CORE (src/)
> El engine real. Todo lo unsafe, Vulkan, RAII, ownership.
> C++ NO puede hacer esto: safety garantizada, zero-cost abstractions, ownership model.

#### **FASE 1: Estabilidad Core (CRÍTICO)**
| # | Tarea | Estado | Archivo(s) | Descripción |
|---|-------|--------|------------|-------------|
| R1 | Vulkan cleanup | ✅ Completado | `reactor.rs` | Fix MSAA destruction, device_wait_idle |
| R2 | Validation Layers | ✅ Completado | `core/context.rs` | `VK_LAYER_KHRONOS_validation` en debug builds, Debug messenger callback |
| R3 | Error Handling | ✅ Completado | `src/core/error.rs` | `ReactorError` enum, `Result<T, ReactorError>`, C ABI + C++ SDK |
| R4 | Ejemplo cube.rs | ✅ Completado | `examples/cube.rs` | Renderiza correctamente |
| R5a | ReactorConfig completo | ✅ Completado | `app.rs` | `vsync`, `fullscreen`, `msaa`, `renderer`, `scene` — builder pattern |

#### **FASE 2: Renderizado — Lo que Rust hace único**
| # | Tarea | Estado | Archivo(s) | Descripción |
|---|-------|--------|------------|-------------|
| R5 | Depth Buffer | ✅ Completado | `reactor.rs`, `pipeline.rs` | Z-buffer en render pass + framebuffers + depth testing |
| R6 | Texturas | ✅ Completado | `resources/texture.rs` | PNG/JPG → VkImage, samplers, mipmaps, `from_file()`, `from_bytes()` |
| R7 | Material con Texturas | ✅ Completado | `material.rs`, `pipeline.rs` | `create_textured_material()` con descriptor sets, shaders texture.vert/frag |
| R8 | Render Pass configurable | 🟡 Pendiente | `graphics/render_pass.rs` | Forward rendering con depth + MSAA integrados |

#### **FASE 3: Assets — Ownership de Rust protege recursos GPU**
| # | Tarea | Estado | Archivo(s) | Descripción |
|---|-------|--------|------------|-------------|
| R9 | OBJ Loader | ✅ Completado | `resources/model.rs` | `ObjData::load()`, normals, UVs, triangulación |
| R10 | glTF 2.0 | ✅ Completado | `resources/model.rs` | `GltfData::load()`, meshes, normals, UVs |
| R11 | Asset Manager | 🟢 Pendiente | `resources/asset_manager.rs` (nuevo) | Cache, deduplicación, async loading |

#### **FASE 4: Sistemas de Juego — Rust = safe multithreading**
| # | Tarea | Estado | Archivo(s) | Descripción |
|---|-------|--------|------------|-------------|
| R12 | Physics funcional | ✅ Completado | `systems/physics.rs` | `CharacterController`, AABB, Sphere, Ray, collision detection |
| R13 | ECS funcional | 🟡 Pendiente | `systems/ecs.rs` | World.query(), sistemas iterando componentes |
| R14 | Animation funcional | 🟡 Pendiente | `systems/animation.rs` | AnimationPlayer integrado con Scene |
| R15 | Audio funcional | 🟢 Pendiente | `systems/audio.rs` | Backend real (rodio/cpal) |

#### **FASE 5: Vulkan Avanzado — Solo posible desde Rust (unsafe controlado)**
| # | Tarea | Estado | Archivo(s) | Descripción |
|---|-------|--------|------------|-------------|
| R16 | Shadow Mapping | 🟢 Pendiente | `graphics/shadows.rs` (nuevo) | Depth pass separado, shadow map sampler |
| R17 | Post-Processing | 🟢 Pendiente | `graphics/post_process.rs` | Bloom, tone mapping como render passes reales |
| R18 | PBR Materials | 🟢 Pendiente | `resources/material.rs` | Metallic-roughness, IBL |
| R19 | Compute Shaders | 🟢 Pendiente | `compute/` | Partículas GPU, physics GPU |

---

### 🔗 PARTE 2: C ABI (cpp/reactor_c_api/)
> La frontera universal. `extern "C"` = cualquier lenguaje puede usar REACTOR.
> Cada función Rust se expone aquí como `reactor_*()` con tipos `repr(C)`.

#### **YA EXPUESTO ✅**
| Categoría | Funciones | Estado |
|-----------|-----------|--------|
| Lifecycle | `reactor_run`, `reactor_run_simple` | ✅ |
| Timing | `reactor_get_delta_time`, `_fps`, `_total_time`, `_frame_count` | ✅ |
| Window | `reactor_get_width`, `_height`, `_aspect_ratio`, `_should_close` | ✅ |
| Input | `reactor_key_down`, `_pressed`, `_mouse_*` (16 funciones) | ✅ |
| Camera | `reactor_set_camera_position`, `_target`, `_get_view_projection` | ✅ |
| Scene | `reactor_add_object`, `_set_transform`, `_clear_scene` (11 funciones) | ✅ |
| Mesh | `reactor_create_cube`, `_create_mesh`, `_destroy_mesh` | ✅ |
| Lighting | `reactor_add_directional_light`, `_point`, `_spot`, `_clear` | ✅ |
| Math | `reactor_mat4_*`, `reactor_vec3_*` (20+ funciones) | ✅ |
| SDF | `reactor_sdf_sphere`, `_box`, `_cylinder`, etc. | ✅ |
| Utils | `reactor_lerp`, `_clamp`, `_smoothstep`, `_log_*` | ✅ |

#### **FALTA EXPONER ❌**
| # | Tarea | Estado | Función C ABI | Depende de |
|---|-------|--------|---------------|------------|
| A0 | CConfig completo | ✅ Completado | `CConfig.renderer`, `.scene`, `CRendererMode` enum | R5a |
| A1 | Error handling | ✅ Completado | `reactor_get_last_error()`, `reactor_error_message()` | R3 |
| A2 | Material creation | ✅ Completado | `reactor_create_material(shader_vert, shader_frag)` | R7 |
| A3 | Texture loading | ✅ Completado | `reactor_load_texture()`, `reactor_texture_width/height()`, `reactor_destroy_texture()` | R6 |
| A4 | Model loading | 🟡 Pendiente | `reactor_load_model(path)`, `reactor_destroy_model()` | R9/R10 |
| A5 | Physics API | 🟡 Pendiente | `reactor_physics_step()`, `_add_rigidbody()`, `_raycast()` | R12 |
| A6 | ECS API | 🟡 Pendiente | `reactor_ecs_create_entity()`, `_add_component()`, `_query()` | R13 |
| A7 | Debug draw API | 🟡 Pendiente | `reactor_debug_line()`, `_debug_aabb()`, `_debug_grid()` | Ya existe en Rust |
| A8 | Animation API | 🟢 Pendiente | `reactor_animation_play()`, `_add_clip()`, `_update()` | R14 |
| A9 | Audio API | 🟢 Pendiente | `reactor_audio_play()`, `_load_sound()`, `_set_volume()` | R15 |
| A10 | Post-process API | 🟢 Pendiente | `reactor_set_post_process()`, `_enable_bloom()` | R17 |
| A11 | GPU Info | 🟡 Pendiente | `reactor_get_gpu_name()`, `_get_vram()`, `_get_msaa()` | Ya existe en Rust |

---

### ⚡ PARTE 3: C++ SDK (cpp/reactor_cpp/)
> Wrappers RAII con herencia. C++ es único por: templates, operator overloading, RAII destructors, STL.
> El usuario hereda de `reactor::Application` y overridea lo que necesite.

#### **YA IMPLEMENTADO ✅**
| Clase C++ | Archivo | Wrappea |
|-----------|---------|---------|
| `reactor::Application` | `application.hpp` | `reactor_run()` con callbacks |
| `ReactorApp()` función | `application.hpp` | One-call con lambdas |
| `reactor::Input` | `application.hpp` | `reactor_key_*`, `reactor_mouse_*` |
| `reactor::Time` | `application.hpp` | `reactor_get_delta_time`, etc. |
| `reactor::Window` | `application.hpp` | `reactor_get_width`, etc. |
| `reactor::Camera` | `application.hpp` | `reactor_set_camera_*` |
| `reactor::Scene` | `application.hpp` | `reactor_*_object_*` |
| `reactor::Lighting` | `application.hpp` | `reactor_add_*_light` |
| `reactor::SDF` | `application.hpp` | `reactor_sdf_*` |
| `reactor::Log` | `application.hpp` | `reactor_log_*` |
| `reactor::Config` | `application.hpp` | `CConfig` builder pattern |
| `Vec2/Vec3/Vec4/Mat4` | `types.hpp` | Operators, constructors, helpers |
| `Transform` | `types.hpp` | `matrix()`, `forward()`, `right()` |
| `Color` | `types.hpp` | `= Vec4` con presets |

#### **FALTA IMPLEMENTAR ❌**
| # | Clase C++ | Estado | Archivo | Wrappea C ABI |
|---|-----------|--------|---------|---------------|
| C0 | `Config` con `renderer`, `scene` | ✅ Completado | `application.hpp` | `CConfig` + `RendererMode` enum + `to_c()` |
| C1 | `reactor::Material` | ✅ Completado | `application.hpp` | `reactor_create_material()`, `from_shaders()`, `from_texture()` |
| C2 | `reactor::Texture` | ✅ Completado | `application.hpp` | RAII wrapper con `from_file()`, `solid()`, move semantics |
| C3 | `reactor::Model` / `ObjInfo` | ✅ Completado | `application.hpp` | `ObjInfo::load()`, `Mesh::cube/quad/plane()` |
| C4 | `reactor::Physics` | ✅ Completado | `application.hpp` | `CharacterController`, `Physics::raycast_aabb()`, collision tests |
| C5 | `reactor::ECS` / `Entity` | 🟡 Pendiente | `ecs.hpp` (nuevo) | `reactor_ecs_*` |
| C6 | `reactor::Debug` | ✅ Completado | `application.hpp` | `line()`, `wire_box()`, `wire_sphere()`, `grid()`, `axes()` |
| C7 | `reactor::Animation` | 🟢 Pendiente | `animation.hpp` (nuevo) | `reactor_animation_*` |
| C8 | `reactor::Audio` | 🟢 Pendiente | `audio.hpp` (nuevo) | `reactor_audio_*` |
| C9 | `reactor::Error` | 🔴 Pendiente | `application.hpp` | `reactor_get_last_error()` |
| C10 | `reactor::PostProcess` | 🟢 Pendiente | `application.hpp` | `reactor_set_post_process()` |
| C11 | `reactor::GPUInfo` | 🟡 Pendiente | `application.hpp` | `reactor_get_gpu_name()` |

---

## 🎯 Flujo Completo: ReactorApp() hereda TODO

```
USUARIO (hereda y modifica desde UN archivo):
┌─────────────────────────────────────────────────────┐
│  class MyGame : public reactor::Application {       │  ← C++
│      void on_init() override { ... }                │
│      void on_update(float dt) override { ... }      │
│  };                                                 │
│  int main() { MyGame().run("Mi Juego"); }           │
└──────────────────────┬──────────────────────────────┘
                       │ hereda automáticamente:
                       ▼
┌─────────────────────────────────────────────────────┐
│  reactor::Application (C++ SDK)                     │
│  ├── Input, Time, Window, Camera                    │  ← Ya listo ✅
│  ├── Scene, Lighting, SDF, Log                      │  ← Ya listo ✅
│  ├── Material, Texture, Model                       │  ← FALTA ❌
│  ├── Physics, ECS, Animation, Audio                 │  ← FALTA ❌
│  └── Debug, PostProcess, Error                      │  ← FALTA ❌
└──────────────────────┬──────────────────────────────┘
                       │ extern "C"
                       ▼
┌─────────────────────────────────────────────────────┐
│  C ABI — reactor_c_api.dll                          │
│  ├── reactor_run(), reactor_key_*(), ...             │  ← Ya listo ✅
│  ├── reactor_create_material(), _load_texture()     │  ← FALTA ❌
│  ├── reactor_physics_*(), reactor_ecs_*()           │  ← FALTA ❌
│  └── reactor_get_last_error()                       │  ← FALTA ❌
└──────────────────────┬──────────────────────────────┘
                       │ Rust FFI
                       ▼
┌─────────────────────────────────────────────────────┐
│  REACTOR Rust Core                                  │
│  ├── VulkanContext (ash) — unsafe controlado         │  ← Solo Rust puede ✅
│  ├── RAII (Drop) — cleanup automático               │  ← Solo Rust puede ✅
│  ├── Ownership — recursos GPU seguros               │  ← Solo Rust puede ✅
│  ├── Texturas, Materials, Models                    │  ← FALTA funcional ❌
│  └── Physics, ECS, Audio reales                     │  ← FALTA funcional ❌
└──────────────────────┬──────────────────────────────┘
                       ▼
                   Vulkan 1.3 → GPU
```

---

## 📋 Orden de Implementación (Dependencias)

### **Sprint 1 — Fundación (CRÍTICO)**
> Sin esto, nada más puede funcionar bien.

| Orden | Rust | C ABI | C++ | Descripción |
|-------|------|-------|-----|-------------|
| 0 | R5a ✅ | A0 | C0 | **ReactorConfig** — `renderer`, `scene`, `vsync` en Rust → propagar a CConfig → Config C++ |
| 1 | R3 | A1 | C9 | **Error Handling** — `ReactorError` enum → `reactor_get_last_error()` → `reactor::Error` |
| 2 | R2 | — | — | **Validation Layers** — Solo Rust, debug builds |
| 3 | R5 | — | — | **Depth Buffer** — Integrar en render pass existente |

### **Sprint 2 — Renderizado Visual**
> Poder ver algo más que un cubo monocolor.

| Orden | Rust | C ABI | C++ | Descripción |
|-------|------|-------|-----|-------------|
| 4 | R6 | A3 | C2 | **Texturas** — Rust carga PNG → C ABI expone → C++ wrappea |
| 5 | R7 | A2 | C1 | **Materials** — Con texturas y uniforms |
| 6 | R8 | — | — | **Render Pass** — Forward con depth+MSAA |

### **Sprint 3 — Contenido 3D**
> Cargar modelos reales del mundo.

| Orden | Rust | C ABI | C++ | Descripción |
|-------|------|-------|-----|-------------|
| 7 | R9 | A4 | C3 | **OBJ Loader** → exponer → wrappear |
| 8 | R10 | A4 | C3 | **glTF 2.0** → mismo pipeline |
| 9 | — | A11 | C11 | **GPU Info** — Exponer lo que ya existe en Rust |
| 10 | — | A7 | C6 | **Debug Draw** — Exponer lo que ya existe en Rust |

### **Sprint 4 — Sistemas de Juego**
> Hacer juegos reales.

| Orden | Rust | C ABI | C++ | Descripción |
|-------|------|-------|-----|-------------|
| 11 | R12 | A5 | C4 | **Physics** — Collision + rigidbody funcional |
| 12 | R13 | A6 | C5 | **ECS** — Query system funcional |
| 13 | R14 | A8 | C7 | **Animation** — Integrado con scene |
| 14 | R15 | A9 | C8 | **Audio** — Backend real |

### **Sprint 5 — Visual Avanzado**
> Calidad gráfica profesional.

| Orden | Rust | C ABI | C++ | Descripción |
|-------|------|-------|-----|-------------|
| 15 | R16 | A10 | C10 | **Shadows** |
| 16 | R17 | A10 | C10 | **Post-Processing** |
| 17 | R18 | — | — | **PBR Materials** |
| 18 | R19 | — | — | **Compute Shaders** |

---

## 💡 ¿Por qué Rust + C++ y no solo uno?

| Aspecto | Rust hace mejor | C++ hace mejor |
|---------|----------------|----------------|
| **Vulkan unsafe** | ✅ Controlado con ownership | ❌ UB fácil de introducir |
| **Resource cleanup** | ✅ Drop automático (RAII perfecto) | 🟡 RAII manual, se puede olvidar |
| **Multithreading** | ✅ Send/Sync en compilación | ❌ Data races en runtime |
| **Templates** | 🟡 Generics (más limitados) | ✅ Templates Turing-completos |
| **Operator overload** | 🟡 Traits (verboso) | ✅ Natural (`mat * vec`) |
| **Herencia OOP** | ❌ No tiene (usa traits) | ✅ `class MyGame : public App` |
| **Ecosistema gamedev** | 🟡 Creciendo | ✅ Maduro (ImGui, FMOD, etc) |
| **Interop universal** | 🟡 Via C ABI | ✅ C ABI nativo |

**Juntos**: Rust protege la GPU, C++ da productividad al usuario. C ABI los conecta.