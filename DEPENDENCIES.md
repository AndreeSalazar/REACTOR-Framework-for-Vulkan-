# REACTOR Framework - Dependencias

## Dependencias Requeridas

### 1. Vulkan SDK (REQUERIDO)

**Versión**: 1.3.x o superior  
**Instalación**: El usuario DEBE descargar e instalar

**Windows**:
```bash
# Descargar desde:
https://vulkan.lunarg.com/

# Verificar instalación:
echo %VULKAN_SDK%
# Debe mostrar: C:\VulkanSDK\1.x.xxx.x
```

**Linux**:
```bash
# Ubuntu/Debian
sudo apt install vulkan-sdk

# Fedora
sudo dnf install vulkan-headers vulkan-loader vulkan-tools
```

**macOS**:
```bash
# Descargar desde: https://vulkan.lunarg.com/
# O usar Homebrew:
brew install --cask vulkan-sdk
```

---

## Dependencias Opcionales (Para Renderizado Completo)

Estas dependencias se instalan automáticamente con `install-dependencies.bat`

### 2. GLFW3 (Sistema de Ventanas)

**Propósito**: Crear ventanas y manejar input  
**Instalación**: Automática con vcpkg

```bash
# Manual:
vcpkg install glfw3:x64-windows
```

**Características**:
- ✅ Creación de ventanas multiplataforma
- ✅ Input (teclado, mouse, gamepad)
- ✅ Integración con Vulkan surface
- ✅ Callbacks de eventos

### 3. GLM (Matemáticas)

**Propósito**: Matemáticas 3D (vectores, matrices, transformaciones)  
**Instalación**: Automática con vcpkg

```bash
# Manual:
vcpkg install glm:x64-windows
```

**Características**:
- ✅ Compatible con GLSL
- ✅ Vectores (vec2, vec3, vec4)
- ✅ Matrices (mat2, mat3, mat4)
- ✅ Transformaciones (translate, rotate, scale)
- ✅ Proyecciones (perspective, ortho)

### 4. STB (Carga de Imágenes)

**Propósito**: Cargar texturas (PNG, JPG, etc.)  
**Instalación**: Automática con vcpkg

```bash
# Manual:
vcpkg install stb:x64-windows
```

**Características**:
- ✅ Carga de PNG, JPG, TGA, BMP
- ✅ Header-only library
- ✅ Sin dependencias externas

---

## Instalación Automática

### Opción 1: Script Automático (Recomendado)

```bash
# Instala GLFW3, GLM y STB automáticamente
install-dependencies.bat
```

Este script:
1. Descarga vcpkg si no está instalado
2. Compila vcpkg
3. Instala todas las dependencias
4. Integra con CMake

### Opción 2: vcpkg Manifest Mode

Si ya tienes vcpkg instalado:

```bash
# El archivo vcpkg.json ya está configurado
cmake -S . -B build -DCMAKE_TOOLCHAIN_FILE=path/to/vcpkg/scripts/buildsystems/vcpkg.cmake
cmake --build build
```

### Opción 3: Conan

```bash
conan install . --build=missing
cmake --preset conan-release
cmake --build build
```

---

## Matriz de Características

| Feature | Sin Dependencias | Con GLFW3 | Con GLM | Con STB |
|---------|-----------------|-----------|---------|---------|
| **Core Vulkan** | ✅ | ✅ | ✅ | ✅ |
| **Buffers** | ✅ | ✅ | ✅ | ✅ |
| **Imágenes** | ✅ | ✅ | ✅ | ✅ |
| **Pipelines** | ✅ | ✅ | ✅ | ✅ |
| **Ventanas** | ❌ | ✅ | ✅ | ✅ |
| **Input** | ❌ | ✅ | ✅ | ✅ |
| **Renderizado** | ❌ | ✅ | ✅ | ✅ |
| **Matemáticas 3D** | ❌ | ❌ | ✅ | ✅ |
| **Texturas** | ❌ | ❌ | ❌ | ✅ |

---

## Uso sin Dependencias Opcionales

REACTOR puede usarse sin GLFW, GLM o STB para:

- ✅ Compute shaders
- ✅ Procesamiento de imágenes offscreen
- ✅ Headless rendering
- ✅ Integración con tu propio sistema de ventanas

**Ejemplo sin ventanas**:
```cpp
#include "reactor/reactor.hpp"
#include "reactor/vulkan_context.hpp"
#include "reactor/buffer.hpp"

int main() {
    reactor::VulkanContext ctx(true);
    ctx.init();
    
    // Usar buffers, compute shaders, etc.
    auto buffer = reactor::Buffer::create(ctx.allocator())
        .size(1024)
        .usage(reactor::BufferUsage::Storage)
        .build();
    
    ctx.shutdown();
    return 0;
}
```

---

## Verificar Dependencias Instaladas

```bash
# Ejecutar script de verificación
verificar.bat
```

Esto mostrará:
- ✅ Vulkan SDK instalado
- ✅ GLFW3 disponible (o no)
- ✅ GLM disponible (o no)
- ✅ STB disponible (o no)

---

## Troubleshooting

### GLFW3 no encontrado

```bash
# Instalar manualmente
install-dependencies.bat

# O con vcpkg:
vcpkg install glfw3:x64-windows
```

### GLM no encontrado

```bash
vcpkg install glm:x64-windows
```

### STB no encontrado

```bash
vcpkg install stb:x64-windows
```

### vcpkg no funciona

```bash
# Reinstalar vcpkg
rmdir /s /q vcpkg
git clone https://github.com/Microsoft/vcpkg.git
cd vcpkg
bootstrap-vcpkg.bat
```

---

## Dependencias Futuras (Roadmap)

### v0.2
- [ ] **ImGui** - UI inmediata
- [ ] **Assimp** - Carga de modelos 3D

### v0.3
- [ ] **Bullet3** - Física
- [ ] **OpenAL** - Audio 3D

### v0.4
- [ ] **Networking** - Multiplayer
- [ ] **JSON** - Configuración

---

## Resumen

**Para empezar rápido**:
1. Instala Vulkan SDK (manual)
2. Ejecuta `install-dependencies.bat` (automático)
3. Ejecuta `quick-setup.bat`
4. ¡Listo para desarrollar!

**Mínimo absoluto**:
- Solo Vulkan SDK (para compute/offscreen)

**Recomendado para juegos/apps**:
- Vulkan SDK + GLFW3 + GLM + STB
