# REACTOR Framework para Vulkan

<div align="center">

**Framework moderno para Vulkan que simplifica el desarrollo sin sacrificar control**

[![C++20](https://img.shields.io/badge/C++-20-blue.svg)](https://en.cppreference.com/w/cpp/20)
[![Vulkan](https://img.shields.io/badge/Vulkan-1.3-red.svg)](https://www.vulkan.org/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

</div>

## 🎯 ¿Qué es REACTOR?

REACTOR es un framework para Vulkan inspirado en el modelo declarativo de React, diseñado para **facilitar enormemente** el desarrollo de aplicaciones gráficas manteniendo el **control total** de Vulkan.

### Características Principales

✨ **API Declarativa**: Builder pattern fluido para todos los recursos  
🛡️ **Type Safety**: Enums fuertemente tipados, sin números mágicos  
♻️ **RAII Automático**: Gestión automática de recursos, sin memory leaks  
⚡ **Zero-Cost**: Abstracciones sin overhead en runtime  
🎨 **Componible**: Construye aplicaciones complejas con componentes simples  
🔧 **Control Total**: Acceso directo a Vulkan cuando lo necesites  

## 🚀 Quick Start

### Instalación

```bash
# Clonar repositorio
git clone https://github.com/tu-usuario/reactor.git
cd reactor

# Configurar y compilar
cmake -S . -B build -G "Ninja" -DCMAKE_BUILD_TYPE=Release
cmake --build build

# Ejecutar ejemplo
build\examples\triangle\reactor-triangle.exe
```

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

- **[USAGE_GUIDE.md](USAGE_GUIDE.md)** - Guía completa de uso con ejemplos
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - Arquitectura técnica detallada
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

### ✅ Core (Capa 1)
- `VulkanContext` - Inicialización y gestión de Vulkan
- `MemoryAllocator` - Gestión unificada de memoria GPU

### ✅ Resources (Capa 2)
- `Buffer` - Buffers con builder pattern (vertex, index, uniform, storage)
- `Image` - Texturas y render targets con mipmapping
- `Sampler` - Samplers configurables (filtrado, wrapping, anisotropía)

### ✅ Shaders & Pipelines (Capa 3)
- `Shader` - Carga de SPIR-V con múltiples stages
- `GraphicsPipeline` - Pipeline gráfico declarativo
- `ComputePipeline` - Pipeline de compute

### ✅ Descriptors (Capa 4)
- `DescriptorSetLayout` - Layouts con builder pattern
- `DescriptorPool` - Pool con gestión automática
- `DescriptorSet` - Sets con updates simplificados

### ✅ Commands (Capa 5)
- `CommandPool` - Pools thread-safe
- `CommandBuffer` - Grabación fluida de comandos

### ✅ Synchronization (Capa 6)
- `Fence` - Sincronización CPU-GPU
- `Semaphore` - Sincronización GPU-GPU
- `Barrier` - Memory barriers y layout transitions

### ✅ Rendering (Capa 7)
- `RenderPass` - Render passes declarativos
- `Framebuffer` - Framebuffers con attachments
- `Swapchain` - Gestión de swapchain para presentación

## 💡 Ejemplos

### Buffer Creation
```cpp
auto vertexBuffer = reactor::Buffer::create(allocator)
    .size(sizeof(vertices))
    .usage(reactor::BufferUsage::Vertex | reactor::BufferUsage::TransferDst)
    .memoryType(reactor::MemoryType::DeviceLocal)
    .build();
```

### Pipeline Creation
```cpp
auto pipeline = reactor::GraphicsPipeline::create(device, renderPass)
    .shader(vertShader)
    .shader(fragShader)
    .vertexInput(bindings, attributes)
    .topology(reactor::Topology::TriangleList)
    .cullMode(reactor::CullMode::Back)
    .depthTest(true)
    .blending(reactor::BlendMode::Alpha)
    .viewport(1920.0f, 1080.0f)
    .build();
```

### Command Recording
```cpp
cmd.begin();
cmd.beginRenderPass(renderPass, framebuffer, extent, clearValues);
cmd.bindPipeline(VK_PIPELINE_BIND_POINT_GRAPHICS, pipeline.handle());
cmd.bindVertexBuffers(0, {vertexBuffer.handle()}, {0});
cmd.draw(3);
cmd.endRenderPass();
cmd.end();
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
├── reactor/
│   ├── include/reactor/    # Headers públicos (API)
│   └── src/                # Implementaciones
├── examples/
│   ├── sandbox/            # Ejemplo mínimo
│   └── triangle/           # Hello Triangle
├── ideas.md                # Diseño completo del framework
├── USAGE_GUIDE.md         # Guía de uso detallada
├── ARCHITECTURE.md        # Arquitectura técnica
└── README.md              # Este archivo
```

## 🎯 Roadmap

### ✅ v0.1 (Actual)
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
- [ ] Render graph (frame graph)
- [ ] Multi-threading support
- [ ] Descriptor update templates
- [ ] Timeline semaphores

### 🎨 v0.4
- [ ] Scene graph
- [ ] Entity-Component System
- [ ] Material system
- [ ] Lighting (point, directional, spot)

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

---

<div align="center">

**REACTOR Framework** - Simplificando Vulkan sin sacrificar control

Hecho con ❤️ para la comunidad de desarrollo gráfico

</div>

