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

## Gestión de Paquetes y Dependencias

### Estrategia Multi-Gestor

REACTOR soportará múltiples gestores de paquetes para máxima flexibilidad:

#### 1. vcpkg (Recomendado para C++)
```bash
# Instalar vcpkg
git clone https://github.com/Microsoft/vcpkg.git
./vcpkg/bootstrap-vcpkg.sh

# Instalar dependencias de REACTOR
vcpkg install vulkan glfw3 glm stb imgui

# Integrar con CMake
cmake -B build -S . -DCMAKE_TOOLCHAIN_FILE=vcpkg/scripts/buildsystems/vcpkg.cmake
```

**vcpkg.json** (manifest mode):
```json
{
  "name": "reactor",
  "version": "0.1.0",
  "dependencies": [
    "vulkan",
    "glfw3",
    "glm",
    "stb",
    {
      "name": "imgui",
      "features": ["glfw-binding", "vulkan-binding"]
    }
  ]
}
```

#### 2. Conan (Alternativa moderna)
```python
# conanfile.py
from conan import ConanFile

class ReactorConan(ConanFile):
    name = "reactor"
    version = "0.1.0"
    settings = "os", "compiler", "build_type", "arch"
    requires = [
        "vulkan-headers/1.3.268",
        "glfw/3.3.8",
        "glm/0.9.9.8",
        "stb/cci.20230920",
        "imgui/1.89.9"
    ]
    generators = "CMakeDeps", "CMakeToolchain"
```

```bash
# Instalar dependencias
conan install . --build=missing
cmake --preset conan-default
cmake --build build
```

#### 3. Sistema de Plugins (Inspirado en npm/Bun)

**reactor.json** (configuración de proyecto):
```json
{
  "name": "mi-app-vulkan",
  "version": "1.0.0",
  "reactor": "^0.1.0",
  "plugins": [
    "@reactor/imgui",
    "@reactor/physics",
    "@reactor/audio",
    "@reactor/networking"
  ],
  "assets": {
    "shaders": "assets/shaders",
    "textures": "assets/textures",
    "models": "assets/models"
  },
  "build": {
    "shaderCompiler": "glslc",
    "optimizations": true,
    "hotReload": true
  }
}
```

**Gestor CLI de REACTOR**:
```bash
# Crear nuevo proyecto
reactor create mi-app --template=game

# Instalar plugin
reactor add @reactor/imgui

# Compilar shaders automáticamente
reactor build-shaders

# Modo desarrollo con hot-reload
reactor dev

# Build optimizado para producción
reactor build --release
```

### Package Registry

**Plugins Oficiales**:
- `@reactor/imgui` - UI inmediata
- `@reactor/physics` - Integración con Bullet/PhysX
- `@reactor/audio` - Sistema de audio 3D
- `@reactor/networking` - Multiplayer
- `@reactor/vr` - Soporte VR/AR
- `@reactor/profiler` - Profiling integrado

**Instalación de Plugins**:
```cpp
// Automáticamente disponible después de 'reactor add'
#include <reactor/plugins/imgui.hpp>

auto ui = reactor::ImGuiPlugin::create()
    .theme(reactor::ImGuiTheme::Dark)
    .docking(true)
    .build();

ui.render([&]() {
    ImGui::Begin("Debug");
    ImGui::Text("FPS: %.1f", fps);
    ImGui::End();
});
```

## React-Style API Mejorada

### Componentes Declarativos (Inspirado en React)

```cpp
// Definir componente reutilizable
class CubeComponent : public reactor::Component {
public:
    // Props (como React props)
    struct Props {
        glm::vec3 position{0, 0, 0};
        glm::vec3 color{1, 1, 1};
        float scale{1.0f};
        std::shared_ptr<reactor::Material> material;
    };
    
    CubeComponent(const Props& props) : props_(props) {}
    
    // Lifecycle (como React)
    void onCreate() override {
        mesh_ = reactor::Mesh::createCube();
        transform_.position = props_.position;
        transform_.scale = glm::vec3(props_.scale);
    }
    
    void onUpdate(float deltaTime) override {
        // Actualizar lógica
        transform_.rotation.y += deltaTime;
    }
    
    void onRender(reactor::RenderContext& ctx) override {
        ctx.draw(mesh_, props_.material, transform_);
    }
    
    // Props reactivos (re-render automático)
    void setColor(const glm::vec3& color) {
        if (props_.color != color) {
            props_.color = color;
            markDirty(); // Re-render
        }
    }

private:
    Props props_;
    reactor::Transform transform_;
    std::shared_ptr<reactor::Mesh> mesh_;
};

// Usar componente (JSX-style en C++)
auto scene = reactor::Scene::create();

// Sintaxis declarativa
scene.add<CubeComponent>({
    .position = {0, 0, 0},
    .color = {1, 0, 0},
    .scale = 2.0f,
    .material = redMaterial
});

// Composición (como React children)
auto group = scene.createGroup("Cubes");
for (int i = 0; i < 10; i++) {
    group.add<CubeComponent>({
        .position = {i * 2.0f, 0, 0},
        .color = {float(i)/10, 0, 1}
    });
}
```

