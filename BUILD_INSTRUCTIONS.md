# REACTOR Framework - Instrucciones de Compilación

## Requisitos del Sistema

### Windows
- **OS**: Windows 10/11 (64-bit)
- **Compilador**: Visual Studio 2022 o MSVC Build Tools
- **CMake**: 3.24 o superior
- **Vulkan SDK**: 1.3.x o superior
- **Ninja** (opcional): Para builds más rápidos

### Linux
- **OS**: Ubuntu 20.04+, Fedora 35+, o similar
- **Compilador**: GCC 11+ o Clang 14+
- **CMake**: 3.24 o superior
- **Vulkan SDK**: 1.3.x o superior
- **Ninja** (opcional): `sudo apt install ninja-build`

### macOS
- **OS**: macOS 12+ (Monterey o superior)
- **Compilador**: Clang 14+ (Xcode Command Line Tools)
- **CMake**: 3.24 o superior
- **Vulkan SDK**: 1.3.x o superior (MoltenVK)
- **Ninja** (opcional): `brew install ninja`

## Instalación de Dependencias

### Windows

#### 1. Instalar Vulkan SDK
```powershell
# Descargar desde: https://vulkan.lunarg.com/
# Ejecutar instalador y configurar VULKAN_SDK

# Verificar instalación
$env:VULKAN_SDK
# Debe mostrar: C:\VulkanSDK\1.3.xxx.x
```

#### 2. Instalar CMake
```powershell
# Descargar desde: https://cmake.org/download/
# O usar chocolatey:
choco install cmake

# Verificar
cmake --version
```

#### 3. Instalar Ninja (opcional)
```powershell
# Descargar desde: https://github.com/ninja-build/ninja/releases
# O usar chocolatey:
choco install ninja

# Verificar
ninja --version
```

#### 4. Instalar Visual Studio 2022
```powershell
# Descargar Community Edition desde:
# https://visualstudio.microsoft.com/downloads/

# Componentes necesarios:
# - Desktop development with C++
# - C++ CMake tools for Windows
```

### Linux (Ubuntu/Debian)

```bash
# Actualizar sistema
sudo apt update && sudo apt upgrade

# Instalar compilador y herramientas
sudo apt install build-essential cmake ninja-build git

# Instalar Vulkan SDK
wget -qO - https://packages.lunarg.com/lunarg-signing-key-pub.asc | sudo apt-key add -
sudo wget -qO /etc/apt/sources.list.d/lunarg-vulkan-1.3.xxx-focal.list \
    https://packages.lunarg.com/vulkan/1.3.xxx/lunarg-vulkan-1.3.xxx-focal.list
sudo apt update
sudo apt install vulkan-sdk

# Verificar
vulkaninfo --summary
```

### Linux (Fedora)

```bash
# Instalar herramientas
sudo dnf install gcc-c++ cmake ninja-build git

# Instalar Vulkan SDK
sudo dnf install vulkan-headers vulkan-loader vulkan-tools vulkan-validation-layers

# Verificar
vulkaninfo --summary
```

### macOS

```bash
# Instalar Homebrew si no está instalado
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Instalar herramientas
brew install cmake ninja git

# Instalar Vulkan SDK (MoltenVK)
# Descargar desde: https://vulkan.lunarg.com/sdk/home
# O usar Homebrew:
brew install --cask vulkan-sdk

# Configurar variable de entorno
export VULKAN_SDK=/Users/$USER/VulkanSDK/1.3.xxx.x/macOS
echo 'export VULKAN_SDK=/Users/$USER/VulkanSDK/1.3.xxx.x/macOS' >> ~/.zshrc

# Verificar
vulkaninfo --summary
```

## Compilación del Framework

### Opción 1: Usando Ninja (Recomendado)

#### Windows (PowerShell)
```powershell
# Clonar repositorio
git clone https://github.com/tu-usuario/reactor.git
cd reactor

# Configurar con Ninja
cmake -S . -B build -G "Ninja" -DCMAKE_BUILD_TYPE=Release

# Compilar
cmake --build build --config Release

# Ejecutar tests
build\examples\sandbox\reactor-sandbox.exe
build\examples\triangle\reactor-triangle.exe
```

#### Linux/macOS (Bash)
```bash
# Clonar repositorio
git clone https://github.com/tu-usuario/reactor.git
cd reactor

# Configurar con Ninja
cmake -S . -B build -G "Ninja" -DCMAKE_BUILD_TYPE=Release

# Compilar
cmake --build build

# Ejecutar tests
./build/examples/sandbox/reactor-sandbox
./build/examples/triangle/reactor-triangle
```

### Opción 2: Usando Visual Studio (Windows)

```powershell
# Configurar para Visual Studio
cmake -S . -B build -G "Visual Studio 17 2022" -A x64

# Compilar
cmake --build build --config Release

# O abrir en Visual Studio
start build\reactor.sln
```

### Opción 3: Usando Make (Linux/macOS)

