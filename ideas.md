# REACTOR Framework - Arquitectura y Diseño Completo

## Filosofía del Framework

REACTOR es un framework para Vulkan que combina:
- **Control total**: Acceso directo a todas las capacidades de Vulkan
- **Declaratividad**: API inspirada en React para describir recursos y operaciones
- **Componibilidad**: Construcción de escenas complejas mediante composición
- **Seguridad**: Gestión automática de ciclo de vida y sincronización

## Principios de Diseño

### 1. Declarativo pero Explícito
```cpp
// Declarar recursos de forma clara
auto buffer = Buffer::create()
    .size(1024)
    .usage(BufferUsage::Vertex | BufferUsage::Transfer)
    .memoryType(MemoryType::DeviceLocal)
    .build();

// Pero con control explícito cuando se necesita
buffer.map([](void* data) {
    // Acceso directo a memoria
});
```

### 2. Composición sobre Herencia
- Componentes pequeños y reutilizables
- Sistema de entidades basado en composición
- Pipelines construidos mediante builder pattern

### 3. RAII y Gestión Automática
- Todos los recursos Vulkan envueltos en clases RAII
- Destrucción automática en orden correcto
- Prevención de leaks mediante smart pointers

### 4. Sincronización Simplificada
- Barreras de memoria automáticas cuando sea posible
- API declarativa para dependencias entre comandos
- Gestión de timeline semaphores para sincronización avanzada

## Arquitectura del Framework

### Capa 1: Core (Contexto Vulkan)
```
VulkanContext
├── Instance (validación, extensiones)
├── PhysicalDevice (selección, propiedades)
├── Device (logical device, queues)
└── Allocator (gestión de memoria unificada)
```

**Responsabilidades**:
- Inicialización de Vulkan
- Selección de dispositivo físico
- Creación de logical device
- Gestión de colas (graphics, compute, transfer)
- Allocator de memoria (VMA-style)

### Capa 2: Resources (Gestión de Recursos)
```
ResourceManager
├── Buffer (vertex, index, uniform, storage)
├── Image (textures, render targets)
├── Sampler (filtrado, wrapping)
└── Memory (allocation, mapping, staging)
```

**Características**:
- Buffer staging automático para transferencias
- Mipmapping automático para texturas
- Pool de recursos reutilizables
- Transiciones de layout automáticas

### Capa 3: Shaders & Pipelines
```
ShaderManager
├── ShaderModule (SPIR-V loading, reflection)
├── PipelineLayout (descriptor sets, push constants)
└── Pipeline (graphics, compute)
    ├── GraphicsPipeline (vertex, fragment, geometry)
    └── ComputePipeline (compute shaders)
```

**Características**:
- Hot-reload de shaders en desarrollo
- Reflection automática de SPIR-V
- Cache de pipelines
- Variantes de pipeline (depth test, blending, etc.)

### Capa 4: Descriptors
```
DescriptorManager
├── DescriptorSetLayout (binding definitions)
├── DescriptorPool (allocation pool)
└── DescriptorSet (bindings actuales)
```

**Características**:
- Allocación dinámica de descriptor sets
- Bindless rendering support
- Update batching automático
- Pool recycling

### Capa 5: Commands
```
CommandManager
├── CommandPool (per-thread pools)
├── CommandBuffer (recording, submission)
└── CommandGraph (dependency tracking)
```

**Características**:
- Command buffer recycling
- Grabación multi-thread
- Dependency graph automático
- Batching de submissions

### Capa 6: Synchronization
```
SyncManager
├── Fence (CPU-GPU sync)
├── Semaphore (GPU-GPU sync)
├── Barrier (memory barriers, layout transitions)
└── Timeline (timeline semaphores)
```

**Características**:
- Sincronización automática entre frames
- Pipeline barriers optimizadas
- Timeline semaphores para dependencias complejas
- Frame pacing

