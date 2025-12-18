# REACTOR Framework - Arquitectura Técnica Detallada

## Visión General

REACTOR es un framework de abstracción para Vulkan diseñado con los siguientes principios:

1. **Zero-cost abstraction**: No overhead en runtime
2. **RAII everywhere**: Gestión automática de recursos
3. **Builder pattern**: API declarativa y fluida
4. **Type safety**: Enums fuertemente tipados
5. **Composability**: Componentes pequeños y reutilizables

## Estructura de Directorios

```
REACTOR/
├── reactor/
│   ├── include/reactor/          # Headers públicos
│   │   ├── reactor.hpp           # API principal
│   │   ├── vulkan_context.hpp    # Contexto Vulkan
│   │   ├── memory_allocator.hpp  # Gestión de memoria
│   │   ├── buffer.hpp            # Buffers
│   │   ├── image.hpp             # Imágenes y samplers
│   │   ├── shader.hpp            # Shaders SPIR-V
│   │   ├── pipeline.hpp          # Pipelines gráficos/compute
│   │   ├── descriptor.hpp        # Descriptor sets
│   │   ├── command_buffer.hpp    # Command buffers
│   │   ├── sync.hpp              # Sincronización
│   │   ├── render_pass.hpp       # Render passes
│   │   └── swapchain.hpp         # Swapchain
│   └── src/                      # Implementaciones
├── examples/                     # Ejemplos
│   ├── sandbox/                  # Ejemplo básico
│   └── triangle/                 # Hello Triangle
├── ideas.md                      # Diseño y roadmap
├── USAGE_GUIDE.md               # Guía de uso
├── ARCHITECTURE.md              # Este archivo
└── README.md                    # Introducción

```

## Capas del Framework

### Capa 1: Core (Fundación)

#### VulkanContext
**Responsabilidad**: Inicialización y gestión del contexto Vulkan

**Componentes**:
- `VkInstance`: Instancia de Vulkan
- `VkPhysicalDevice`: GPU seleccionada
- `VkDevice`: Logical device
- `VkQueue`: Cola de comandos
- `MemoryAllocator`: Allocator de memoria

**Flujo de Inicialización**:
```
1. createInstance() → Crear VkInstance con validation layers
2. pickPhysicalDevice() → Seleccionar GPU adecuada
3. createDevice() → Crear logical device y obtener queues
4. Crear MemoryAllocator → Inicializar gestión de memoria
```

**Decisiones de Diseño**:
- Una sola queue family (graphics) por simplicidad inicial
- Validation layers opcionales via constructor
- API 1.3 como mínimo
- Extensiones hardcodeadas (futuro: configurables)

#### MemoryAllocator
**Responsabilidad**: Gestión unificada de memoria GPU

**Estrategia**:
- Allocación directa por ahora (futuro: suballocación)
- Tipos de memoria predefinidos (DeviceLocal, HostVisible, etc.)
- Thread-safe con mutex
- Tracking de allocaciones para debugging

**Tipos de Memoria**:
```cpp
enum class MemoryType {
    DeviceLocal,    // GPU only (fastest)
    HostVisible,    // CPU→GPU (staging)
    HostCoherent,   // CPU↔GPU (uniforms)
    HostCached      // GPU→CPU (readback)
};
```

### Capa 2: Resources (Recursos)

#### Buffer
**Características**:
- Builder pattern para creación
- RAII: destrucción automática
- Upload helper para datos
- Map/unmap con lambda scoped
- Combinación de usage flags

**Lifecycle**:
```
create() → Builder → build() → Buffer
                                  ↓
                            upload(data)
                                  ↓
                            [uso en comandos]
                                  ↓
                            ~Buffer() [auto cleanup]
```

#### Image
**Características**:
- Soporte para 2D images
- Mipmapping (futuro: generación automática)
- Image view creado automáticamente
- Transiciones de layout (futuro: automáticas)

**Formatos Soportados**:
- RGBA8, RGBA16F, RGBA32F (color)
- D32F, D24S8 (depth/stencil)
- BGRA8 (swapchain)

