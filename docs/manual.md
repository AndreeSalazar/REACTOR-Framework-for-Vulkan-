# REACTOR Framework - Manual General

**Version 1.0.5** | Vulkan 1.3 | Rust + C++

## Que es REACTOR?

REACTOR es un framework de desarrollo de juegos que simplifica Vulkan usando Rust para seguridad y C++ para facilidad de uso.

## Instalacion

### Requisitos

- **Vulkan SDK 1.3+** - [vulkan.lunarg.com](https://vulkan.lunarg.com/)
- **Rust 1.70+** (para desarrollo Rust)
- **CMake 3.20+** (para desarrollo C++)
- **GPU compatible con Vulkan**

### Clonar Repositorio

```bash
git clone https://github.com/user/REACTOR-Framework-for-Vulkan.git
cd REACTOR-Framework-for-Vulkan
```

### Compilar

```bash
# Rust core
cargo build --release

# C API (para C++)
cargo build --release --manifest-path cpp/reactor_c_api/Cargo.toml
```

## Uso Basico

### Patron ReactorApp()

El patron principal es **heredar, configurar, ejecutar**:

```
1. Crear clase/struct que herede de Application/ReactorApp
2. Implementar config() - configuracion inicial
3. Implementar on_init() - setup de escena
4. Implementar on_update() - logica de juego
5. Llamar run() - ejecutar
```

### Ejemplo Minimo (Rust)

```rust
use reactor::prelude::*;

struct MiJuego;

impl ReactorApp for MiJuego {
    fn config(&self) -> ReactorConfig {
        ReactorConfig::new("Mi Juego").with_size(1280, 720)
    }
}

fn main() {
    reactor::run(MiJuego);
}
```

### Ejemplo Minimo (C++)

```cpp
#include <reactor/reactor.hpp>

class MiJuego : public reactor::Application {
    Config config() override {
        return Config("Mi Juego").with_size(1280, 720);
    }
};

int main() {
    return MiJuego().run();
}
```

## Funciones Principales

### Camara

```cpp
reactor_set_camera_position(x, y, z);
reactor_set_camera_target(x, y, z);
```

### Iluminacion

```cpp
reactor_add_directional_light(dx, dy, dz, r, g, b, intensity);
reactor_add_point_light(x, y, z, r, g, b, intensity, range);
```

### Escena

```cpp
void* mesh = reactor_create_cube();
void* material = reactor_create_material_simple(r, g, b);
int32_t index = reactor_add_object(mesh, material, transform);
reactor_set_object_transform(index, new_transform);
```

### Input

```cpp
if (reactor_key_pressed(reactor_key_escape())) {
    reactor_request_close();
}
```

### Info

```cpp
const char* gpu = reactor_get_gpu_name();
float fps = reactor_get_fps();
uint32_t msaa = reactor_get_msaa_samples();
```

## Arquitectura

```
Tu Juego (C++ o Rust)
        |
        v
   REACTOR SDK
        |
        v
   reactor.dll (C ABI)
        |
        v
   Rust Core (safe)
        |
        v
   Vulkan 1.3 (GPU)
```

## Caracteristicas

| Modulo | Descripcion |
| ------ | ----------- |
| Core | VulkanContext, Device, Allocator |
| Graphics | Swapchain, Pipeline, MSAA, Depth |
| Ray Tracing | RTX en GPUs compatibles |
| Resources | Mesh, Material, Texture |
| Systems | Input, Camera, Lighting, Physics |
| ADead-GPU | ISR, SDF, Anti-Aliasing avanzado |

## Actualizaciones

### v1.0.5 (Actual)

- C ABI completo
- C++ SDK con RAII
- Shaders embebidos
- Ray Tracing automatico
- MSAA 4x por defecto

### v0.4.x

- Version inicial Rust
- Vulkan 1.3 base

## Soporte

- **Issues**: GitHub Issues
- **Docs**: `/docs/` en el repositorio
- **Ejemplos**: `/examples/` (Rust) y `/cpp/examples/` (C++)

## Licencia

MIT License - Uso libre para proyectos comerciales y personales.