```bash
# Configurar con Make
cmake -S . -B build -G "Unix Makefiles" -DCMAKE_BUILD_TYPE=Release

# Compilar
cmake --build build -- -j$(nproc)

# Ejecutar
./build/examples/triangle/reactor-triangle
```

## Opciones de Compilación

### Validation Layers

```bash
# Habilitar validation layers (default: ON)
cmake -S . -B build -DREACTOR_ENABLE_VALIDATION=ON

# Deshabilitar para mejor performance
cmake -S . -B build -DREACTOR_ENABLE_VALIDATION=OFF
```

### Build Types

```bash
# Debug (con símbolos de depuración)
cmake -S . -B build -DCMAKE_BUILD_TYPE=Debug

# Release (optimizado)
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release

# RelWithDebInfo (optimizado + símbolos)
cmake -S . -B build -DCMAKE_BUILD_TYPE=RelWithDebInfo

# MinSizeRel (optimizado para tamaño)
cmake -S . -B build -DCMAKE_BUILD_TYPE=MinSizeRel
```

### Compilación Paralela

```bash
# Windows
cmake --build build --config Release -- /m

# Linux/macOS (usar todos los cores)
cmake --build build -- -j$(nproc)

# Especificar número de cores
cmake --build build -- -j8
```

## Integración en Proyectos

### Como Subdirectorio

```cmake
# En tu CMakeLists.txt
add_subdirectory(external/reactor)

add_executable(mi_app main.cpp)
target_link_libraries(mi_app PRIVATE reactor)
```

### Como Librería Instalada

```bash
# Instalar REACTOR
cmake -S . -B build -DCMAKE_INSTALL_PREFIX=/usr/local
cmake --build build
sudo cmake --install build

# En tu proyecto
find_package(reactor REQUIRED)
target_link_libraries(mi_app PRIVATE reactor::reactor)
```

## Troubleshooting

### Error: "Vulkan SDK not found"

**Windows**:
```powershell
# Verificar variable de entorno
echo $env:VULKAN_SDK

# Si no está configurada:
$env:VULKAN_SDK = "C:\VulkanSDK\1.3.xxx.x"
```

**Linux/macOS**:
```bash
# Verificar
echo $VULKAN_SDK

# Configurar
export VULKAN_SDK=/path/to/vulkan/sdk
```

### Error: "CMake version too old"

```bash
# Actualizar CMake
# Windows (chocolatey):
choco upgrade cmake

# Linux:
sudo snap install cmake --classic

# macOS:
brew upgrade cmake
```

### Error: "Compiler does not support C++20"

**Windows**: Actualizar a Visual Studio 2022

**Linux**:
```bash
# Instalar GCC 11+
sudo apt install gcc-11 g++-11

# Configurar como default
sudo update-alternatives --install /usr/bin/gcc gcc /usr/bin/gcc-11 100
sudo update-alternatives --install /usr/bin/g++ g++ /usr/bin/g++-11 100
```

**macOS**:
```bash
# Actualizar Xcode Command Line Tools
xcode-select --install
```

### Error: "Ninja not found"

```bash
# Windows:
choco install ninja

# Linux:
sudo apt install ninja-build

# macOS:
brew install ninja
```

### Warnings sobre validation layers

```bash
# Instalar validation layers
# Windows: Ya incluidas en Vulkan SDK

# Linux:
sudo apt install vulkan-validationlayers

# macOS: Ya incluidas en Vulkan SDK
```

## Verificación de la Instalación

### Test Rápido

```bash
# Compilar y ejecutar ejemplo básico
cmake -S . -B build -G "Ninja" -DCMAKE_BUILD_TYPE=Release
cmake --build build
./build/examples/triangle/reactor-triangle

# Salida esperada:
# REACTOR Triangle Example - Framework initialized successfully!
# Created vertex buffer with 3 vertices
# REACTOR Framework demonstration complete!
```

### Verificar Componentes

```bash
# Verificar que todos los archivos se compilaron
ls build/reactor/
# Debe mostrar: libreactor.a (Linux/macOS) o reactor.lib (Windows)

ls build/examples/triangle/
# Debe mostrar: reactor-triangle (ejecutable)
```

## Performance Tips

### Compilación Optimizada

```bash
# Máxima optimización
cmake -S . -B build \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_CXX_FLAGS="-O3 -march=native" \
    -DREACTOR_ENABLE_VALIDATION=OFF
```

### Link Time Optimization (LTO)

```bash
cmake -S . -B build \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_INTERPROCEDURAL_OPTIMIZATION=ON
```

## Siguiente Paso

Una vez compilado exitosamente, consulta:
- **[USAGE_GUIDE.md](USAGE_GUIDE.md)** - Para aprender a usar el framework
- **[examples/](examples/)** - Para ver ejemplos completos
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - Para entender la arquitectura

## Soporte

Si encuentras problemas:
1. Verifica que cumples todos los requisitos
2. Revisa la sección de Troubleshooting
3. Abre un issue en GitHub con:
   - Sistema operativo y versión
   - Versión de CMake, compilador y Vulkan SDK
   - Comando completo que causó el error
   - Output completo del error
