# REACTOR Framework - Guía de Uso Completa

## Introducción

REACTOR es un framework moderno para Vulkan que simplifica enormemente el desarrollo de aplicaciones gráficas manteniendo el control total de la API. Este documento te guiará a través de todos los componentes del framework.

## Instalación y Configuración

### Requisitos Previos
- Vulkan SDK instalado y `VULKAN_SDK` configurado
- CMake 3.24 o superior
- Compilador con soporte C++20 (MSVC 2022, GCC 11+, Clang 14+)
- Ninja (opcional pero recomendado)

### Compilación
```bash
# Configurar proyecto
cmake -S . -B build -G "Ninja" -DCMAKE_BUILD_TYPE=Release

# Compilar
cmake --build build

# Ejecutar ejemplo
build\examples\triangle\reactor-triangle.exe
```

## Arquitectura del Framework

REACTOR está organizado en capas modulares:

```
┌─────────────────────────────────────┐
│     Scene Graph & Components        │  Capa 8: Alto nivel
├─────────────────────────────────────┤
│     Render Graph & Passes           │  Capa 7: Rendering
├─────────────────────────────────────┤
│     Synchronization (Fences, etc)   │  Capa 6: Sincronización
├─────────────────────────────────────┤
│     Command Buffers & Pools         │  Capa 5: Comandos
├─────────────────────────────────────┤
│     Descriptor Sets & Layouts       │  Capa 4: Descriptores
├─────────────────────────────────────┤
│     Shaders & Pipelines             │  Capa 3: Pipelines
├─────────────────────────────────────┤
│     Buffers, Images, Samplers       │  Capa 2: Recursos
├─────────────────────────────────────┤
│     VulkanContext & Memory Alloc    │  Capa 1: Core
└─────────────────────────────────────┘
```

## Guía de Componentes

### 1. Inicialización del Contexto

```cpp
#include "reactor/vulkan_context.hpp"

// Crear contexto con validación habilitada
reactor::VulkanContext ctx(true);
ctx.init();

// Acceder a componentes
VkDevice device = ctx.device();
VkPhysicalDevice physicalDevice = ctx.physical();
VkQueue graphicsQueue = ctx.graphicsQueue();
auto allocator = ctx.allocator();

// Limpiar al finalizar
ctx.shutdown();
```

### 2. Gestión de Memoria y Buffers

#### Memory Allocator
El allocator gestiona automáticamente la memoria de GPU:

```cpp
auto allocator = ctx.allocator();

// El allocator se usa internamente por Buffer e Image
// No necesitas interactuar directamente con él
```

#### Buffers
Crear buffers con el patrón builder:

```cpp
#include "reactor/buffer.hpp"

// Buffer de vértices
auto vertexBuffer = reactor::Buffer::create(allocator)
    .size(sizeof(vertices))
    .usage(reactor::BufferUsage::Vertex)
    .memoryType(reactor::MemoryType::HostVisible)
    .build();

// Subir datos
vertexBuffer.upload(vertices.data(), sizeof(vertices));

// O mapear memoria manualmente
vertexBuffer.mapScoped([&](void* data) {
    memcpy(data, vertices.data(), sizeof(vertices));
});

// Buffer de índices
auto indexBuffer = reactor::Buffer::create(allocator)
    .size(sizeof(indices))
    .usage(reactor::BufferUsage::Index)
    .memoryType(reactor::MemoryType::DeviceLocal)
    .build();

// Buffer uniforme
auto uniformBuffer = reactor::Buffer::create(allocator)
    .size(sizeof(UniformData))
    .usage(reactor::BufferUsage::Uniform)
    .memoryType(reactor::MemoryType::HostVisible)
    .build();

// Combinar usos
auto stagingBuffer = reactor::Buffer::create(allocator)
    .size(dataSize)
    .usage(reactor::BufferUsage::TransferSrc | reactor::BufferUsage::TransferDst)
    .memoryType(reactor::MemoryType::HostVisible)
    .build();
```