### Hooks System (Inspirado en React Hooks)

```cpp
class MyComponent : public reactor::Component {
public:
    void onUpdate(float dt) override {
        // useState - Estado local
        auto [count, setCount] = useState<int>(0);
        
        // useEffect - Efectos secundarios
        useEffect([&]() {
            std::cout << "Count changed: " << count << std::endl;
            return []() { /* cleanup */ };
        }, {count}); // Dependencies
        
        // useContext - Contexto global
        auto& renderer = useContext<reactor::Renderer>();
        
        // useMemo - Memoización
        auto expensiveValue = useMemo([&]() {
            return calculateExpensive(count);
        }, {count});
        
        // useCallback - Callbacks memoizados
        auto onClick = useCallback([&]() {
            setCount(count + 1);
        }, {count});
        
        // Custom hooks
        auto [pos, vel] = usePhysics(transform_);
    }
};

// Custom hooks
template<typename T>
auto usePhysics(reactor::Transform& transform) {
    auto [velocity, setVelocity] = useState<glm::vec3>({0, 0, 0});
    
    useEffect([&]() {
        // Aplicar física
        transform.position += velocity * dt;
    }, {velocity});
    
    return std::make_tuple(transform.position, velocity);
}
```

### Virtual DOM para Escenas (Reconciliation)

```cpp
// Declarar escena completa (como React render)
class GameScene : public reactor::Scene {
public:
    void render() override {
        // Virtual scene tree
        return scene(
            // Luz ambiental
            ambientLight({.color = {0.2, 0.2, 0.2}}),
            
            // Luz direccional
            directionalLight({
                .direction = {-1, -1, 0},
                .color = {1, 1, 1},
                .intensity = 1.0f
            }),
            
            // Cámara
            camera({
                .position = cameraPos_,
                .target = {0, 0, 0},
                .fov = 60.0f
            }),
            
            // Objetos (solo se re-crean si props cambian)
            map(cubes_, [](const CubeData& data) {
                return cube({
                    .position = data.pos,
                    .color = data.color,
                    .key = data.id // React key
                });
            }),
            
            // Condicional rendering
            showDebug_ ? debugGrid() : nullptr,
            
            // UI overlay
            ui(
                panel({.title = "Debug"},
                    text("FPS: " + std::to_string(fps_)),
                    button("Reset", [&]() { reset(); })
                )
            )
        );
    }

private:
    std::vector<CubeData> cubes_;
    glm::vec3 cameraPos_;
    bool showDebug_ = true;
    float fps_ = 60.0f;
};
```

### State Management (Redux-style)

```cpp
// Estado global de la aplicación
struct AppState {
    struct GameState {
        int score = 0;
        int level = 1;
        bool paused = false;
    } game;
    
    struct RenderState {
        bool vsync = true;
        int msaa = 4;
        bool shadows = true;
    } render;
};

// Actions
struct IncrementScore { int amount; };
struct SetLevel { int level; };
struct TogglePause {};

// Reducer
auto gameReducer = [](AppState state, const auto& action) {
    using T = std::decay_t<decltype(action)>;
    
    if constexpr (std::is_same_v<T, IncrementScore>) {
        state.game.score += action.amount;
    } else if constexpr (std::is_same_v<T, SetLevel>) {
        state.game.level = action.level;
    } else if constexpr (std::is_same_v<T, TogglePause>) {
        state.game.paused = !state.game.paused;
    }
    
    return state;
};

// Store
auto store = reactor::createStore<AppState>(gameReducer);

// Usar en componentes
class ScoreDisplay : public reactor::Component {
    void onRender() override {
        auto state = store.getState();
        ui.text("Score: " + std::to_string(state.game.score));
    }
};

// Dispatch actions
store.dispatch(IncrementScore{10});
store.dispatch(TogglePause{});

// Subscribe a cambios
store.subscribe([](const AppState& state) {
    std::cout << "Score: " << state.game.score << std::endl;
});
```

### Hot Module Replacement (HMR)

```cpp
// Shaders con hot-reload
auto shader = reactor::Shader::create("shader.vert")
    .hotReload(true)  // Recargar automáticamente
    .onReload([](const auto& newShader) {
        std::cout << "Shader reloaded!" << std::endl;
    })
    .build();

// Componentes con hot-reload
REACTOR_HOT_RELOAD(MyComponent) {
    // Este código se puede recargar sin reiniciar
    void onUpdate(float dt) override {
        // Cambios aquí se aplican inmediatamente
    }
};

// Assets con hot-reload
auto texture = reactor::Texture::load("texture.png")
    .hotReload(true)
    .build();
```

### Desarrollo Declarativo Completo

