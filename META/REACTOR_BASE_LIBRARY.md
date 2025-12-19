# 📚 REACTOR - Biblioteca Base Completa (A)

**REACTOR es la biblioteca base (A)** que contiene TODAS las funcionalidades fundamentales para que proyectos B, C, D, etc. puedan heredar y construir sobre ella sin problemas.

---

## 🎯 Concepto: A → B → C

```
┌─────────────────────────────────────┐
│  REACTOR (A) - Biblioteca Base      │
│  ✅ Vulkan Context                  │
│  ✅ Memory Management                │
│  ✅ Buffers & Images                 │
│  ✅ Shaders & Pipelines              │
│  ✅ Compute Pipelines                │
│  ✅ Descriptor Management            │
│  ✅ Command Buffers                  │
│  ✅ Synchronization                  │
│  ✅ Render Passes                    │
│  ✅ Swapchain                        │
│  ✅ Window Management                │
│  ✅ SDF System (Killer Triangle)     │
│  ✅ ISR System                       │
│  ✅ Math Utilities                   │
└─────────────────────────────────────┘
              ↓ hereda
┌─────────────────────────────────────┐
│  Proyecto B - Tu Motor Gráfico      │
│  Hereda TODO de REACTOR             │
│  + Tus features específicas         │
└─────────────────────────────────────┘
              ↓ hereda
┌─────────────────────────────────────┐
│  Proyecto C - Tu Juego/App          │
│  Hereda de B (que hereda de A)      │
│  + Gameplay, UI, etc.               │
└─────────────────────────────────────┘
```

---

## ✅ Componentes Completos en REACTOR

### 1. **Core Vulkan** (100%)

#### VulkanContext
```cpp
#include <reactor/reactor.hpp>

reactor::VulkanContext ctx(true); // validation layers
ctx.init();

// Acceso a todo
VkDevice device = ctx.device();
VkPhysicalDevice physical = ctx.physical();
VkQueue graphicsQueue = ctx.graphicsQueue();
VkQueue computeQueue = ctx.computeQueue();
```

#### Memory Management
```cpp
reactor::MemoryAllocator* allocator = ctx.allocator();

// Allocar memoria automáticamente
auto block = allocator->allocate(
    memRequirements,
    reactor::MemoryType::DeviceLocal
);
```

#### Buffers
```cpp
// Vertex buffer
reactor::Buffer vertexBuffer(
    ctx,
    sizeof(vertices),
    VK_BUFFER_USAGE_VERTEX_BUFFER_BIT,
    reactor::MemoryType::HostVisible
);
vertexBuffer.upload(vertices, sizeof(vertices));

// Uniform buffer
reactor::Buffer uniformBuffer(
    ctx,
    sizeof(UniformData),
    VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT,
    reactor::MemoryType::HostVisible
);
```

#### Images
```cpp
reactor::Image texture(
    ctx,
    width, height,
    VK_FORMAT_R8G8B8A8_UNORM,
    VK_IMAGE_USAGE_SAMPLED_BIT | VK_IMAGE_USAGE_TRANSFER_DST_BIT
);
```

### 2. **Pipelines** (100%)

#### Graphics Pipeline
```cpp
reactor::GraphicsPipelineBuilder builder(ctx.device(), renderPass);

auto pipeline = builder
    .vertexShader(vertShader)
    .fragmentShader(fragShader)
    .vertexInput(bindings, attributes)
    .viewport(width, height)
    .cullMode(reactor::CullMode::Back)
    .depthTest(true)
    .build();
```

#### Compute Pipeline (NUEVO)
```cpp
#include <reactor/compute_pipeline.hpp>

reactor::ComputePipelineBuilder builder(ctx.device());

VkPipeline pipeline = builder
    .shader(computeShader)
    .descriptorSetLayout(descriptorLayout)
    .pushConstantRange(pushConstant)
    .build();

// Wrapper RAII
reactor::ComputePipeline computePipeline(
    ctx.device(),
    pipeline,
    builder.pipelineLayout()
);
```

### 3. **Descriptor Management** (100% NUEVO)

```cpp
#include <reactor/descriptor_manager.hpp>

reactor::DescriptorManager descriptorMgr(ctx.device());

// Crear layout
std::vector<VkDescriptorSetLayoutBinding> bindings = {
    {0, VK_DESCRIPTOR_TYPE_STORAGE_IMAGE, 1, VK_SHADER_STAGE_COMPUTE_BIT}
};
VkDescriptorSetLayout layout = descriptorMgr.createLayout(bindings);

// Crear pool
std::vector<VkDescriptorPoolSize> poolSizes = {
    {VK_DESCRIPTOR_TYPE_STORAGE_IMAGE, 1}
};
VkDescriptorPool pool = descriptorMgr.createPool(poolSizes, 1);

// Allocar sets
auto sets = descriptorMgr.allocateSets(pool, {layout});

// Update con helpers
descriptorMgr.updateImageDescriptor(
    sets[0], 0,
    VK_DESCRIPTOR_TYPE_STORAGE_IMAGE,
    imageView,
    VK_IMAGE_LAYOUT_GENERAL
);
```