### 3. Imágenes y Texturas

```cpp
#include "reactor/image.hpp"

// Crear imagen para render target
auto colorImage = reactor::Image::create(allocator)
    .size(1920, 1080)
    .format(reactor::ImageFormat::RGBA8)
    .usage(reactor::ImageUsage::ColorAttachment | reactor::ImageUsage::Sampled)
    .mipLevels(1)
    .build();

// Crear imagen de profundidad
auto depthImage = reactor::Image::create(allocator)
    .size(1920, 1080)
    .format(reactor::ImageFormat::D32F)
    .usage(reactor::ImageUsage::DepthStencilAttachment)
    .build();

// Crear sampler
auto sampler = reactor::Sampler::create(device)
    .filter(reactor::Filter::Linear, reactor::Filter::Linear)
    .addressMode(reactor::AddressMode::Repeat)
    .anisotropy(16.0f)
    .build();
```

### 4. Shaders y Pipelines

#### Cargar Shaders

```cpp
#include "reactor/shader.hpp"

auto vertShader = std::make_shared<reactor::Shader>(
    device, "shaders/vertex.spv", reactor::ShaderStage::Vertex
);

auto fragShader = std::make_shared<reactor::Shader>(
    device, "shaders/fragment.spv", reactor::ShaderStage::Fragment
);
```

#### Graphics Pipeline

```cpp
#include "reactor/pipeline.hpp"

// Definir vertex input
std::vector<reactor::VertexInputBinding> bindings = {
    {0, sizeof(Vertex), VK_VERTEX_INPUT_RATE_VERTEX}
};

std::vector<reactor::VertexInputAttribute> attributes = {
    {0, 0, VK_FORMAT_R32G32_SFLOAT, offsetof(Vertex, pos)},
    {1, 0, VK_FORMAT_R32G32B32_SFLOAT, offsetof(Vertex, color)}
};

// Crear pipeline
auto pipeline = reactor::GraphicsPipeline::create(device, renderPass)
    .shader(vertShader)
    .shader(fragShader)
    .vertexInput(bindings, attributes)
    .topology(reactor::Topology::TriangleList)
    .polygonMode(reactor::PolygonMode::Fill)
    .cullMode(reactor::CullMode::Back)
    .depthTest(true)
    .depthWrite(true)
    .blending(reactor::BlendMode::Alpha)
    .viewport(1920.0f, 1080.0f)
    .build();
```

#### Compute Pipeline

```cpp
auto computeShader = std::make_shared<reactor::Shader>(
    device, "shaders/compute.spv", reactor::ShaderStage::Compute
);

auto computePipeline = reactor::ComputePipeline::create(device)
    .shader(computeShader)
    .descriptorSetLayouts(layouts)
    .build();
```

### 5. Descriptors

```cpp
#include "reactor/descriptor.hpp"

// Crear descriptor set layout
auto layout = reactor::DescriptorSetLayout::create(device)
    .binding(0, reactor::DescriptorType::UniformBuffer, 
             VK_SHADER_STAGE_VERTEX_BIT)
    .binding(1, reactor::DescriptorType::CombinedImageSampler, 
             VK_SHADER_STAGE_FRAGMENT_BIT)
    .build();

// Crear descriptor pool
auto pool = std::make_shared<reactor::DescriptorPool>(
    reactor::DescriptorPool::create(device)
        .maxSets(100)
        .poolSize(reactor::DescriptorType::UniformBuffer, 100)
        .poolSize(reactor::DescriptorType::CombinedImageSampler, 100)
        .build()
);

// Allocar descriptor set
auto descriptorSet = reactor::DescriptorSet(pool, layout.handle());

// Actualizar descriptors
descriptorSet.updateBuffer(0, uniformBuffer.handle(), 0, sizeof(UniformData));
descriptorSet.updateImage(1, texture.view(), sampler.handle(), 
                          VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL);
```

### 6. Render Pass y Framebuffers

