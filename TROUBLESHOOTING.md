# REACTOR Framework - Troubleshooting Guide

## Problemas Comunes de Compilación

### Error: "The C++ compiler is not able to compile a simple test program"

**Síntomas**:
```
CMake Error: The C++ compiler is not able to compile a simple test program
ninja: build stopped: subcommand failed
```

**Causa**: Visual Studio no está correctamente configurado en el PATH o falta el entorno de desarrollo.

**Soluciones**:

#### Solución 1: Usar el script de configuración (Recomendado)
```bash
# En PowerShell o CMD
configure.bat
build.bat
```

#### Solución 2: Developer Command Prompt
```bash
# Abrir "Developer Command Prompt for VS 2022" desde el menú inicio
# Luego ejecutar:
cd "C:\Users\andre\OneDrive\Documentos\REACTOR (Framework for Vulkan)"
cmake -S . -B build -G "Visual Studio 17 2022" -A x64
cmake --build build --config Release
```

#### Solución 3: Configurar manualmente el entorno
```powershell
# En PowerShell
& "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat" x64

# O para BuildTools:
& "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat" x64

# Luego configurar y compilar
cmake -S . -B build -G "Ninja" -DCMAKE_BUILD_TYPE=Release
cmake --build build
```

#### Solución 4: Usar Visual Studio Generator (sin Ninja)
```bash
# No requiere Ninja
cmake -S . -B build -G "Visual Studio 17 2022" -A x64
cmake --build build --config Release
```

### Error: "VULKAN_SDK not found"

**Síntomas**:
```
CMake Error: Could not find Vulkan SDK
```

**Soluciones**:

#### Verificar instalación
```powershell
# En PowerShell
$env:VULKAN_SDK
# Debe mostrar algo como: C:\VulkanSDK\1.3.xxx.x
```

#### Instalar Vulkan SDK
1. Descargar desde: https://vulkan.lunarg.com/
2. Ejecutar instalador
3. Reiniciar terminal/IDE
4. Verificar con `$env:VULKAN_SDK`

#### Configurar manualmente
```powershell
# Temporal (esta sesión)
$env:VULKAN_SDK = "C:\VulkanSDK\1.3.xxx.x"

# Permanente (System Properties > Environment Variables)
# Agregar VULKAN_SDK = C:\VulkanSDK\1.3.xxx.x
```

### Error: "CMake version too old"

**Síntomas**:
```
CMake Error: CMake 3.24 or higher is required
```

**Solución**:
```bash
# Actualizar CMake
choco upgrade cmake

# O descargar desde: https://cmake.org/download/
```

### Error: "Ninja not found"

**Síntomas**:
```
CMake Error: Could not find Ninja
```

**Soluciones**:

#### Opción 1: Instalar Ninja
```bash
# Con chocolatey
choco install ninja

# O descargar desde: https://github.com/ninja-build/ninja/releases
```

#### Opción 2: Usar Visual Studio Generator
```bash
# No requiere Ninja
cmake -S . -B build -G "Visual Studio 17 2022" -A x64
```

### Error: "Cannot open include file: 'vulkan/vulkan.h'"

**Síntomas**:
```
fatal error C1083: Cannot open include file: 'vulkan/vulkan.h'
```

**Causa**: CMake no encuentra los headers de Vulkan.

**Solución**:
```bash
# Limpiar y reconfigurar
rmdir /s /q build
cmake -S . -B build -G "Visual Studio 17 2022" -A x64 -DCMAKE_PREFIX_PATH="%VULKAN_SDK%"
```

### Error: Linking con Vulkan

**Síntomas**:
```
LINK : fatal error LNK1181: cannot open input file 'vulkan-1.lib'
```

**Solución**:
```bash
# Verificar que VULKAN_SDK esté configurado
echo %VULKAN_SDK%

# Reconfigurar con path explícito
cmake -S . -B build -G "Visual Studio 17 2022" -A x64 ^
  -DCMAKE_PREFIX_PATH="%VULKAN_SDK%" ^
  -DVulkan_LIBRARY="%VULKAN_SDK%\Lib\vulkan-1.lib" ^
  -DVulkan_INCLUDE_DIR="%VULKAN_SDK%\Include"
```

## Problemas de Runtime

### Error: "Validation layers not found"

**Síntomas**:
```
ERROR: Validation layer VK_LAYER_KHRONOS_validation not available
```

**Solución**:
```bash
# Verificar que las validation layers estén instaladas
dir "%VULKAN_SDK%\Bin"

# Si no están, reinstalar Vulkan SDK con validation layers
```

### Error: "Failed to create Vulkan instance"

**Síntomas**:
```
vkCreateInstance failed
```