### 4. **Command Buffers** (100%)

```cpp
reactor::CommandPool cmdPool(ctx.device(), queueFamily);
auto cmdPoolPtr = std::make_shared<reactor::CommandPool>(std::move(cmdPool));

reactor::CommandBuffer cmd(cmdPoolPtr);

cmd.begin();
// ... record commands ...
cmd.end();

// Submit
VkSubmitInfo submitInfo{};
// ... configure ...
vkQueueSubmit(ctx.graphicsQueue(), 1, &submitInfo, fence);
```

### 5. **Synchronization** (100%)

```cpp
// Fences
reactor::Fence fence(ctx.device(), false); // unsignaled
fence.wait();
fence.reset();

// Semaphores
reactor::Semaphore imageAvailable(ctx.device());
reactor::Semaphore renderFinished(ctx.device());
```

### 6. **Render Pass & Swapchain** (100%)

```cpp
// Render pass
std::vector<reactor::AttachmentDescription> attachments = {
    {
        .format = swapchain.imageFormat(),
        .loadOp = VK_ATTACHMENT_LOAD_OP_CLEAR,
        .storeOp = VK_ATTACHMENT_STORE_OP_STORE,
        .initialLayout = VK_IMAGE_LAYOUT_UNDEFINED,
        .finalLayout = VK_IMAGE_LAYOUT_PRESENT_SRC_KHR
    }
};
reactor::RenderPass renderPass(ctx.device(), attachments, false);

// Swapchain
reactor::Swapchain swapchain(
    ctx.device(),
    ctx.physical(),
    surface,
    width, height
);
```

### 7. **Window Management** (100%)

```cpp
#include <reactor/window.hpp>

reactor::Window::init();

reactor::WindowConfig config;
config.title = "Mi Aplicación";
config.width = 1920;
config.height = 1080;

reactor::Window window(config);

while (!window.shouldClose()) {
    window.pollEvents();
    // ... render ...
}

reactor::Window::terminate();
```

### 8. **SDF System - Killer Triangle** (100%)

```cpp
#include <reactor/sdf/sdf_primitives.hpp>

using namespace reactor::sdf;

// Crear primitivas
auto sphere = std::make_shared<SphereSDF>(1.0f);
sphere->position = glm::vec3(0, 2, 0);

auto box = std::make_shared<BoxSDF>(glm::vec3(1.0f));
box->position = glm::vec3(2, 0, 0);

// Crear escena
SDFScene scene;
scene.addPrimitive(sphere);
scene.addPrimitive(box);

// Evaluar
float distance = scene.evaluate(glm::vec3(0, 0, 0));
glm::vec3 normal = sphere->getNormal(glm::vec3(1, 0, 0));

// CSG operations
using namespace reactor::sdf::operations;
float combined = opUnion(dist1, dist2);
float carved = opSubtraction(dist1, dist2);
float smooth = opSmoothUnion(dist1, dist2, 0.5f);
```

### 9. **Math Utilities** (100%)

```cpp
#include <reactor/math.hpp>

// Camera
reactor::Camera camera;
camera.position = glm::vec3(0, 2, 5);
camera.target = glm::vec3(0, 0, 0);
camera.aspectRatio = 16.0f / 9.0f;

glm::mat4 view = camera.getViewMatrix();
glm::mat4 proj = camera.getProjectionMatrix();

// Transform
reactor::Transform transform;
transform.position = glm::vec3(1, 2, 3);
transform.rotation = glm::vec3(0, 45, 0);
transform.scale = glm::vec3(2, 2, 2);

glm::mat4 matrix = transform.getMatrix();
```

---

## 🚀 Cómo Usar REACTOR como Base (A)

### Paso 1: Incluir REACTOR en tu Proyecto B

**CMakeLists.txt de tu proyecto B:**
```cmake
cmake_minimum_required(VERSION 3.24)
project(MiMotorGrafico)

# Agregar REACTOR como subdirectorio
add_subdirectory(REACTOR)

# Tu ejecutable/biblioteca
add_executable(mi_motor
    src/main.cpp
    src/mi_renderer.cpp
)

# Linkear con REACTOR (hereda TODO)
target_link_libraries(mi_motor PRIVATE reactor)

# Ahora tienes acceso a TODO de REACTOR
```

