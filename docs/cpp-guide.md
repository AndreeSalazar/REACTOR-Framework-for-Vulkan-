# REACTOR Framework — Guía de Desarrollo en C++

**Versión 1.0.5** | Para desarrolladores C++ | Powered by Salazar-interactive

## Introducción

Esta guía te enseña a crear juegos con REACTOR usando C++ a través del C ABI.
REACTOR expone **3300+ funciones** desde Rust vía `extern "C"`, y el **C++ SDK** (1477 líneas header-only) las envuelve en clases RAII idiomáticas.

## Requisitos

- CMake 3.16+
- Compilador C++17 (MSVC, GCC, Clang)
- Vulkan SDK 1.3+
- `reactor_c_api.dll` (compilado desde Rust)

## Configuración del Proyecto

### Compilar la C API

```bash
cd REACTOR-Framework-for-Vulkan-
cargo build --release --manifest-path cpp/reactor_c_api/Cargo.toml
```

Esto genera `reactor_c_api.dll` en `cpp/reactor_c_api/target/release/`.

### CMakeLists.txt

```cmake
cmake_minimum_required(VERSION 3.16)
project(MiJuego)

set(CMAKE_CXX_STANDARD 17)

include_directories(path/to/REACTOR/cpp/reactor_cpp/include)

add_executable(mi_juego main.cpp)

target_link_directories(mi_juego PRIVATE path/to/REACTOR/cpp/reactor_c_api/target/release)
target_link_libraries(mi_juego PRIVATE reactor_c_api.dll.lib)

# Copiar DLL al lado del ejecutable
add_custom_command(TARGET mi_juego POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        "path/to/REACTOR/cpp/reactor_c_api/target/release/reactor_c_api.dll"
        "$<TARGET_FILE_DIR:mi_juego>"
)
```

## Patrón ReactorApp

### Estilo Clase (Recomendado)

```cpp
#include <reactor/application.hpp>
using namespace reactor;

class MiJuego : public Application {
    float rotacion = 0.0f;

public:
    Config config() override {
        return Config("Mi Juego", 1920, 1080).with_msaa(4);
    }

    void on_init() override {
        Camera::set_position({0, 2, 5});
        Camera::set_target({0, 0, 0});
        Lighting::add_directional({-0.5f, -1, -0.3f}, {1, 1, 1}, 1.0f);
    }

    void on_update(float dt) override {
        rotacion += dt;
        if (Input::key_pressed(Input::KEY_ESCAPE())) Window::request_close();
    }
};

int main() { return MiJuego().run(); }
```

### Estilo Lambda (Prototyping rápido)

```cpp
#include <reactor/application.hpp>

int main() {
    float rot = 0;
    return reactor::ReactorApp(
        reactor::Config("Mi Juego").with_size(1280, 720),
        []() { reactor::Camera::set_position({0, 2, 5}); },
        [&](float dt) { rot += dt; },
        []() {}
    );
}
```

## API Completa del C++ SDK

### Cámara

```cpp
Camera::set_position({x, y, z});
Camera::set_target({x, y, z});
Vec3 pos = Camera::position();
Mat4 vp = Camera::view_projection();
```

### Iluminación

```cpp
Lighting::add_directional({dx, dy, dz}, {r, g, b}, intensity);
Lighting::add_point({x, y, z}, {r, g, b}, intensity, range);
Lighting::add_spot({x, y, z}, {dx, dy, dz}, {r, g, b}, intensity, range, angle);
Lighting::clear();
uint32_t count = Lighting::count();
```

### Input

```cpp
if (Input::key_pressed(Input::KEY_SPACE())) { /* una vez */ }
if (Input::key_down(Input::KEY_W())) { /* mientras presionado */ }
auto [mx, my] = Input::mouse_position();
auto [dx, dy] = Input::mouse_delta();
```

### Escena

```cpp
auto* mesh = reactor_create_cube();
auto* mat = reactor_create_material_simple(1.0f, 0.5f, 0.2f);
int32_t idx = Scene::add_object(mesh, mat, transform);
Scene::set_transform(idx, new_transform);
Scene::set_visible(idx, false);
Scene::clear();
```

### ECS — Entity Component System

```cpp
// Crear entidad
Entity player = Entity::create("Player");
player.set_position(Vec3(0, 1, 0));
player.add_mesh_renderer(mesh_id, material_id);

// Componentes
player.add_rigidbody(80.0f, false);  // mass, is_kinematic
player.apply_force(Vec3(0, 500, 0));

CLight light{};
light.light_type = 1;  // Point
light.color = {1, 0.8f, 0.6f};
light.intensity = 2.0f;
light.range = 10.0f;
player.add_light(light);

// Queries
auto entities = ECS::query(COMPONENT_MESH_RENDERER | COMPONENT_RIGIDBODY);
for (auto& e : entities) {
    Vec3 pos = e.position();
    // ...
}

// Cleanup
player.destroy();
```