#### Sampler
**Características**:
- Filtrado (Nearest, Linear)
- Address modes (Repeat, Clamp, etc.)
- Anisotropic filtering
- Mipmap modes

### Capa 3: Shaders & Pipelines

#### Shader
**Características**:
- Carga de SPIR-V desde archivo
- Reflection (futuro: automática)
- Hot-reload (futuro)
- Múltiples stages

**Stages Soportados**:
- Vertex, Fragment (básicos)
- Compute
- Geometry, Tessellation (futuro)

#### GraphicsPipeline
**Builder Pattern**:
```cpp
GraphicsPipeline::create(device, renderPass)
    .shader(vert)
    .shader(frag)
    .vertexInput(bindings, attributes)
    .topology(Triangles)
    .cullMode(Back)
    .depthTest(true)
    .blending(Alpha)
    .build()
```

**Estado Configurable**:
- Vertex input (bindings + attributes)
- Topology (points, lines, triangles)
- Rasterization (polygon mode, cull mode)
- Depth/stencil testing
- Blending (None, Alpha, Additive, Multiply)
- Viewport/scissor (dynamic en futuro)

#### ComputePipeline
**Más Simple**:
- Solo compute shader
- Descriptor layouts
- Push constants

### Capa 4: Descriptors

#### DescriptorSetLayout
**Builder Pattern**:
```cpp
DescriptorSetLayout::create(device)
    .binding(0, UniformBuffer, VERTEX_STAGE)
    .binding(1, CombinedImageSampler, FRAGMENT_STAGE)
    .build()
```

#### DescriptorPool
**Gestión**:
- Pool con tamaños predefinidos
- Reset para reutilización
- Free individual de sets

#### DescriptorSet
**Updates**:
- `updateBuffer()`: Para uniform/storage buffers
- `updateImage()`: Para texturas
- Batch updates (futuro)

### Capa 5: Commands

#### CommandPool
**Características**:
- Por queue family
- Reset de pool completo
- Thread-local pools (futuro)

#### CommandBuffer
**API Fluida**:
```cpp
cmd.begin();
cmd.beginRenderPass(...);
cmd.bindPipeline(...);
cmd.bindVertexBuffers(...);
cmd.draw(...);
cmd.endRenderPass();
cmd.end();
```

**Comandos Soportados**:
- Render pass begin/end
- Pipeline binding
- Resource binding (vertex, index, descriptors)
- Draw calls (draw, drawIndexed)
- Compute dispatch
- Copy operations (buffer, image)
- Pipeline barriers
- Dynamic state (viewport, scissor)
- Push constants

### Capa 6: Synchronization

#### Fence
**CPU-GPU Sync**:
- Wait con timeout
- Reset para reutilización
- Query de estado

#### Semaphore
**GPU-GPU Sync**:
- Binary semaphores
- Timeline semaphores (futuro)

#### Barriers
**Memory & Layout Transitions**:
- Image barriers (layout transitions)
- Buffer barriers (memory dependencies)
- Pipeline stage synchronization

### Capa 7: Rendering

#### RenderPass
**Builder Pattern**:
```cpp
RenderPass::create(device)
    .colorAttachment(format, finalLayout)
    .depthAttachment(depthFormat)
    .build()
```

**Características**:
- Múltiples color attachments
- Depth/stencil attachment opcional
- Subpass dependencies automáticas
- Load/store ops configurables

#### Framebuffer
**Simple Wrapper**:
- Asocia image views a render pass
- Dimensiones fijas
- Recreación para resize

#### Swapchain
**Presentación**:
- Creación con surface
- VSync configurable
- Acquire/present helpers
- Recreación para resize (futuro)

### Capa 8: Scene (Futuro)

Componentes planeados:
- Entity-Component System
- Transform hierarchy
- Camera system
- Material system
- Lighting
- Culling

## Patrones de Diseño Utilizados

### 1. RAII (Resource Acquisition Is Initialization)
```cpp
class Buffer {
    ~Buffer() {
        if (buffer != VK_NULL_HANDLE) {
            vkDestroyBuffer(device, buffer, nullptr);
        }
        allocator->free(memory);
    }
};
```