### Paso 2: Usar en tu Código

**main.cpp:**
```cpp
// Un solo include para TODO
#include <reactor/reactor.hpp>

int main() {
    // Verificar features disponibles
    std::cout << reactor::getVersion() << std::endl;
    std::cout << "Window support: " << reactor::Features::HAS_WINDOW << std::endl;
    std::cout << "SDF support: " << reactor::Features::HAS_SDF << std::endl;
    
    // Usar cualquier componente de REACTOR
    reactor::Window::init();
    reactor::WindowConfig config;
    config.title = "Mi Motor Gráfico (hereda de REACTOR)";
    reactor::Window window(config);
    
    reactor::VulkanContext ctx(true);
    ctx.init();
    
    // ... tu código específico ...
    
    reactor::Window::terminate();
    return 0;
}
```

### Paso 3: Extender REACTOR en tu Proyecto B

**mi_renderer.hpp:**
```cpp
#pragma once
#include <reactor/reactor.hpp>

namespace mi_motor {

/**
 * @brief Tu renderer que HEREDA funcionalidad de REACTOR
 */
class MiRenderer {
public:
    MiRenderer(reactor::VulkanContext& ctx)
        : ctx_(ctx)
        , descriptorMgr_(ctx.device()) // Usar REACTOR
    {
        // Inicializar usando componentes de REACTOR
        createPipeline();
    }
    
    void render() {
        // Usar command buffers de REACTOR
        // Usar pipelines de REACTOR
        // Usar descriptors de REACTOR
        // etc.
    }
    
private:
    reactor::VulkanContext& ctx_;
    reactor::DescriptorManager descriptorMgr_; // Componente de REACTOR
    
    void createPipeline() {
        // Usar GraphicsPipelineBuilder de REACTOR
        reactor::GraphicsPipelineBuilder builder(ctx_.device(), renderPass_);
        // ...
    }
};

} // namespace mi_motor
```

---

## 📦 Estructura de REACTOR (Biblioteca Base A)

```
REACTOR/
├── reactor/
│   ├── include/reactor/
│   │   ├── reactor.hpp              ⭐ HEADER PRINCIPAL (incluye TODO)
│   │   ├── vulkan_context.hpp       ✅ Core
│   │   ├── memory_allocator.hpp     ✅ Core
│   │   ├── buffer.hpp               ✅ Core
│   │   ├── image.hpp                ✅ Core
│   │   ├── shader.hpp               ✅ Core
│   │   ├── pipeline.hpp             ✅ Graphics
│   │   ├── compute_pipeline.hpp     ✅ Compute (NUEVO)
│   │   ├── descriptor.hpp           ✅ Core
│   │   ├── descriptor_manager.hpp   ✅ Helper (NUEVO)
│   │   ├── command_buffer.hpp       ✅ Core
│   │   ├── sync.hpp                 ✅ Core
│   │   ├── render_pass.hpp          ✅ Core
│   │   ├── swapchain.hpp            ✅ Core
│   │   ├── window.hpp               ✅ Window
│   │   ├── math.hpp                 ✅ Math
│   │   ├── sdf/
│   │   │   ├── sdf_primitives.hpp   ✅ Killer Triangle
│   │   │   ├── primitives.hpp       ✅ SDF
│   │   │   └── raymarcher.hpp       ✅ SDF
│   │   └── isr/
│   │       └── importance.hpp       ✅ ISR
│   └── src/
│       └── ... (implementaciones)
├── shaders/
│   ├── sdf/
│   │   └── raymarch.comp            ✅ Ray marching
│   └── isr/
│       └── ...                      ✅ ISR shaders
├── examples/
│   ├── triangle/                    ✅ Ejemplo básico
│   ├── stack-gpu-cube/              ✅ Ejemplo avanzado
│   └── killer-triangle/             ✅ SDF rendering
└── META/
    ├── REACTOR_BASE_LIBRARY.md      ⭐ Este documento
    ├── KILLER_TRIANGLE.md           ✅ Arquitectura SDF
    └── ...
```

---

## 🎓 Ejemplos de Herencia

### Ejemplo 1: Proyecto B hereda de REACTOR (A)

```cpp
// Proyecto B: MiMotorGrafico
#include <reactor/reactor.hpp>

namespace mi_motor {

class Engine {
public:
    Engine() {
        // Inicializar REACTOR
        ctx_ = std::make_unique<reactor::VulkanContext>(true);
        ctx_->init();
        
        // Usar componentes de REACTOR
        descriptorMgr_ = std::make_unique<reactor::DescriptorManager>(ctx_->device());
    }
    
    void render() {
        // Usar TODO de REACTOR
        // + Tu lógica específica
    }
    
private:
    std::unique_ptr<reactor::VulkanContext> ctx_;
    std::unique_ptr<reactor::DescriptorManager> descriptorMgr_;
    // ... más componentes de REACTOR según necesites
};

} // namespace mi_motor
```