```cpp
int main() {
    // Configuración declarativa completa
    auto app = reactor::App::create()
        // Window
        .window({
            .title = "Mi Juego",
            .width = 1920,
            .height = 1080,
            .fullscreen = false,
            .vsync = true
        })
        
        // Renderer
        .renderer({
            .msaa = 4,
            .shadows = true,
            .postProcessing = {
                reactor::Bloom{},
                reactor::ToneMapping{},
                reactor::FXAA{}
            }
        })
        
        // Plugins
        .plugin<reactor::ImGuiPlugin>()
        .plugin<reactor::PhysicsPlugin>()
        
        // Scene inicial
        .scene<MainMenuScene>()
        
        // Hot reload en desarrollo
        .hotReload(true)
        
        // Build
        .build();
    
    // Run con render loop automático
    return app.run();
}
```

## Sistema de Assets (Asset Pipeline)

### Carga Declarativa de Assets

```cpp
// Definir assets en JSON/YAML
// assets.yaml
assets:
  textures:
    - name: "player_diffuse"
      path: "textures/player.png"
      format: RGBA8
      mipmaps: true
      
  models:
    - name: "player_model"
      path: "models/player.gltf"
      scale: 1.0
      
  shaders:
    - name: "pbr_shader"
      vertex: "shaders/pbr.vert"
      fragment: "shaders/pbr.frag"
      defines:
        USE_NORMAL_MAP: 1

// Cargar automáticamente
auto assets = reactor::AssetManager::loadManifest("assets.yaml");

// Acceso type-safe
auto texture = assets.get<reactor::Texture>("player_diffuse");
auto model = assets.get<reactor::Model>("player_model");
auto shader = assets.get<reactor::Shader>("pbr_shader");

// Async loading con progreso
assets.loadAsync("level1", [](float progress) {
    std::cout << "Loading: " << progress * 100 << "%" << std::endl;
}).then([](auto& loadedAssets) {
    std::cout << "Level loaded!" << std::endl;
});
```

### Asset Bundles y Streaming

```cpp
// Crear bundle
reactor::AssetBundle::create("level1")
    .add("textures/*.png")
    .add("models/*.gltf")
    .compress(true)
    .build("level1.bundle");

// Cargar bundle
auto bundle = reactor::AssetBundle::load("level1.bundle");
bundle.loadAsync().then([&](auto& assets) {
    // Assets disponibles
});

// Streaming (LOD automático)
auto terrain = reactor::StreamingTerrain::create()
    .tileSize(256)
    .viewDistance(1000.0f)
    .lod(4)
    .build();
```

## Developer Experience (DX)

### CLI Tool Completo

```bash
# Crear proyecto desde template
reactor new mi-juego --template=fps
reactor new mi-app --template=visualization

# Gestión de dependencias
reactor add glm
reactor add imgui --version=1.89
reactor remove stb

# Desarrollo
reactor dev                    # Hot-reload activado
reactor dev --port=3000       # Custom port para web preview
reactor dev --profile         # Con profiler

# Build
reactor build                 # Debug build
reactor build --release       # Release optimizado
reactor build --platform=web  # WebGPU target

# Shaders
reactor compile-shaders       # Compilar todos los shaders
reactor watch-shaders         # Watch mode

# Assets
reactor pack-assets           # Crear bundles
reactor optimize-textures     # Optimizar texturas

# Deployment
reactor deploy --platform=windows
reactor deploy --platform=linux
reactor deploy --platform=web
```

### Debugging Visual

```cpp
// Debug rendering integrado
reactor::Debug::drawLine(start, end, color);
reactor::Debug::drawSphere(center, radius, color);
reactor::Debug::drawBox(min, max, color);
reactor::Debug::drawText(position, "Debug Info");

// Profiling integrado
REACTOR_PROFILE_SCOPE("Update Loop");
REACTOR_PROFILE_GPU("Shadow Pass");

// Inspector en tiempo real
reactor::Inspector::watch("Player Position", playerPos);
reactor::Inspector::watch("FPS", fps);
reactor::Inspector::graph("Frame Time", frameTime);
```

## Conclusión Mejorada

REACTOR busca ser el framework definitivo para Vulkan, combinando:
- **Facilidad de uso** estilo React/modern frameworks
- **Control total** de Vulkan puro cuando se necesita
- **Performance** sin overhead (zero-cost abstractions)
- **Developer Experience** de primera clase
- **Gestión de paquetes** moderna y flexible
- **Hot-reload** para desarrollo rápido
- **Componentes reutilizables** y composables
- **Flexibilidad** para cualquier tipo de aplicación

### Objetivos Cuantificables

- ⏱️ Reducir tiempo de setup: **de 2 días a 5 minutos**
- 📝 Reducir código boilerplate: **70% menos líneas**
- 🔄 Hot-reload: **cambios visibles en <1 segundo**
- 🎨 Componentes reutilizables: **biblioteca de 100+ componentes**
- 📦 Gestión de assets: **automática con bundles optimizados**
- 🐛 Debugging: **inspector visual integrado**
- 🚀 Performance: **0% overhead vs Vulkan puro**

El objetivo es que crear una aplicación Vulkan sea tan fácil como:
```bash
reactor new mi-juego --template=fps
cd mi-juego
reactor dev
```

Y tener un juego funcional con hot-reload, UI, física y audio en **menos de 5 minutos**.