**Ventajas**:
- No memory leaks
- Exception safe
- Orden de destrucción correcto

### 2. Builder Pattern
```cpp
class Buffer {
    class Builder {
        Builder& size(VkDeviceSize s) { 
            bufSize = s; 
            return *this; 
        }
        Buffer build() { 
            return Buffer(...); 
        }
    };
};
```

**Ventajas**:
- API fluida y legible
- Validación en build()
- Parámetros opcionales claros

### 3. Move Semantics
```cpp
Buffer(Buffer&& other) noexcept {
    buffer = other.buffer;
    other.buffer = VK_NULL_HANDLE;
}
```

**Ventajas**:
- Zero-copy transfers
- Ownership claro
- No copias accidentales

### 4. Smart Pointers
```cpp
std::shared_ptr<CommandPool> pool;
std::shared_ptr<MemoryAllocator> allocator;
```

**Ventajas**:
- Lifetime management automático
- Shared ownership donde necesario
- No dangling pointers

## Decisiones de Diseño Importantes

### 1. ¿Por qué Builder Pattern?
**Alternativas consideradas**:
- Constructores con muchos parámetros ❌
- Structs de configuración ✓ (menos fluido)
- Builder pattern ✓✓ (elegido)

**Razón**: API más legible y flexible

### 2. ¿Por qué RAII en vez de handles?
**Alternativas**:
- Handles raw de Vulkan ❌
- Wrappers RAII ✓✓

**Razón**: Seguridad y conveniencia sin overhead

### 3. ¿Por qué no VMA (Vulkan Memory Allocator)?
**Decisión**: Implementación propia simple por ahora

**Futuro**: Integrar VMA como opción

### 4. ¿Por qué C++20?
**Características usadas**:
- Concepts (futuro)
- Ranges (futuro)
- Modules (futuro)
- Designated initializers

## Performance Considerations

### Zero-Cost Abstractions
```cpp
// Esto:
auto buffer = Buffer::create(allocator)
    .size(1024)
    .usage(BufferUsage::Vertex)
    .build();

// Compila a lo mismo que:
VkBufferCreateInfo info = {...};
vkCreateBuffer(device, &info, nullptr, &buffer);
```

### Inline Everything
Headers con implementaciones inline donde posible

### No Virtual Functions
Sin overhead de vtables

### Move Semantics
Evitar copias innecesarias

## Testing Strategy (Futuro)

### Unit Tests
- Cada componente aislado
- Mock de Vulkan (vk-layer-mock)

### Integration Tests
- Pipelines completos
- Render passes

### Performance Tests
- Benchmarks de allocación
- Command buffer recording

## Extensibilidad

### Agregar Nuevos Componentes
1. Crear header en `reactor/include/reactor/`
2. Implementar en `reactor/src/`
3. Agregar a CMakeLists.txt
4. Documentar en USAGE_GUIDE.md
5. Crear ejemplo

### Agregar Nuevas Features
1. Diseñar API en `ideas.md`
2. Implementar con tests
3. Documentar
4. Ejemplo de uso

## Roadmap Técnico

### v0.2 (Próximo)
- [ ] Window integration (GLFW)
- [ ] Swapchain resize
- [ ] Staging buffer pool
- [ ] Descriptor update templates

### v0.3
- [ ] Render graph
- [ ] Multi-threading
- [ ] Compute integration
- [ ] Timeline semaphores

### v0.4
- [ ] Scene graph
- [ ] Material system
- [ ] Lighting
- [ ] Shadows

### v1.0
- [ ] Ray tracing
- [ ] Mesh shaders
- [ ] Variable rate shading
- [ ] Production ready

## Conclusión

REACTOR está diseñado para ser:
- **Simple**: API intuitiva
- **Seguro**: RAII y type safety
- **Rápido**: Zero-cost abstractions
- **Extensible**: Arquitectura modular
- **Completo**: Todas las features de Vulkan

El framework crece orgánicamente, agregando features según se necesitan, siempre manteniendo la filosofía de control total con conveniencia máxima.
