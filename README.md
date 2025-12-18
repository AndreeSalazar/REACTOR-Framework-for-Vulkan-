# 🚀 Stack-GPU-OP

<div align="center">

**ADead-GPU Technologies Reimplemented in Pure Vulkan**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Vulkan](https://img.shields.io/badge/Vulkan-1.3-red.svg)](https://www.vulkan.org/)
[![Platform](https://img.shields.io/badge/Platform-Cross--Platform-blue.svg)](https://www.vulkan.org/)
[![Status](https://img.shields.io/badge/Status-Professional-brightgreen.svg)](https://github.com)
[![Version](https://img.shields.io/badge/Version-0.4.1-blue.svg)](https://github.com)
[![FPS](https://img.shields.io/badge/FPS-74--75-brightgreen.svg)](https://github.com)
[![Progress](https://img.shields.io/badge/Progress-45%25-yellow.svg)](https://github.com)

**Stack-GPU-OP** combina las tecnologías revolucionarias de **ADead-GPU** (DirectX 12) con el framework **REACTOR** (Vulkan), creando el framework GPU más avanzado y accesible.

### La Ecuación

```
ADead-GPU (DirectX 12 Research) + REACTOR (Vulkan Framework) = Stack-GPU-OP
```

### ⭐ Estado Actual - v0.4.0

✅ **Cubo 3D Profesional** - Phong shading completo (Ambient + Diffuse + Specular)  
✅ **Depth Buffer** - Renderizado 3D correcto con D32_SFLOAT  
✅ **Normales por Vértice** - Iluminación realista en cada cara  
✅ **ISR Headers + Shaders** - Sistema completo (implementación pendiente)  
✅ **SDF Primitives** - 6 primitivas + CSG operations  
✅ **70-75 FPS** - Performance profesional estable  

</div>

---

## 🎨 Características Visuales (v0.4.0)

### Phong Shading Profesional
- **Ambient Light**: 30% - Iluminación base constante
- **Diffuse Light**: 100% - Iluminación direccional basada en normales
- **Specular Highlights**: 60% - Reflejos brillantes (shininess 32)
- **Luz dinámica**: Posición (5, 5, 5) con color blanco

### Geometría Avanzada
- **24 vértices** con normales correctas por cara
- **36 índices** optimizados (12 triángulos)
- **Depth buffer** D32_SFLOAT para renderizado 3D correcto
- **Back-face culling** para mejor performance

### Colores Vibrantes
- **Cara frontal**: Cyan/Teal brillante (como LunarG)
- **Caras laterales**: Grises con gradientes
- **Top/Bottom**: Cyan claro/oscuro
- **Modulación**: Por iluminación Phong

---

## 🚀 Características del Framework

✨ **API Declarativa**: Builder pattern fluido para todos los recursos  
🛡️ **Type Safety**: Enums fuertemente tipados, sin números mágicos  
♻️ **RAII Automático**: Gestión automática de recursos, sin memory leaks  
⚡ **Zero-Cost**: Abstracciones sin overhead en runtime  
🎨 **Phong Shading**: Iluminación profesional con ambient, diffuse y specular  
🔧 **Control Total**: Acceso directo a Vulkan cuando lo necesites  
📊 **Performance**: 70-75 FPS constantes con depth buffer  

## 🚀 Quick Start

### ⚡ Cómo Ejecutar (Sin Depender de Nadie)

**Opción 1: Ejecutar Directamente (Más Rápido)**
```bash
# Navegar al ejecutable
cd build\examples\stack-gpu-cube\Release

# Ejecutar
.\stack-gpu-cube.exe
```

**Opción 2: Compilar y Ejecutar**
```bash
# 1. Compilar (solo si hiciste cambios)
cmake --build build --config Release --target stack-gpu-cube

# 2. Ejecutar
cd build\examples\stack-gpu-cube\Release
.\stack-gpu-cube.exe
```

**Opción 3: Desde Cero (Primera Vez)**
```bash
# 1. Setup completo (solo primera vez)
quick-setup.bat

# 2. Ejecutar
cd build\examples\stack-gpu-cube\Release
.\stack-gpu-cube.exe
```

**Controles en el Cubo:**
- **Teclas 1-7**: Cambiar modos de visualización
- **ESC**: Salir

### Ver el Cubo 3D con Phong Shading

```bash
# Compilar todo el proyecto
quick-setup.bat

# Ejecutar el cubo 3D profesional (Stack-GPU-OP)
cd build\examples\stack-gpu-cube\Release
.\stack-gpu-cube.exe
```

**Lo que verás**:
- Cubo 3D rotando con iluminación Phong realista
- Reflejos especulares brillantes
- Depth buffer funcionando correctamente
- FPS en tiempo real en el título de la ventana
- 70-75 FPS constantes

### Setup Automático (5 minutos)

```bash
# Un solo comando - detecta todo automáticamente
quick-setup.bat

# Otros ejemplos disponibles
build\examples\triangle\reactor-triangle.exe
```

### Setup Manual

```bash
# Configurar (detecta Vulkan SDK automáticamente)
configure.bat

# Compilar
build.bat

# Ejecutar ejemplo
build\examples\triangle\reactor-triangle.exe
```

### Usar Template Starter

```bash
cd templates\starter
setup.bat
build.bat
run.bat
```

> 💡 **Nota**: Los scripts detectan automáticamente tu Vulkan SDK (1.4.328.1) y Visual Studio 2022

### Ejemplo Básico

```cpp
#include "reactor/reactor.hpp"
#include "reactor/vulkan_context.hpp"
#include "reactor/buffer.hpp"

int main() {
    // 1. Inicializar contexto
    reactor::VulkanContext ctx(true);
    ctx.init();
    
    // 2. Crear buffer con builder pattern
    auto buffer = reactor::Buffer::create(ctx.allocator())
        .size(1024)
        .usage(reactor::BufferUsage::Vertex)
        .memoryType(reactor::MemoryType::HostVisible)
        .build();
    
    // 3. Subir datos
    buffer.upload(vertices.data(), sizeof(vertices));
    
    // 4. Usar en comandos...
    
    // 5. Cleanup automático (RAII)
    ctx.shutdown();
    return 0;
}
```

## 📚 Documentación

### Stack-GPU-OP (ADead-GPU + Vulkan)
- **[META/META.md](META/META.md)** - ⭐ Overview completo del proyecto v0.4.0
- **[META/STACK_GPU_OP_VISION.md](META/STACK_GPU_OP_VISION.md)** - Visión: ADead-GPU implementado en Vulkan
- **[META/IMPROVEMENTS_v0.4.0.md](META/IMPROVEMENTS_v0.4.0.md)** - Detalles de Phong shading y mejoras visuales
- **[META/ROADMAP.md](META/ROADMAP.md)** - Plan de desarrollo completo
- **[META/CHANGELOG.md](META/CHANGELOG.md)** - Historial de cambios (v0.4.0, v0.3.1, v0.3.0)

### REACTOR Framework
- **[USAGE_GUIDE.md](USAGE_GUIDE.md)** - Guía completa de uso con ejemplos
- **[META/ARCHITECTURE.md](META/ARCHITECTURE.md)** - Arquitectura técnica detallada
- **[ideas.md](ideas.md)** - Diseño, roadmap y filosofía del framework

## 🏗️ Arquitectura

REACTOR está organizado en capas modulares:

```
┌─────────────────────────────────────┐
│     Scene Graph & Components        │  Capa 8: Alto nivel (futuro)
├─────────────────────────────────────┤
│     Render Graph & Passes           │  Capa 7: Rendering
├─────────────────────────────────────┤
│     Synchronization                 │  Capa 6: Fences, Semaphores
├─────────────────────────────────────┤
│     Command Buffers                 │  Capa 5: Grabación de comandos
├─────────────────────────────────────┤
│     Descriptor Sets                 │  Capa 4: Bindings
├─────────────────────────────────────┤
│     Shaders & Pipelines             │  Capa 3: Graphics/Compute
├─────────────────────────────────────┤
│     Buffers, Images, Samplers       │  Capa 2: Recursos
├─────────────────────────────────────┤
│     VulkanContext & Allocator       │  Capa 1: Core ✅
└─────────────────────────────────────┘
```

## 🎨 Componentes Implementados

### ✅ Stack-GPU-OP Extensions (Capa 6) ⭐ NUEVO

#### ISR (Intelligent Shading Rate)
- `importance.hpp/cpp` - Cálculo de importancia visual
- `adaptive.hpp/cpp` - Pixel sizing adaptivo
- `temporal.hpp/cpp` - Coherencia temporal
- `isr_system.hpp/cpp` - Sistema completo con Builder
- **Shaders**: `importance.comp`, `adaptive.comp`, `temporal.comp`
- **Estado**: Headers + Shaders completos (implementación pendiente)

#### SDF Rendering (Vector3D)
- `primitives.hpp/cpp` - 6 primitivas SDF (Sphere, Box, Torus, Cylinder, Capsule, Cone)
- `raymarcher.hpp/cpp` - Ray marching engine
- **Shaders**: `primitives.glsl` - Biblioteca completa de funciones SDF
- **CSG Operations**: Union, Subtract, Intersect, Smooth variants
- **Estado**: ✅ Completo

#### Cube Renderer (Ejemplo Profesional) ⭐ v0.4.0
- **Phong Shading**: Ambient (30%) + Diffuse (100%) + Specular (60%)
- **Normales por vértice**: 24 vértices con normales correctas
- **Depth Buffer**: D32_SFLOAT para renderizado 3D correcto
- **Push Constants**: MVP + Model matrices (128 bytes)
- **Performance**: 70-75 FPS constantes
- **Estado**: ✅ Profesional

### ✅ REACTOR Core

#### Core (Capa 1)
- `VulkanContext` - Inicialización y gestión de Vulkan
- `MemoryAllocator` - Gestión unificada de memoria GPU

#### Resources (Capa 2)
- `Buffer` - Buffers con builder pattern (vertex, index, uniform, storage)
- `Image` - Texturas y render targets con mipmapping
- `Sampler` - Samplers configurables (filtrado, wrapping, anisotropía)

#### Shaders & Pipelines (Capa 3)
- `Shader` - Carga de SPIR-V con múltiples stages
- `GraphicsPipeline` - Pipeline gráfico declarativo
- `ComputePipeline` - Pipeline de compute

#### Descriptors (Capa 4)
- `DescriptorSetLayout` - Layouts con builder pattern
- `DescriptorPool` - Pool con gestión automática
- `DescriptorSet` - Sets con updates simplificados

#### Commands (Capa 5)
- `CommandPool` - Pools thread-safe
- `CommandBuffer` - Grabación fluida de comandos

#### Synchronization (Capa 6)
- `Fence` - Sincronización CPU-GPU
- `Semaphore` - Sincronización GPU-GPU
- `Barrier` - Memory barriers y layout transitions

#### Rendering (Capa 7)
- `RenderPass` - Render passes declarativos (con depth support)
- `Framebuffer` - Framebuffers con attachments (color + depth)
- `Swapchain` - Gestión de swapchain para presentación

## 💡 Ejemplos

### Cubo 3D con Phong Shading (v0.4.0)

```cpp
// Crear cube renderer con Phong shading
cube::CubeRenderer cubeRenderer(ctx, renderPass.handle(), width, height);

// En el render loop
glm::mat4 model = transform.getMatrix();
glm::mat4 view = camera.getViewMatrix();
glm::mat4 proj = camera.getProjectionMatrix();
glm::mat4 mvp = proj * view * model;

// Renderizar con iluminación Phong
cubeRenderer.render(cmd, mvp, model);
```

**Resultado**: Cubo 3D con iluminación realista (ambient + diffuse + specular) a 70-75 FPS

### Depth Buffer Creation

```cpp
// Crear depth image
VkImageCreateInfo depthInfo{};
depthInfo.format = VK_FORMAT_D32_SFLOAT;
depthInfo.extent = {width, height, 1};
depthInfo.usage = VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT;

VkImage depthImage;
vkCreateImage(device, &depthInfo, nullptr, &depthImage);

// Usar REACTOR allocator
auto depthBlock = allocator->allocate(memReqs, MemoryType::DeviceLocal);
vkBindImageMemory(device, depthImage, depthBlock.memory, depthBlock.offset);
```

### Buffer Creation
```cpp
auto vertexBuffer = reactor::Buffer::create(allocator)
    .size(sizeof(vertices))
    .usage(reactor::BufferUsage::Vertex | reactor::BufferUsage::TransferDst)
    .memoryType(reactor::MemoryType::DeviceLocal)
    .build();
```

### Pipeline with Depth Test
```cpp
auto pipeline = reactor::GraphicsPipeline::create(device, renderPass)
    .shader(vertShader)
    .shader(fragShader)
    .vertexInput(bindings, attributes)
    .topology(reactor::Topology::TriangleList)
    .cullMode(reactor::CullMode::Back)
    .depthTest(true)  // ✅ Depth buffer enabled
    .viewport(1920.0f, 1080.0f)
    .build();
```

## 🔧 Requisitos

- **Vulkan SDK** 1.3+ con `VULKAN_SDK` configurado
- **CMake** 3.24 o superior
- **C++20** compatible compiler:
  - MSVC 2022 (Windows)
  - GCC 11+ (Linux)
  - Clang 14+ (macOS/Linux)
- **Ninja** (opcional pero recomendado)

## 📦 Estructura del Proyecto

```
REACTOR/
├── META/                           ⭐ Documentación Stack-GPU-OP
│   ├── META.md                     # Overview v0.4.0
│   ├── STACK_GPU_OP_VISION.md      # ADead-GPU + Vulkan
│   ├── IMPROVEMENTS_v0.4.0.md      # Phong shading details
│   ├── ROADMAP.md                  # Plan completo
│   ├── CHANGELOG.md                # Historial de versiones
│   ├── ARCHITECTURE.md             # Arquitectura técnica
│   └── PROGRESS_REPORT.md          # Progreso (40% completado)
│
├── reactor/
│   ├── include/reactor/
│   │   ├── core/                   # Vulkan context, buffers, etc.
│   │   ├── isr/                    ⭐ ISR System (headers)
│   │   └── sdf/                    ⭐ SDF Rendering (completo)
│   └── src/
│       ├── core/                   # Implementaciones
│       └── sdf/                    # SDF implementations
│
├── shaders/
│   ├── isr/                        ⭐ ISR compute shaders
│   ├── sdf/                        ⭐ SDF GLSL library
│   └── cube/                       ⭐ Phong shading shaders
│
├── examples/
│   ├── stack-gpu-cube/             ⭐ Cubo 3D profesional (v0.4.0)
│   ├── triangle/                   # Hello Triangle
│   └── sandbox/                    # Ejemplo mínimo
│
├── docs/                           # Documentación adicional
├── LICENSE                         # MIT License
└── README.md                       # Este archivo
```

## 🎯 Roadmap

### ✅ v0.4.0 (Actual) - Phong Shading Profesional
- **Cubo 3D con Phong shading** - Ambient + Diffuse + Specular
- **Normales por vértice** - 24 vértices con normales correctas
- **Depth buffer** - D32_SFLOAT para renderizado 3D
- **Push constants mejorados** - MVP + Model matrices
- **70-75 FPS** - Performance profesional estable

### ✅ v0.3.1 - Mejoras Visuales
- Depth buffer implementado
- 24 vértices con colores por cara
- Render pass con depth attachment
- FPS en título de ventana

### ✅ v0.3.0 - Cubo 3D Funcionando
- Cube Renderer completo
- Shaders con MVP matrices
- Rotación animada
- 74-80 FPS constantes

### ✅ v0.2.0 - Stack-GPU-OP Headers
- ISR System (headers + shaders)
- SDF Rendering (completo)
- React-Style Builder API

### ✅ v0.1.0 - REACTOR Core
- Core framework completo
- Gestión de recursos (buffers, images)
- Pipelines gráficos y compute
- Command buffers y sincronización
- Render passes y swapchain

### 🚧 v0.2 (Próximo)
- [ ] Window integration (GLFW/SDL)
- [ ] Swapchain resize automático
- [ ] Staging buffer pool
- [ ] Shader hot-reload

### 📋 v0.3

### 🚀 v1.0.0 - Release Completo
- Advanced Ray Tracing (cone/beam tracing)
- GPU Language (.gpu parser)
- Scene graph y componentes
- Material system PBR(point, directional, spot)

### 🚀 v1.0
- [ ] Ray tracing support
- [ ] Mesh shaders
- [ ] Variable rate shading
- [ ] Production ready

## 🤝 Contribuir

Las contribuciones son bienvenidas! Por favor:

1. Fork el proyecto
2. Crea una branch para tu feature (`git checkout -b feature/AmazingFeature`)
3. Commit tus cambios (`git commit -m 'Add AmazingFeature'`)
4. Push a la branch (`git push origin feature/AmazingFeature`)
5. Abre un Pull Request

## 📄 Licencia

Este proyecto está bajo la licencia MIT. Ver `LICENSE` para más detalles.

## 🙏 Agradecimientos

- Vulkan SDK y la comunidad de Khronos
- Inspiración de frameworks modernos como React
- Comunidad de desarrollo de gráficos

## 📞 Contacto

- Issues: [GitHub Issues](https://github.com/tu-usuario/reactor/issues)
- Documentación: Ver archivos `.md` en el repositorio
- +51 945 375 729
---

<div align="center">

**REACTOR Framework** - Simplificando Vulkan sin sacrificar control

Hecho con ❤️ para la comunidad de desarrollo gráfico

</div>

