# 🏗️ Stack-GPU-OP - Arquitectura Técnica

Documentación técnica completa de la arquitectura del proyecto.

---

## 📊 Visión General

Stack-GPU-OP está construido en capas, donde cada capa depende solo de las capas inferiores:

```
┌─────────────────────────────────────────────┐
│  Layer 8: Scene Graph & Components          │  Futuro
├─────────────────────────────────────────────┤
│  Layer 7: Render Graph & Passes             │  Parcial
├─────────────────────────────────────────────┤
│  Layer 6: Stack-GPU-OP Extensions           │  ⭐ ISR, SDF, RT
├─────────────────────────────────────────────┤
│  Layer 5: Synchronization                   │  ✅ Completo
├─────────────────────────────────────────────┤
│  Layer 4: Command Buffers                   │  ✅ Completo
├─────────────────────────────────────────────┤
│  Layer 3: Descriptor Sets                   │  ✅ Completo
├─────────────────────────────────────────────┤
│  Layer 2: Shaders & Pipelines               │  ✅ Completo
├─────────────────────────────────────────────┤
│  Layer 1: Buffers, Images, Samplers         │  ✅ Completo
├─────────────────────────────────────────────┤
│  Layer 0: VulkanContext & Allocator         │  ✅ Completo
└─────────────────────────────────────────────┘
```

---

## 🎯 Layer 0: Core

### VulkanContext

**Responsabilidad**: Inicialización y gestión de Vulkan

```cpp
class VulkanContext {
    VkInstance instance;
    VkPhysicalDevice physicalDevice;
    VkDevice device;
    VkQueue graphicsQueue;
    VkQueue computeQueue;
    std::shared_ptr<MemoryAllocator> allocator;
};
```

**Características**:
- Auto-detección de Vulkan SDK
- Selección automática de GPU
- Validation layers en debug
- Extension management

### MemoryAllocator

**Responsabilidad**: Gestión unificada de memoria GPU

```cpp
class MemoryAllocator {
    VkDevice device;
    VkPhysicalDevice physicalDevice;
    std::vector<MemoryPool> pools;
};
```

**Características**:
- Pool-based allocation
- Memory type selection automática
- Defragmentation (futuro)
- Statistics tracking

---

## 📦 Layer 1: Resources

### Buffer

**Responsabilidad**: Gestión de buffers GPU

```cpp
class Buffer {
    VkBuffer handle;
    VmaAllocation allocation;
    size_t size;
    BufferUsage usage;
    
    class Builder {
        Builder& size(size_t s);
        Builder& usage(BufferUsage u);
        Builder& memoryType(MemoryType t);
        Buffer build();
    };
};
```

**Tipos**:
- Vertex Buffer
- Index Buffer
- Uniform Buffer
- Storage Buffer

### Image

**Responsabilidad**: Gestión de texturas y render targets

```cpp
class Image {
    VkImage handle;
    VkImageView view;
    VmaAllocation allocation;
    VkFormat format;
    VkExtent3D extent;
};
```

**Características**:
- Mipmapping automático
- Layout transitions
- Multiple formats

---

## 🎨 Layer 2: Shaders & Pipelines

### Shader

**Responsabilidad**: Carga y gestión de shaders SPIR-V

```cpp
class Shader {
    VkShaderModule module;
    ShaderStage stage;
    
    Shader(VkDevice device, const std::string& filepath, ShaderStage stage);
    VkPipelineShaderStageCreateInfo getStageInfo() const;
};
```

### GraphicsPipeline

**Responsabilidad**: Pipeline gráfico declarativo

```cpp
class GraphicsPipeline {
    VkPipeline handle;
    VkPipelineLayout layout;
    
    class Builder {
        Builder& shader(std::shared_ptr<Shader> s);
        Builder& vertexInput(bindings, attributes);
        Builder& topology(Topology t);
        Builder& viewport(float w, float h);
        Builder& cullMode(CullMode m);
        Builder& depthTest(bool enable);
        GraphicsPipeline build();
    };
};
```

---

## 🔗 Layer 3: Descriptors

### DescriptorSetLayout

**Responsabilidad**: Layout de descriptor sets

```cpp
class DescriptorSetLayout {
    VkDescriptorSetLayout handle;
    std::vector<VkDescriptorSetLayoutBinding> bindings;
};
```

### DescriptorSet

**Responsabilidad**: Binding de recursos a shaders

```cpp
class DescriptorSet {
    VkDescriptorSet handle;
    
    void updateBuffer(uint32_t binding, VkBuffer buffer);
    void updateImage(uint32_t binding, VkImageView view, VkSampler sampler);
};
```

---

## 📝 Layer 4: Commands

