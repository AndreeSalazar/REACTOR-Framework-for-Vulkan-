# REACTOR Framework - Gestión de Paquetes

## Filosofía de Gestión de Dependencias

REACTOR adopta un enfoque **multi-gestor** para máxima flexibilidad, similar a cómo los frameworks modernos web soportan múltiples package managers (npm, yarn, pnpm, bun).

## Opciones de Gestores de Paquetes

### 1. vcpkg (Recomendado para Windows)

**Ventajas**:
- ✅ Integración perfecta con Visual Studio
- ✅ Soporte oficial de Microsoft
- ✅ Binarios pre-compilados
- ✅ Manifest mode (vcpkg.json)

**Setup**:
```bash
# Instalar vcpkg
git clone https://github.com/Microsoft/vcpkg.git
cd vcpkg
bootstrap-vcpkg.bat  # Windows
./bootstrap-vcpkg.sh # Linux/macOS

# Instalar REACTOR con todas las features
vcpkg install reactor[window,math,imgui,image-loading]

# O usar manifest mode (recomendado)
# vcpkg.json ya está incluido en el proyecto
cmake -B build -S . -DCMAKE_TOOLCHAIN_FILE=vcpkg/scripts/buildsystems/vcpkg.cmake
cmake --build build
```

**vcpkg.json Features**:
```json
{
  "dependencies": [
    "vulkan",                    // Core (siempre)
    "reactor[window]",           // + GLFW
    "reactor[math]",             // + GLM
    "reactor[imgui]",            // + ImGui
    "reactor[image-loading]",    // + STB
    "reactor[model-loading]"     // + Assimp
  ]
}
```

### 2. Conan (Recomendado para Multi-plataforma)

**Ventajas**:
- ✅ Mejor para proyectos cross-platform
- ✅ Gestión de versiones más granular
- ✅ Recetas personalizables
- ✅ Repositorios privados fáciles

**Setup**:
```bash
# Instalar Conan
pip install conan

# Configurar perfil
conan profile detect --force

# Instalar dependencias
conan install . --build=missing -s build_type=Release

# Compilar con Conan
cmake --preset conan-release
cmake --build --preset conan-release
```

**Opciones de Conan**:
```bash
# Con window support
conan install . -o reactor/*:with_window=True

# Con ImGui
conan install . -o reactor/*:with_imgui=True

# Con física
conan install . -o reactor/*:with_physics=True

# Todo junto
conan install . -o reactor/*:with_window=True \
                -o reactor/*:with_imgui=True \
                -o reactor/*:with_physics=True
```

### 3. Sistema Nativo de CMake (FetchContent)

**Ventajas**:
- ✅ Sin dependencias externas
- ✅ Compilación desde fuente
- ✅ Control total

**Setup**:
```cmake
# En tu CMakeLists.txt
include(FetchContent)

FetchContent_Declare(
  reactor
  GIT_REPOSITORY https://github.com/tu-usuario/reactor.git
  GIT_TAG        v0.1.0
)

FetchContent_MakeAvailable(reactor)

target_link_libraries(mi_app PRIVATE reactor::reactor)
```

### 4. Sistema de Plugins REACTOR (Futuro)

**Visión**: CLI tool estilo npm/bun para C++

```bash
# Crear proyecto
reactor new mi-juego --template=fps

# Instalar plugins
reactor add @reactor/imgui
reactor add @reactor/physics@latest
reactor add @reactor/audio --save-dev

# Actualizar
reactor update

# Listar instalados
reactor list

# Remover
reactor remove @reactor/audio
```

**reactor.json** (configuración):
```json
{
  "name": "mi-juego",
  "version": "1.0.0",
  "reactor": "^0.1.0",
  "plugins": {
    "@reactor/imgui": "^1.0.0",
    "@reactor/physics": "^2.1.0",
    "@reactor/audio": "^1.5.0"
  },
  "devPlugins": {
    "@reactor/profiler": "^0.5.0"
  },
  "config": {
    "hotReload": true,
    "validation": true
  }
}
```

## Comparación de Gestores

| Feature | vcpkg | Conan | CMake FetchContent | REACTOR CLI |
|---------|-------|-------|-------------------|-------------|
| **Facilidad de uso** | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Windows** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Linux** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **macOS** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Binarios pre-compilados** | ✅ | ✅ | ❌ | ✅ |
| **Versiones específicas** | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Repositorios privados** | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Velocidad** | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ |

## Recomendaciones por Caso de Uso

### Desarrollo en Windows con Visual Studio
```bash
# Usar vcpkg
vcpkg install reactor[window,imgui]
cmake -B build -DCMAKE_TOOLCHAIN_FILE=vcpkg/scripts/buildsystems/vcpkg.cmake
```

### Proyecto Cross-Platform
```bash
# Usar Conan
conan install . --build=missing
cmake --preset conan-release
```

### Prototipado Rápido
```bash
# Usar REACTOR CLI (cuando esté disponible)
reactor new prototipo --template=minimal
reactor dev
```

### CI/CD Pipeline
```bash
# Usar Conan para reproducibilidad
conan install . --build=missing -s build_type=Release
conan build .
conan export-pkg .
```

## Plugins Oficiales de REACTOR

### Core Plugins

#### @reactor/window
```bash
reactor add @reactor/window
```
```cpp
#include <reactor/plugins/window.hpp>

auto window = reactor::Window::create()
    .title("Mi App")
    .size(1920, 1080)
    .vsync(true)
    .build();
```