**Causas posibles**:
1. Drivers de GPU desactualizados
2. GPU no soporta Vulkan 1.3
3. Validation layers no disponibles

**Soluciones**:

#### Actualizar drivers
- NVIDIA: https://www.nvidia.com/drivers
- AMD: https://www.amd.com/support
- Intel: https://www.intel.com/content/www/us/en/download-center/home.html

#### Verificar soporte de Vulkan
```bash
# Ejecutar desde Vulkan SDK
"%VULKAN_SDK%\Bin\vulkaninfo.exe"
```

#### Deshabilitar validation layers
```cpp
// En tu código
reactor::VulkanContext ctx(false);  // false = sin validation
```

### Error: "No suitable GPU found"

**Síntomas**:
```
ERROR: no suitable device
```

**Solución**:
```bash
# Verificar GPUs disponibles
"%VULKAN_SDK%\Bin\vulkaninfo.exe" --summary

# Si no aparece tu GPU, actualizar drivers
```

## Problemas de Desarrollo

### Hot-reload no funciona

**Causa**: File watcher no configurado correctamente.

**Solución**:
```cpp
// Asegurarse de que hot-reload esté habilitado
auto shader = reactor::Shader::create("shader.vert")
    .hotReload(true)
    .build();

// En modo desarrollo
auto app = reactor::App::create()
    .hotReload(true)
    .build();
```

### Performance pobre en Debug

**Causa**: Validation layers activas en Debug build.

**Solución**:
```bash
# Compilar en Release para mejor performance
cmake --build build --config Release

# O deshabilitar validation
cmake -S . -B build -DREACTOR_ENABLE_VALIDATION=OFF
```

### Memory leaks detectados

**Causa**: Recursos no destruidos correctamente.

**Solución**:
```cpp
// REACTOR usa RAII, asegúrate de que los recursos
// estén en scope correcto

{
    auto buffer = reactor::Buffer::create(allocator)...build();
    // Usar buffer
} // Destruido automáticamente aquí

// NO hacer:
auto* buffer = new reactor::Buffer(...);  // ❌
// Usar RAII siempre
```

## Verificación del Sistema

### Script de Diagnóstico

Crea un archivo `diagnose.bat`:

```batch
@echo off
echo REACTOR Framework - System Diagnostics
echo ========================================
echo.

echo Checking Vulkan SDK...
if defined VULKAN_SDK (
    echo [OK] VULKAN_SDK = %VULKAN_SDK%
) else (
    echo [ERROR] VULKAN_SDK not set
)
echo.

echo Checking CMake...
where cmake >nul 2>nul
if %ERRORLEVEL% equ 0 (
    cmake --version
) else (
    echo [ERROR] CMake not found
)
echo.

echo Checking Ninja...
where ninja >nul 2>nul
if %ERRORLEVEL% equ 0 (
    ninja --version
) else (
    echo [INFO] Ninja not found (optional)
)
echo.

echo Checking Visual Studio...
where cl >nul 2>nul
if %ERRORLEVEL% equ 0 (
    cl 2>&1 | findstr "Version"
) else (
    echo [WARNING] MSVC compiler not in PATH
    echo Try running from Developer Command Prompt
)
echo.

echo Checking Vulkan...
if exist "%VULKAN_SDK%\Bin\vulkaninfo.exe" (
    "%VULKAN_SDK%\Bin\vulkaninfo.exe" --summary
) else (
    echo [ERROR] vulkaninfo.exe not found
)
echo.

echo ========================================
echo Diagnostics complete
echo ========================================
```

Ejecutar:
```bash
diagnose.bat
```

## Obtener Ayuda

Si ninguna solución funciona:

1. **Ejecutar diagnóstico**:
   ```bash
   diagnose.bat > diagnostics.txt
   ```

2. **Recopilar información**:
   - Sistema operativo y versión
   - Versión de Visual Studio
   - Versión de Vulkan SDK
   - Modelo de GPU
   - Output completo del error

3. **Reportar issue**:
   - GitHub Issues con toda la información
   - Incluir `diagnostics.txt`
   - Incluir output completo de CMake

## Solución Rápida (Clean Slate)

Si todo falla, empezar desde cero:

```bash
# 1. Limpiar completamente
rmdir /s /q build
del CMakeCache.txt

# 2. Verificar requisitos
diagnose.bat

# 3. Usar script de configuración
configure.bat

# 4. Compilar
build.bat

# 5. Ejecutar ejemplo
build\examples\triangle\Release\reactor-triangle.exe
```

## Contacto

- **Issues**: GitHub Issues
- **Documentación**: Ver archivos `.md` en el repositorio
- **Ejemplos**: Directorio `examples/`