```cpp
#include "reactor/render_pass.hpp"

// Crear render pass
auto renderPass = reactor::RenderPass::create(device)
    .colorAttachment(VK_FORMAT_B8G8R8A8_UNORM, VK_IMAGE_LAYOUT_PRESENT_SRC_KHR)
    .depthAttachment(VK_FORMAT_D32_SFLOAT)
    .build();

// Crear framebuffer
std::vector<VkImageView> attachments = {
    colorImageView,
    depthImageView
};

auto framebuffer = reactor::Framebuffer(
    device, renderPass.handle(), attachments, 1920, 1080
);
```

### 7. Command Buffers

```cpp
#include "reactor/command_buffer.hpp"

// Crear command pool
auto commandPool = std::make_shared<reactor::CommandPool>(
    device, queueFamilyIndex, false
);

// Crear command buffer
auto cmd = reactor::CommandBuffer(commandPool);

// Grabar comandos
cmd.begin();

// Comenzar render pass
std::vector<VkClearValue> clearValues(2);
clearValues[0].color = {{0.0f, 0.0f, 0.0f, 1.0f}};
clearValues[1].depthStencil = {1.0f, 0};

cmd.beginRenderPass(renderPass.handle(), framebuffer.handle(), 
                    {1920, 1080}, clearValues);

// Bind pipeline y recursos
cmd.bindPipeline(VK_PIPELINE_BIND_POINT_GRAPHICS, pipeline.handle());
cmd.bindVertexBuffers(0, {vertexBuffer.handle()}, {0});
cmd.bindIndexBuffer(indexBuffer.handle(), 0, VK_INDEX_TYPE_UINT32);
cmd.bindDescriptorSets(VK_PIPELINE_BIND_POINT_GRAPHICS, 
                       pipeline.layout(), 0, {descriptorSet.handle()});

// Configurar viewport y scissor
cmd.setViewport(0, 0, 1920, 1080);
cmd.setScissor(0, 0, 1920, 1080);

// Draw
cmd.drawIndexed(indexCount);

cmd.endRenderPass();
cmd.end();
```

### 8. Sincronización

```cpp
#include "reactor/sync.hpp"

// Crear fence
reactor::Fence fence(device, false);

// Crear semaphore
reactor::Semaphore imageAvailable(device);
reactor::Semaphore renderFinished(device);

// Esperar fence
fence.wait();
fence.reset();

// Pipeline barriers para transiciones de layout
std::vector<reactor::ImageBarrier> barriers = {
    {
        image.handle(),
        VK_IMAGE_LAYOUT_UNDEFINED,
        VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
        0,
        VK_ACCESS_SHADER_READ_BIT,
        VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT,
        VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT
    }
};

cmd.pipelineBarrier(barriers);
```

### 9. Swapchain (Presentación)

```cpp
#include "reactor/swapchain.hpp"

// Crear swapchain
auto swapchain = reactor::Swapchain(
    device, physicalDevice, surface, 1920, 1080, true
);

// Adquirir imagen
uint32_t imageIndex = swapchain.acquireNextImage(imageAvailable.handle());

// ... renderizar a la imagen ...

// Presentar
swapchain.present(graphicsQueue, imageIndex, renderFinished.handle());
```

## Patrones de Uso Comunes

### Patrón 1: Inicialización Completa

```cpp
// 1. Crear contexto
reactor::VulkanContext ctx(true);
ctx.init();

// 2. Crear recursos
auto allocator = ctx.allocator();
auto vertexBuffer = reactor::Buffer::create(allocator)...build();
auto pipeline = reactor::GraphicsPipeline::create(...)...build();

// 3. Crear command buffers
auto commandPool = std::make_shared<reactor::CommandPool>(...);
auto cmd = reactor::CommandBuffer(commandPool);

// 4. Crear sincronización
reactor::Fence fence(ctx.device());
reactor::Semaphore semaphore(ctx.device());

// 5. Render loop
while (running) {
    cmd.begin();
    // ... grabar comandos ...
    cmd.end();
    
    // Submit
    // Present
}

// 6. Cleanup
ctx.shutdown();
```