### CommandBuffer

**Responsabilidad**: Grabación de comandos GPU

```cpp
class CommandBuffer {
    VkCommandBuffer handle;
    
    void begin();
    void end();
    void bindPipeline(VkPipelineBindPoint point, VkPipeline pipeline);
    void bindVertexBuffers(uint32_t first, buffers, offsets);
    void draw(uint32_t vertexCount);
    void drawIndexed(uint32_t indexCount);
};
```

---

## 🔄 Layer 5: Synchronization

### Fence

**Responsabilidad**: Sincronización CPU-GPU

```cpp
class Fence {
    VkFence handle;
    
    void wait();
    void reset();
    bool isSignaled();
};
```

### Semaphore

**Responsabilidad**: Sincronización GPU-GPU

```cpp
class Semaphore {
    VkSemaphore handle;
};
```

---

## ⭐ Layer 6: Stack-GPU-OP Extensions

### ISR (Intelligent Shading Rate)

**Arquitectura**:

```
┌─────────────────────────────────────────┐
│  ISRSystem (Orquestador)                │
├─────────────────────────────────────────┤
│  ImportanceCalculator                   │
│  ├─ Compute Pipeline                    │
│  ├─ importance.comp shader              │
│  └─ Output: Importance Image            │
├─────────────────────────────────────────┤
│  TemporalCoherence                      │
│  ├─ Compute Pipeline                    │
│  ├─ temporal.comp shader                │
│  └─ Output: Smoothed Importance         │
├─────────────────────────────────────────┤
│  AdaptivePixelSizer                     │
│  ├─ Compute Pipeline                    │
│  ├─ adaptive.comp shader                │
│  └─ Output: Shading Rate Image          │
└─────────────────────────────────────────┘
```

**Flujo de Datos**:

```
Scene Render
    ↓
[Color, Depth, Normal, Motion] → ImportanceCalculator
    ↓
Importance Map → TemporalCoherence
    ↓
Smoothed Importance → AdaptivePixelSizer
    ↓
Shading Rate Image → Next Frame Render
```

**Implementación**:

```cpp
class ISRSystem {
    std::unique_ptr<ImportanceCalculator> importance;
    std::unique_ptr<TemporalCoherence> temporal;
    std::unique_ptr<AdaptivePixelSizer> adaptive;
    
    void calculate(CommandBuffer& cmd, const SceneInputs& inputs);
    VkImageView getShadingRateImage() const;
};
```

### SDF (Signed Distance Fields)

**Arquitectura**:

```
┌─────────────────────────────────────────┐
│  SDFScene (Scene Graph)                 │
│  ├─ Primitives (Sphere, Box, etc.)      │
│  ├─ Transforms                          │
│  └─ CSG Operations                      │
├─────────────────────────────────────────┤
│  RayMarcher (Renderer)                  │
│  ├─ Graphics Pipeline                   │
│  ├─ raymarching.vert/frag shaders       │
│  ├─ Uniform Buffers (Camera, Scene)     │
│  └─ Output: Rendered Image              │
└─────────────────────────────────────────┘
```

**Primitivas Soportadas**:

```cpp
enum class SDFPrimitiveType {
    Sphere,    // sdSphere(p, center, radius)
    Box,       // sdBox(p, center, size)
    Torus,     // sdTorus(p, center, r1, r2)
    Cylinder,  // sdCylinder(p, center, radius, height)
    Capsule,   // sdCapsule(p, a, b, radius)
    Cone       // sdCone(p, center, angle, height)
};
```

**CSG Operations**:

```cpp
enum class CSGOperation {
    Union,        // opUnion(d1, d2)
    SmoothUnion,  // opSmoothUnion(d1, d2, k)
    Subtract,     // opSubtract(d1, d2)
    Intersect     // opIntersect(d1, d2)
};
```

---

## 🔄 Flujo de Renderizado Completo

### Frame Rendering Pipeline

```
1. Acquire Swapchain Image
   ↓
2. Wait for Fence (frame in flight)
   ↓
3. Reset Command Buffer
   ↓
4. Begin Command Buffer
   ↓
5. [OPTIONAL] ISR: Calculate Importance
   ↓
6. Begin Render Pass
   ↓
7. Bind Pipeline (Graphics/SDF)
   ↓
8. Bind Descriptor Sets
   ↓
9. Push Constants (MVP, etc.)
   ↓
10. Bind Vertex/Index Buffers
   ↓
11. Draw Commands
   ↓
12. End Render Pass
   ↓
13. End Command Buffer
   ↓
14. Submit to Queue
   ↓
15. Present Swapchain Image
   ↓
16. Advance Frame Index
```

---

