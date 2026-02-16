# REACTOR Framework - Documentacion

Bienvenido a la documentacion oficial de **REACTOR Framework**.

## Guias Disponibles

| Guia | Descripcion |
|------|-------------|
| [Manual General](manual.md) | Manual corto y completo para uso general |
| [Guia Rust](rust-guide.md) | Desarrollo de juegos con Rust |
| [Guia C++](cpp-guide.md) | Desarrollo de juegos con C++ |

## Inicio Rapido

### Rust
```rust
use reactor::prelude::*;

fn main() {
    reactor::run(MiJuego::default());
}
```

### C++
```cpp
#include <reactor/reactor.hpp>

int main() {
    return reactor::ReactorApp("Mi Juego");
}
```

## Version Actual: 1.0.5

### Novedades
- C ABI completo para interoperabilidad C/C++
- C++ SDK con wrappers RAII
- Shaders SPIR-V embebidos por defecto
- Ray Tracing en cualquier GPU compatible
- MSAA 4x automatico
- Sistema ADead-GPU integrado

## Arquitectura

```
C++ Game  -->  reactor.dll (C ABI)  -->  Rust Core  -->  Vulkan 1.3
    |              |                        |              |
 Simple         Bridge                   Safe           GPU
```

## Licencia
MIT License