### Patrón 2: Transferencia de Datos a GPU

```cpp
// Crear staging buffer
auto staging = reactor::Buffer::create(allocator)
    .size(dataSize)
    .usage(reactor::BufferUsage::TransferSrc)
    .memoryType(reactor::MemoryType::HostVisible)
    .build();

// Crear buffer de destino en GPU
auto gpuBuffer = reactor::Buffer::create(allocator)
    .size(dataSize)
    .usage(reactor::BufferUsage::Vertex | reactor::BufferUsage::TransferDst)
    .memoryType(reactor::MemoryType::DeviceLocal)
    .build();

// Subir a staging
staging.upload(data, dataSize);

// Copiar a GPU
cmd.begin(true);
cmd.copyBuffer(staging.handle(), gpuBuffer.handle(), dataSize);
cmd.end();

// Submit y esperar
```

### Patrón 3: Multi-Pass Rendering

```cpp
// Pass 1: Shadow map
cmd.beginRenderPass(shadowPass, shadowFramebuffer, extent, clearValues);
cmd.bindPipeline(VK_PIPELINE_BIND_POINT_GRAPHICS, shadowPipeline.handle());
// ... render scene from light perspective ...
cmd.endRenderPass();

// Barrier para transición
cmd.pipelineBarrier(shadowMapBarriers);

// Pass 2: Main render
cmd.beginRenderPass(mainPass, mainFramebuffer, extent, clearValues);
cmd.bindPipeline(VK_PIPELINE_BIND_POINT_GRAPHICS, mainPipeline.handle());
// ... render scene con sombras ...
cmd.endRenderPass();
```

## Ventajas del Framework

### 1. RAII Automático
Todos los recursos se limpian automáticamente:
```cpp
{
    auto buffer = reactor::Buffer::create(allocator)...build();
    // Usar buffer
} // Buffer destruido automáticamente aquí
```

### 2. Builder Pattern
API fluida y legible:
```cpp
auto pipeline = reactor::GraphicsPipeline::create(device, renderPass)
    .shader(vertShader)
    .shader(fragShader)
    .depthTest(true)
    .blending(reactor::BlendMode::Alpha)
    .build();
```

### 3. Type Safety
Enums fuertemente tipados previenen errores:
```cpp
reactor::BufferUsage::Vertex  // No int mágicos
reactor::ImageFormat::RGBA8   // Formatos claros
reactor::BlendMode::Alpha     // Modos predefinidos
```

### 4. Gestión de Memoria Simplificada
El allocator maneja toda la complejidad:
```cpp
// No necesitas:
// - vkAllocateMemory
// - vkBindBufferMemory
// - vkFreeMemory
// Todo es automático
```

## Debugging y Validación

### Habilitar Validation Layers
```cpp
reactor::VulkanContext ctx(true);  // true = validación habilitada
```

### Nombrar Recursos (para RenderDoc/Nsight)
```cpp
// TODO: Agregar en futuras versiones
// buffer.setDebugName("Vertex Buffer - Cube");
// pipeline.setDebugName("Main Render Pipeline");
```

## Performance Tips

1. **Reutilizar Command Buffers**: No crear/destruir cada frame
2. **Batch Descriptor Updates**: Actualizar múltiples descriptors a la vez
3. **Memory Pooling**: Usar el allocator para suballocación eficiente
4. **Pipeline Cache**: Cachear pipelines entre ejecuciones
5. **Staging Buffers**: Usar para transferencias grandes a GPU

## Próximos Pasos

1. Revisar `examples/triangle` para un ejemplo completo
2. Leer `ideas.md` para la arquitectura completa
3. Explorar los headers en `reactor/include/reactor/`
4. Construir tu primera aplicación con REACTOR

## Soporte y Contribuciones

- Issues: Reportar en el repositorio
- Documentación: Ver `ideas.md` y headers
- Ejemplos: Directorio `examples/`

---

**REACTOR Framework** - Simplificando Vulkan sin sacrificar control