#### @reactor/imgui
```bash
reactor add @reactor/imgui
```
```cpp
#include <reactor/plugins/imgui.hpp>

auto ui = reactor::ImGuiPlugin::create()
    .theme(reactor::ImGuiTheme::Dark)
    .docking(true)
    .build();

ui.render([&]() {
    ImGui::ShowDemoWindow();
});
```

#### @reactor/physics
```bash
reactor add @reactor/physics
```
```cpp
#include <reactor/plugins/physics.hpp>

auto physics = reactor::PhysicsWorld::create()
    .gravity({0, -9.81f, 0})
    .build();

auto rigidBody = physics.createRigidBody({
    .mass = 1.0f,
    .shape = reactor::BoxShape({1, 1, 1})
});
```

#### @reactor/audio
```bash
reactor add @reactor/audio
```
```cpp
#include <reactor/plugins/audio.hpp>

auto audio = reactor::AudioEngine::create()
    .spatialAudio(true)
    .build();

auto sound = audio.loadSound("explosion.wav");
sound.play({.position = {0, 0, 0}, .volume = 0.8f});
```

### Utility Plugins

#### @reactor/profiler
```bash
reactor add @reactor/profiler --dev
```
```cpp
#include <reactor/plugins/profiler.hpp>

REACTOR_PROFILE_SCOPE("Game Loop");
REACTOR_PROFILE_GPU("Shadow Pass");

// Ver resultados en UI
reactor::Profiler::showUI();
```

#### @reactor/networking
```bash
reactor add @reactor/networking
```
```cpp
#include <reactor/plugins/networking.hpp>

auto server = reactor::Server::create()
    .port(7777)
    .maxClients(32)
    .build();

server.onConnect([](auto& client) {
    std::cout << "Client connected: " << client.id() << std::endl;
});
```

## Gestión de Assets

### Asset Pipeline Automático

```yaml
# assets.yaml
pipeline:
  textures:
    input: "assets/textures/**/*.png"
    output: "build/assets/textures"
    compress: true
    mipmaps: true
    
  models:
    input: "assets/models/**/*.gltf"
    output: "build/assets/models"
    optimize: true
    
  shaders:
    input: "assets/shaders/**/*.{vert,frag}"
    output: "build/assets/shaders"
    compiler: glslc
    optimize: true
```

```bash
# Procesar assets
reactor build-assets

# Watch mode
reactor watch-assets
```

## Versionado y Compatibilidad

### Semantic Versioning

REACTOR sigue semver estricto:
- `0.1.0` - Versión inicial
- `0.2.0` - Nuevas features (backward compatible)
- `1.0.0` - API estable
- `1.1.0` - Nuevas features
- `2.0.0` - Breaking changes

### Especificar Versiones

```json
{
  "plugins": {
    "@reactor/imgui": "^1.0.0",    // >= 1.0.0 < 2.0.0
    "@reactor/physics": "~2.1.0",   // >= 2.1.0 < 2.2.0
    "@reactor/audio": "1.5.0",      // Exactamente 1.5.0
    "@reactor/vr": "latest"         // Última versión
  }
}
```

## Lockfile

```json
// reactor.lock
{
  "version": "1.0.0",
  "plugins": {
    "@reactor/imgui": {
      "version": "1.0.5",
      "resolved": "https://registry.reactor.dev/@reactor/imgui/-/imgui-1.0.5.tgz",
      "integrity": "sha512-...",
      "dependencies": {
        "glfw": "3.3.8"
      }
    }
  }
}
```

## Registry Privado

### Configurar Registry Privado

```bash
# Configurar registry
reactor config set registry https://registry.mi-empresa.com

# Autenticación
reactor login

# Publicar plugin privado
reactor publish
```

### Crear Plugin Personalizado

```cpp
// mi_plugin/include/mi_plugin.hpp
#pragma once
#include <reactor/plugin.hpp>

namespace mi_empresa {

class MiPlugin : public reactor::Plugin {
public:
    void onInit() override {
        // Inicialización
    }
    
    void onUpdate(float dt) override {
        // Actualización
    }
};

} // namespace mi_empresa

// Registrar plugin
REACTOR_REGISTER_PLUGIN(mi_empresa::MiPlugin, "mi-plugin", "1.0.0")
```

```json
// plugin.json
{
  "name": "@mi-empresa/mi-plugin",
  "version": "1.0.0",
  "description": "Plugin personalizado",
  "main": "lib/mi_plugin.so",
  "dependencies": {
    "reactor": "^0.1.0"
  }
}
```

## Migración entre Gestores

### De vcpkg a Conan

```bash
# Generar conanfile.py desde vcpkg.json
reactor migrate vcpkg-to-conan

# Instalar con Conan
conan install . --build=missing
```

### De CMake FetchContent a vcpkg

```bash
# Generar vcpkg.json desde CMakeLists.txt
reactor migrate cmake-to-vcpkg

# Instalar con vcpkg
vcpkg install
```

## Conclusión

REACTOR ofrece **máxima flexibilidad** en gestión de dependencias:

1. **vcpkg** - Mejor para Windows/Visual Studio
2. **Conan** - Mejor para cross-platform
3. **CMake FetchContent** - Sin dependencias externas
4. **REACTOR CLI** - Experiencia moderna (futuro)

Elige el que mejor se adapte a tu workflow y plataforma. Todos son totalmente soportados.