### Capa 7: Rendering
```
RenderSystem
├── RenderPass (attachments, subpasses)
├── Framebuffer (render targets)
├── Swapchain (presentación)
└── RenderGraph (frame graph execution)
```

**Características**:
- Render passes declarativos
- Automatic subpass dependencies
- Swapchain recreation automática
- Frame graph para optimización

### Capa 8: Scene (Alto Nivel)
```
SceneGraph
├── Entity (game objects)
├── Component (transform, mesh, material)
├── Camera (view, projection)
└── Light (point, directional, spot)
```

**Características**:
- Sistema de entidades componible
- Culling automático (frustum, occlusion)
- LOD management
- Material system

## Patrones de Uso

### Patrón 1: Inicialización Simple
```cpp
auto app = Reactor::create()
    .validation(true)
    .window(1920, 1080, "Mi App")
    .build();

app.run([](Frame& frame) {
    // Render loop
});
```

### Patrón 2: Recursos Declarativos
```cpp
auto mesh = Mesh::create()
    .vertices(vertices)
    .indices(indices)
    .build();

auto texture = Texture::load("texture.png")
    .mipmaps(true)
    .filter(Filter::Linear)
    .build();
```

### Patrón 3: Pipeline Builder
```cpp
auto pipeline = GraphicsPipeline::create()
    .shader("vertex.spv", ShaderStage::Vertex)
    .shader("fragment.spv", ShaderStage::Fragment)
    .vertexInput<Vertex>()
    .topology(Topology::Triangles)
    .depthTest(true)
    .blending(BlendMode::Alpha)
    .build();
```

### Patrón 4: Render Graph
```cpp
auto graph = RenderGraph::create();

auto gbuffer = graph.addPass("GBuffer")
    .output("position", Format::RGBA16F)
    .output("normal", Format::RGBA16F)
    .output("albedo", Format::RGBA8)
    .output("depth", Format::D32F)
    .execute([](CommandBuffer& cmd, Resources& res) {
        // Geometry pass
    });

auto lighting = graph.addPass("Lighting")
    .input("position", gbuffer)
    .input("normal", gbuffer)
    .input("albedo", gbuffer)
    .output("color", Format::RGBA8)
    .execute([](CommandBuffer& cmd, Resources& res) {
        // Lighting pass
    });

graph.compile();
graph.execute();
```

### Patrón 5: Componentes de Escena
```cpp
auto entity = scene.createEntity("Cube");
entity.add<Transform>()
    .position(0, 0, 0)
    .rotation(0, 45, 0)
    .scale(1, 1, 1);

entity.add<MeshRenderer>()
    .mesh(cubeMesh)
    .material(material);

entity.add<PointLight>()
    .color(1, 1, 1)
    .intensity(10.0f)
    .radius(5.0f);
```

## Características Avanzadas

### 1. Multi-Threading
- Command buffer recording paralelo
- Resource loading asíncrono
- Compute dispatch independiente

### 2. Compute Shaders
```cpp
auto computePipeline = ComputePipeline::create()
    .shader("compute.spv")
    .build();

cmd.dispatch(computePipeline)
    .bind("inputBuffer", buffer1)
    .bind("outputBuffer", buffer2)
    .workgroups(width/16, height/16, 1)
    .execute();
```

### 3. Ray Tracing (Extensión)
```cpp
auto rtPipeline = RayTracingPipeline::create()
    .raygenShader("raygen.spv")
    .missShader("miss.spv")
    .closestHitShader("closesthit.spv")
    .maxRecursion(4)
    .build();

auto tlas = AccelerationStructure::createTLAS()
    .instances(instances)
    .build();
```

### 4. Debugging & Profiling
```cpp
// Debug markers
cmd.beginDebugLabel("Shadow Pass", Color::Red);
// ... render commands
cmd.endDebugLabel();

// GPU timestamps
auto query = QueryPool::create(QueryType::Timestamp, 100);
cmd.writeTimestamp(query, 0);
// ... work
cmd.writeTimestamp(query, 1);
```