### Ejemplo 2: Proyecto C hereda de B (que hereda de A)

```cpp
// Proyecto C: MiJuego
#include "mi_motor/engine.hpp" // B (que incluye reactor/reactor.hpp = A)

namespace mi_juego {

class Game {
public:
    Game() {
        // Usar motor B (que usa REACTOR A)
        engine_ = std::make_unique<mi_motor::Engine>();
    }
    
    void run() {
        while (running_) {
            // Motor B renderiza (usando REACTOR A)
            engine_->render();
            
            // Tu gameplay específico
            updateGameplay();
        }
    }
    
private:
    std::unique_ptr<mi_motor::Engine> engine_;
    // ... tu lógica de juego
};

} // namespace mi_juego
```

---

## ✨ Ventajas de REACTOR como Base (A)

### 1. **Todo Incluido**
```cpp
// Un solo include
#include <reactor/reactor.hpp>

// Acceso a TODO:
// - Vulkan completo
// - Pipelines (graphics + compute)
// - Descriptors
// - Memory management
// - SDF system
// - ISR system
// - Math utilities
// - Window management
```

### 2. **Fácil de Extender**
```cpp
// Tu proyecto B solo necesita:
target_link_libraries(proyecto_b PRIVATE reactor)

// Y ya tienes TODO de REACTOR disponible
```

### 3. **Modular**
```cpp
// Usar solo lo que necesites
reactor::VulkanContext ctx;
reactor::Buffer buffer;
reactor::ComputePipeline pipeline;

// O usar componentes completos
reactor::sdf::SDFScene scene;
reactor::DescriptorManager descriptorMgr;
```

### 4. **RAII Completo**
```cpp
// Todo se limpia automáticamente
{
    reactor::Buffer buffer(...);
    reactor::ComputePipeline pipeline(...);
    reactor::Fence fence(...);
} // Cleanup automático
```

### 5. **Documentado**
```cpp
// Todos los headers tienen documentación inline
// Todos los ejemplos muestran uso correcto
// META/ contiene arquitectura completa
```

---

## 📊 Checklist de REACTOR como Base (A)

### Core Vulkan
- [x] VulkanContext - Device, queues, physical device
- [x] MemoryAllocator - Gestión automática de memoria
- [x] Buffer - Vertex, index, uniform, storage buffers
- [x] Image - Texturas, render targets
- [x] Shader - SPIR-V loading
- [x] GraphicsPipeline - Builder pattern
- [x] ComputePipeline - Builder pattern (NUEVO)
- [x] Descriptor - Descriptor sets
- [x] DescriptorManager - Helper simplificado (NUEVO)
- [x] CommandBuffer - Recording y submission
- [x] Sync - Fences y semaphores
- [x] RenderPass - Attachments y subpasses
- [x] Swapchain - Present queue

### Window & Input
- [x] Window - GLFW integration
- [x] Input handling - Keyboard, mouse

### Math
- [x] Camera - View y projection matrices
- [x] Transform - Position, rotation, scale
- [x] GLM integration

### Advanced Features
- [x] SDF System - 7 primitivas + CSG
- [x] Ray Marching - Compute shader
- [x] ISR System - Importance calculation
- [x] Killer Triangle - Rendering sin triángulos

### Documentation
- [x] Header principal (reactor.hpp)
- [x] Inline documentation
- [x] Examples
- [x] Architecture docs (META/)
- [x] This guide (REACTOR_BASE_LIBRARY.md)

---

## 🎯 Conclusión

**REACTOR está 100% completo como biblioteca base (A):**

✅ **Core Vulkan**: Completo y funcional  
✅ **Pipelines**: Graphics + Compute  
✅ **Descriptors**: Manager simplificado  
✅ **Memory**: Gestión automática  
✅ **SDF System**: Killer Triangle integrado  
✅ **ISR System**: Intelligent shading rate  
✅ **Math**: Camera + Transform  
✅ **Window**: GLFW integration  
✅ **Documentation**: Completa  

**Proyectos B, C, D, etc. pueden ahora:**
1. Incluir `<reactor/reactor.hpp>`
2. Linkear con `reactor`
3. Heredar TODO sin problemas
4. Extender según necesiten

---

**REACTOR - La Base Sólida (A) para Todos tus Proyectos Vulkan**

🔧 **Completo** | 📦 **Modular** | 🚀 **Fácil de Usar** | 💪 **Production Ready**
