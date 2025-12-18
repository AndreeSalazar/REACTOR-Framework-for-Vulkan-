@echo off
REM REACTOR Starter Template - Setup Script

echo ==========================================
echo   REACTOR Starter Template - Setup
echo ==========================================
echo.

REM Verificar que estamos en el directorio correcto
if not exist "src\main.cpp" (
    echo ERROR: src\main.cpp not found
    echo Please run this script from the template directory
    exit /b 1
)

echo [1/4] Verificando requisitos...
echo.

REM Verificar Vulkan SDK
if not defined VULKAN_SDK (
    echo [INFO] VULKAN_SDK no configurado, buscando instalacion...
    
    REM Buscar versiones especificas
    if exist "C:\VulkanSDK\1.4.328.1" (
        set "VULKAN_SDK=C:\VulkanSDK\1.4.328.1"
        echo [OK] Vulkan SDK 1.4.328.1 encontrado!
    ) else if exist "C:\VulkanSDK\1.3.290.0" (
        set "VULKAN_SDK=C:\VulkanSDK\1.3.290.0"
        echo [OK] Vulkan SDK 1.3.290.0 encontrado!
    ) else (
        REM Buscar cualquier version
        for /d %%i in ("C:\VulkanSDK\*") do (
            if exist "%%i\Include\vulkan\vulkan.h" (
                set "VULKAN_SDK=%%i"
                echo [OK] Vulkan SDK encontrado: %%i
                goto :vulkan_ok
            )
        )
        
        echo [X] ERROR: Vulkan SDK no encontrado
        echo.
        echo Por favor instala Vulkan SDK desde:
        echo https://vulkan.lunarg.com/
        echo.
        echo Despues de instalar, ejecuta este script nuevamente.
        pause
        exit /b 1
    )
)
:vulkan_ok
echo [OK] Vulkan SDK: %VULKAN_SDK%

REM Verificar CMake
where cmake >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo [X] ERROR: CMake no encontrado
    echo.
    echo Por favor instala CMake desde:
    echo https://cmake.org/download/
    pause
    exit /b 1
)
echo [OK] CMake encontrado

REM Verificar que REACTOR framework existe
if not exist "..\..\reactor\include\reactor\reactor.hpp" (
    echo [X] ERROR: REACTOR Framework no encontrado
    echo.
    echo Este template debe estar en: REACTOR/templates/starter/
    echo.
    echo Estructura esperada:
    echo   REACTOR/
    echo   ├── reactor/
    echo   └── templates/
    echo       └── starter/  ^<-- Aqui
    pause
    exit /b 1
)
echo [OK] REACTOR Framework encontrado

echo.
echo [2/4] Creando directorio de build...
if not exist build mkdir build
echo [OK] Directorio build/ creado

echo.
echo [3/4] Configurando proyecto con CMake...
echo.

REM Detectar generador
where ninja >nul 2>nul
if %ERRORLEVEL% equ 0 (
    set GENERATOR=Ninja
    echo Usando Ninja generator...
) else (
    set GENERATOR=Visual Studio 17 2022
    echo Usando Visual Studio generator...
)

REM Configurar con CMake
if "%GENERATOR%"=="Ninja" (
    cmake -S . -B build -G "Ninja" -DCMAKE_BUILD_TYPE=Release
) else (
    cmake -S . -B build -G "Visual Studio 17 2022" -A x64
)

if %ERRORLEVEL% neq 0 (
    echo.
    echo [X] ERROR: Configuracion fallida
    echo.
    echo Intenta:
    echo   1. Ejecutar desde "Developer Command Prompt for VS 2022"
    echo   2. Verificar que Visual Studio 2022 este instalado
    echo   3. Ejecutar diagnose.bat en el directorio raiz de REACTOR
    pause
    exit /b 1
)

echo.
echo [4/4] Setup completado!
echo.
echo ==========================================
echo   ✓ Setup exitoso!
echo ==========================================
echo.
echo Proximos pasos:
echo   1. Ejecuta: build.bat
echo   2. Ejecuta: run.bat
echo.
echo O manualmente:
if "%GENERATOR%"=="Ninja" (
    echo   cmake --build build
    echo   build\reactor-starter.exe
) else (
    echo   cmake --build build --config Release
    echo   build\Release\reactor-starter.exe
)
echo.
pause