### PBR Materials

```cpp
// Crear material base
uint32_t mat = PBRMaterial::create("Gold");
PBRMaterial::set_base_color(mat, 1.0f, 0.84f, 0.0f, 1.0f);
PBRMaterial::set_metallic(mat, 1.0f);
PBRMaterial::set_roughness(mat, 0.3f);

// Crear instancia con variaciones
uint32_t inst = PBRMaterial::create_instance(mat);
PBRMaterial::set_roughness(inst, 0.8f);

// Emissive
PBRMaterial::set_emissive(mat, 0.0f, 1.0f, 0.0f, 5.0f);

PBRMaterial::destroy(mat);
```

### FrameGraph

```cpp
// Crear graph
uint32_t graph = FrameGraph::create();

// Declarar recursos
uint32_t color = FrameGraph::create_resource(graph, "GBuffer_Color", 0, 1920, 1080);
uint32_t depth = FrameGraph::create_resource(graph, "Depth", 1, 1920, 1080);

// Declarar passes
uint32_t geo_pass = FrameGraph::add_pass(graph, "GeometryPass", 0);
FrameGraph::pass_write(graph, geo_pass, color);
FrameGraph::pass_write(graph, geo_pass, depth);

uint32_t light_pass = FrameGraph::add_pass(graph, "LightingPass", 1);
FrameGraph::pass_read(graph, light_pass, color);

// Compilar (auto-barriers)
FrameGraph::compile(graph);

// O usar presets
uint32_t forward = FrameGraph::create_forward(1920, 1080);
uint32_t deferred = FrameGraph::create_deferred(1920, 1080);
```

### Render Stats y Telemetría

```cpp
auto stats = RenderStats::get();
printf("FPS: %.0f | Draw: %u | Tris: %u | VRAM: %u MB\n",
    stats.fps, stats.draw_calls, stats.triangles, stats.vram_total_mb);

auto budget = RenderStats::memory_budget();
printf("Device Local: %llu MB\n", budget.device_local_budget / (1024*1024));

// GPU Info
printf("GPU: %s\n", GPUInfo::name());
printf("VRAM: %u MB\n", GPUInfo::vram_mb());
printf("RT: %s\n", GPUInfo::raytracing_supported() ? "YES" : "NO");
```

### PlayMode (Editor Bridge)

```cpp
PlayMode::enter();    // Snapshot scene, start play
PlayMode::pause(true);
PlayMode::update(dt);
float pt = PlayMode::time();
PlayMode::exit();     // Restore scene snapshot
```

### Scene Serialization

```cpp
std::string json = SceneSerializer::serialize();
printf("Scene: %s\n", json.c_str());
```

### Tiempo

```cpp
float dt = Time::delta();
float total = Time::total();
float fps = Time::fps();
uint64_t frame = Time::frame_count();
```

## Ejemplos C++ (9 demos)

Todos en `cpp/examples/3D/`:

| Carpeta | Ejecutable | Qué demuestra |
| ------- | ---------- | ------------- |
| `main_basic.cpp` | `reactor_3d` | Lifecycle básico, cubo con material |
| `ecs_scene/` | `reactor_ecs_scene` | Entity CRUD, components, queries |
| `pbr_materials/` | `reactor_pbr_materials` | PBR metallic/roughness, instances, emissive |
| `frame_graph/` | `reactor_frame_graph` | Custom render passes, forward/deferred |
| `fps_controller/` | `reactor_fps_controller` | WASD + mouse look + jump + gravity |
| `lighting_showcase/` | `reactor_lighting` | Directional, point, spot lights animados |
| `telemetry_stats/` | `reactor_telemetry` | GPU stats, memory budget, serialización |
| `play_mode/` | `reactor_play_mode` | Enter/exit/pause play mode |
| `multi_object/` | `reactor_multi_object` | 225 objetos, wave, visibility, queries |

### Compilar y ejecutar todos

```bash
# 1. Compilar C API
cargo build --release -p reactor-c-api

# 2. Compilar ejemplos
cd cpp/examples/3D
cmake -B build
cmake --build build --config Release

# 3. Ejecutar cualquiera
./build/Release/reactor_lighting.exe
```

## Tips

1. **Copiar DLL** — `reactor_c_api.dll` debe estar junto al ejecutable
2. **C++17** — Usa structured bindings, `std::string`, etc.
3. **RAII** — Los wrappers C++ manejan memoria automáticamente
4. **Delta time** — Siempre multiplica movimiento por `dt`
5. **Ownership** — Rust crea → Rust destruye. Nunca `delete` un handle

## Licencia

MIT License — **Powered by Salazar-interactive**