## Optimizaciones Integradas

### 1. Memory Management
- Suballocación de memoria (VMA-style)
- Staging buffer pool
- Garbage collection diferido

### 2. Descriptor Management
- Descriptor indexing (bindless)
- Update templates
- Pool recycling

### 3. Command Buffers
- Secondary command buffers para reutilización
- Command buffer inheritance
- Multi-threaded recording

### 4. Pipeline Cache
- Serialización de pipeline cache
- Warm-up automático
- Shader variants

## Integración con Herramientas

### 1. RenderDoc
- Capture automático en debug
- Frame markers
- Resource naming

### 2. Validation Layers
- Best practices validation
- Synchronization validation
- GPU-assisted validation

### 3. Profilers
- Nsight Graphics support
- PIX support
- Custom profiling markers

## Roadmap de Implementación

### Fase 1: Core (Semana 1-2)
- [x] VulkanContext básico
- [ ] Memory allocator
- [ ] Buffer management
- [ ] Image management

### Fase 2: Pipelines (Semana 3-4)
- [ ] Shader loading y reflection
- [ ] Graphics pipeline builder
- [ ] Compute pipeline builder
- [ ] Pipeline cache

### Fase 3: Rendering (Semana 5-6)
- [ ] Render pass system
- [ ] Swapchain management
- [ ] Command buffer recording
- [ ] Synchronization primitives

### Fase 4: Scene (Semana 7-8)
- [ ] Entity-component system
- [ ] Camera system
- [ ] Material system
- [ ] Lighting

### Fase 5: Advanced (Semana 9-10)
- [ ] Render graph
- [ ] Multi-threading
- [ ] Compute integration
- [ ] Profiling tools

## Ejemplos a Implementar

1. **Hello Triangle**: Triángulo básico con color
2. **Textured Cube**: Cubo con textura y rotación
3. **Lighting**: Phong lighting con múltiples luces
4. **Deferred Rendering**: G-buffer y lighting pass
5. **Compute Particles**: Sistema de partículas con compute
6. **Shadow Mapping**: Sombras con shadow maps
7. **PBR Materials**: Materiales physically-based
8. **Post-Processing**: Bloom, tone mapping, FXAA
9. **Instancing**: Renderizado de múltiples objetos
10. **Ray Tracing**: Path tracing básico

## API Reference (Resumen)

### Core
- `Reactor::create()` - Inicialización del framework
- `VulkanContext` - Contexto Vulkan global
- `Device` - Logical device wrapper

### Resources
- `Buffer` - Buffer de GPU
- `Image` - Textura/render target
- `Mesh` - Geometría (vertex + index buffers)
- `Texture` - Image con sampler

### Shaders & Pipelines
- `Shader` - Módulo de shader SPIR-V
- `GraphicsPipeline` - Pipeline gráfico
- `ComputePipeline` - Pipeline de compute
- `PipelineLayout` - Layout de descriptores

### Commands
- `CommandBuffer` - Buffer de comandos
- `CommandPool` - Pool de command buffers
- `Queue` - Cola de comandos

### Sync
- `Fence` - CPU-GPU sync
- `Semaphore` - GPU-GPU sync
- `Barrier` - Memory barrier

### Rendering
- `RenderPass` - Render pass
- `Framebuffer` - Framebuffer
- `Swapchain` - Swapchain para presentación
- `RenderGraph` - Frame graph

### Scene
- `Entity` - Entidad de escena
- `Transform` - Componente de transformación
- `Camera` - Cámara
- `Light` - Luz
- `Material` - Material

## Conclusión

REACTOR busca ser el framework definitivo para Vulkan, combinando:
- **Facilidad de uso** de engines modernos
- **Control total** de Vulkan puro
- **Performance** sin overhead
- **Flexibilidad** para cualquier tipo de aplicación

El objetivo es reducir el tiempo de desarrollo de aplicaciones Vulkan de semanas a días, sin sacrificar rendimiento ni control.
