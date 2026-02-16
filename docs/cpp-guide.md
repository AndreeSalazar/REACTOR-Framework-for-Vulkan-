# REACTOR Framework - Guia de Desarrollo en C++

**Version 1.0.5** | Para desarrolladores C++

## Introduccion

Esta guia te ensena a crear juegos con REACTOR usando C++ a traves del C ABI.

## Requisitos

- CMake 3.20+
- Compilador C++17 (MSVC, GCC, Clang)
- Vulkan SDK 1.3+
- reactor_c_api.dll (compilado desde Rust)

## Configuracion del Proyecto

### Compilar la C API

```bash
cd REACTOR-Framework-for-Vulkan-
cargo build --release --manifest-path cpp/reactor_c_api/Cargo.toml
```

Esto genera `reactor_c_api.dll` en `cpp/reactor_c_api/target/release/`.

### CMakeLists.txt

```cmake
cmake_minimum_required(VERSION 3.20)
project(MiJuego)

set(CMAKE_CXX_STANDARD 17)

# Incluir headers de REACTOR
include_directories(path/to/REACTOR/cpp/reactor_cpp/include)

# Ejecutable
add_executable(mi_juego main.cpp)

# Linkear contra la C API
target_link_directories(mi_juego PRIVATE path/to/REACTOR/cpp/reactor_c_api/target/release)
target_link_libraries(mi_juego PRIVATE reactor_c_api.dll.lib)

# Copiar DLL
add_custom_command(TARGET mi_juego POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        "path/to/REACTOR/cpp/reactor_c_api/target/release/reactor_c_api.dll"
        "$<TARGET_FILE_DIR:mi_juego>"
)
```

## Patron ReactorApp

### Estilo Clase (Recomendado)

```cpp
#include <reactor/reactor.hpp>

class MiJuego : public reactor::Application {
    float rotacion = 0.0f;

public:
    Config config() override {
        return Config("Mi Juego")
            .with_size(1920, 1080)
            .with_vsync(true)
            .with_msaa(4);
    }

    void on_init() override {
        // Configurar camara
        Camera::set_position({0.0f, 2.0f, 5.0f});
        Camera::set_target({0.0f, 0.0f, 0.0f});
        
        // Agregar luz
        Lighting::add_directional({-0.5f, -1.0f, -0.3f}, {1.0f, 1.0f, 1.0f}, 1.0f);
    }

    void on_update(float dt) override {
        rotacion += dt;
        
        // Input
        if (Input::key_pressed(Key::Escape)) {
            request_close();
        }
    }
};

int main() {
    return MiJuego().run();
}
```

### Estilo Funcional (Lambda)

```cpp
#include <reactor/reactor.hpp>

int main() {
    float rotacion = 0.0f;
    
    return reactor::ReactorApp(
        "Mi Juego",
        1280, 720,
        [&]() {
            // on_init
            reactor::Camera::set_position({0, 2, 5});
        },
        [&](float dt) {
            // on_update
            rotacion += dt;
        },
        [&]() {
            // on_render (opcional)
        }
    );
}
```

### Estilo Ultra-Simple (C API Directo)

```cpp
#include <cstdint>
#include <cstdio>

extern "C" {
    int32_t reactor_run_simple(const char* title, uint32_t w, uint32_t h,
        void(*init)(), void(*update)(float), void(*render)());
    void reactor_set_camera_position(float x, float y, float z);
    int32_t reactor_key_pressed(uint32_t key);
    uint32_t reactor_key_escape();
    void reactor_request_close();
}

void on_init() {
    reactor_set_camera_position(0, 2, 5);
}

void on_update(float dt) {
    if (reactor_key_pressed(reactor_key_escape())) {
        reactor_request_close();
    }
}

void on_render() {}

int main() {
    return reactor_run_simple("Mi Juego", 1280, 720, on_init, on_update, on_render);
}
```

## API de C++ (reactor.hpp)

### Camara

```cpp
reactor::Camera::set_position({x, y, z});
reactor::Camera::set_target({x, y, z});
auto pos = reactor::Camera::get_position();
```

### Iluminacion

```cpp
reactor::Lighting::add_directional({dx, dy, dz}, {r, g, b}, intensity);
reactor::Lighting::add_point({x, y, z}, {r, g, b}, intensity, range);
reactor::Lighting::add_spot({x, y, z}, {dx, dy, dz}, {r, g, b}, intensity, range, angle);
```

### Input

```cpp
if (reactor::Input::key_pressed(reactor::Key::Space)) { /* una vez */ }
if (reactor::Input::key_down(reactor::Key::W)) { /* mientras presionado */ }

auto [mx, my] = reactor::Input::mouse_position();
auto [dx, dy] = reactor::Input::mouse_delta();
```

### Escena

```cpp
// Crear mesh y material
auto* mesh = reactor::Mesh::create_cube();
auto* material = reactor::Material::create_simple(1.0f, 0.5f, 0.2f);

// Agregar a escena
int32_t index = reactor::Scene::add_object(mesh, material, transform);

// Modificar
reactor::Scene::set_transform(index, new_transform);
```

### Tiempo

```cpp
float dt = reactor::Time::delta();
float total = reactor::Time::total();
float fps = reactor::Time::fps();
uint64_t frame = reactor::Time::frame_count();
```

### Info

```cpp
const char* gpu = reactor::GPU::name();
uint32_t msaa = reactor::GPU::msaa_samples();
```

## C API Directo

Si prefieres usar el C API directamente:

```cpp
extern "C" {
    // Core
    int32_t reactor_run_simple(const char*, uint32_t, uint32_t, void(*)(), void(*)(float), void(*)());
    void reactor_request_close();
    
    // Camera
    void reactor_set_camera_position(float, float, float);
    void reactor_set_camera_target(float, float, float);
    
    // Lighting
    int32_t reactor_add_directional_light(float, float, float, float, float, float, float);
    int32_t reactor_add_point_light(float, float, float, float, float, float, float, float);
    
    // Input
    int32_t reactor_key_pressed(uint32_t);
    int32_t reactor_key_down(uint32_t);
    uint32_t reactor_key_w();
    uint32_t reactor_key_escape();
    
    // Scene
    void* reactor_create_cube();
    void* reactor_create_material_simple(float, float, float);
    int32_t reactor_add_object(void*, void*, CMat4);
    
    // Info
    const char* reactor_get_gpu_name();
    float reactor_get_fps();
}
```

## Estructura de Proyecto

```
mi-juego/
  CMakeLists.txt
  src/
    main.cpp
    game.hpp
    game.cpp
  assets/
    shaders/
    textures/
  libs/
    reactor_c_api.dll
    reactor_c_api.dll.lib
```

## Compilar y Ejecutar

```bash
mkdir build
cd build
cmake ..
cmake --build . --config Release
./Release/mi_juego.exe
```

## Tips

1. **Copiar DLL** - Asegurate de que `reactor_c_api.dll` este junto al ejecutable
2. **C++17** - Usa features modernas como structured bindings
3. **RAII** - Los wrappers C++ manejan memoria automaticamente
4. **Delta time** - Siempre multiplica movimiento por `dt`

## Ejemplos

Ver ejemplos en:

- `cpp/examples/hello_cpp/` - Ejemplo basico
- `cpp/examples/simple_cube/` - Cubo rotando
- `cpp/examples/3D/` - Ejemplo completo 3D

## Licencia

MIT License