## 🎨 Ejemplo Completo: Cubo 3D

### Inicialización

```cpp
// 1. Crear contexto Vulkan
VulkanContext ctx(true);
ctx.init();

// 2. Crear ventana
Window window(ctx, 1280, 720, "Cubo 3D");

// 3. Crear swapchain
Swapchain swapchain(ctx, window);

// 4. Crear render pass
RenderPass renderPass(ctx.device(), attachments);

// 5. Crear cube renderer
CubeRenderer cubeRenderer(ctx, renderPass.handle(), 1280, 720);
```

### Render Loop

```cpp
while (!window.shouldClose()) {
    // Poll events
    window.pollEvents();
    
    // Wait for fence
    inFlight[currentFrame].wait();
    
    // Acquire image
    uint32_t imageIndex = swapchain.acquireNextImage(
        imageAvailable[currentFrame].handle()
    );
    
    // Reset fence
    inFlight[currentFrame].reset();
    
    // Update matrices
    glm::mat4 model = transform.getMatrix();
    glm::mat4 view = camera.getViewMatrix();
    glm::mat4 proj = camera.getProjectionMatrix();
    glm::mat4 mvp = proj * view * model;
    
    // Record commands
    auto& cmd = cmdBuffers[imageIndex];
    cmd.reset();
    cmd.begin();
    
    cmd.beginRenderPass(renderPass, framebuffers[imageIndex]);
    cubeRenderer.render(cmd, mvp);
    cmd.endRenderPass();
    
    cmd.end();
    
    // Submit
    VkSubmitInfo submitInfo{};
    submitInfo.waitSemaphoreCount = 1;
    submitInfo.pWaitSemaphores = &imageAvailable[currentFrame].handle();
    submitInfo.commandBufferCount = 1;
    submitInfo.pCommandBuffers = &cmd.handle();
    submitInfo.signalSemaphoreCount = 1;
    submitInfo.pSignalSemaphores = &renderFinished[currentFrame].handle();
    
    vkQueueSubmit(ctx.graphicsQueue(), 1, &submitInfo, 
                  inFlight[currentFrame].handle());
    
    // Present
    swapchain.present(ctx.graphicsQueue(), imageIndex, 
                     renderFinished[currentFrame].handle());
    
    currentFrame = (currentFrame + 1) % MAX_FRAMES_IN_FLIGHT;
}
```

---

## 📊 Comparación: ADead-GPU vs Stack-GPU-OP

### Mapeo de Componentes

| ADead-GPU (DX12) | Stack-GPU-OP (Vulkan) | Estado |
|------------------|----------------------|--------|
| `ID3D12Device` | `VkDevice` | ✅ |
| `ID3D12CommandQueue` | `VkQueue` | ✅ |
| `ID3D12CommandList` | `VkCommandBuffer` | ✅ |
| `ID3D12Resource` | `VkBuffer/VkImage` | ✅ |
| `ID3D12PipelineState` | `VkPipeline` | ✅ |
| `ID3D12DescriptorHeap` | `VkDescriptorPool` | ✅ |
| `ISR System` | `ISRSystem` | ⏳ 50% |
| `Vector3D` | `SDF Rendering` | ✅ |
| `Ray Tracing` | `Advanced RT` | ⏳ 0% |
| `.gpu Language` | `.gpu → SPIR-V` | ⏳ 0% |

### Extensiones Vulkan Usadas

| Extensión | Propósito | Estado |
|-----------|-----------|--------|
| `VK_KHR_swapchain` | Presentación | ✅ |
| `VK_EXT_fragment_shading_rate` | ISR | ⏳ |
| `VK_KHR_push_descriptor` | Push descriptors | Futuro |
| `VK_KHR_dynamic_rendering` | Render pass dinámico | Futuro |

---

## 🔮 Arquitectura Futura

### Layer 7: Render Graph

```cpp
class RenderGraph {
    struct Pass {
        std::string name;
        std::vector<Resource> inputs;
        std::vector<Resource> outputs;
        std::function<void(CommandBuffer&)> execute;
    };
    
    void addPass(const Pass& pass);
    void compile();
    void execute(CommandBuffer& cmd);
};
```

### Layer 8: Scene Graph

```cpp
class SceneNode {
    Transform transform;
    std::vector<std::shared_ptr<Component>> components;
    std::vector<std::shared_ptr<SceneNode>> children;
};

class Scene {
    std::shared_ptr<SceneNode> root;
    Camera camera;
    std::vector<Light> lights;
};
```

---

<div align="center">

**Stack-GPU-OP Architecture**

*Diseñado para ser simple, potente y extensible*

v0.3.0 - Diciembre 2025

</div>
