@echo off
REM REACTOR Framework - Quick Setup (Auto-deteccion completa)

echo ==========================================
echo   REACTOR Framework - Quick Setup
echo ==========================================
echo.
echo Este script configurara automaticamente REACTOR
echo detectando tu instalacion de Vulkan SDK.
echo.
pause

echo.
echo [1/6] Detectando Vulkan SDK...
echo.

REM Auto-detectar Vulkan SDK
set VULKAN_FOUND=0

if defined VULKAN_SDK (
    echo [OK] VULKAN_SDK ya configurado: %VULKAN_SDK%
    set VULKAN_FOUND=1
) else (
    echo [INFO] Buscando Vulkan SDK en ubicaciones comunes...
    
    REM Buscar version especifica del usuario
    if exist "C:\VulkanSDK\1.4.328.1\Include\vulkan\vulkan.h" (
        set "VULKAN_SDK=C:\VulkanSDK\1.4.328.1"
        echo [OK] Encontrado Vulkan SDK 1.4.328.1
        set VULKAN_FOUND=1
    ) else if exist "C:\VulkanSDK\1.3.290.0\Include\vulkan\vulkan.h" (
        set "VULKAN_SDK=C:\VulkanSDK\1.3.290.0"
        echo [OK] Encontrado Vulkan SDK 1.3.290.0
        set VULKAN_FOUND=1
    ) else (
        REM Buscar cualquier version
        for /d %%i in ("C:\VulkanSDK\*") do (
            if exist "%%i\Include\vulkan\vulkan.h" (
                set "VULKAN_SDK=%%i"
                echo [OK] Encontrado Vulkan SDK: %%i
                set VULKAN_FOUND=1
                goto :vulkan_detected
            )
        )
    )
)

:vulkan_detected
if %VULKAN_FOUND%==0 (
    echo [X] ERROR: Vulkan SDK no encontrado
    echo.
    echo Por favor:
    echo   1. Descarga Vulkan SDK desde: https://vulkan.lunarg.com/
    echo   2. Instala el SDK
    echo   3. Ejecuta este script nuevamente
    echo.
    pause
    exit /b 1
)

echo     Ubicacion: %VULKAN_SDK%
echo.

echo [2/6] Verificando CMake...
where cmake >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo [X] ERROR: CMake no encontrado
    echo.
    echo Por favor instala CMake desde: https://cmake.org/download/
    pause
    exit /b 1
)
cmake --version | findstr /C:"version"
echo [OK] CMake encontrado
echo.

echo [3/6] Verificando compilador...
where cl >nul 2>nul
if %ERRORLEVEL% equ 0 (
    echo [OK] MSVC encontrado en PATH
) else (
    echo [INFO] MSVC no en PATH, buscando Visual Studio...
    
    REM Buscar vcvarsall.bat
    set "VCVARSALL="
    if exist "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat" (
        set "VCVARSALL=C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat"
    ) else if exist "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvarsall.bat" (
        set "VCVARSALL=C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvarsall.bat"
    ) else if exist "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat" (
        set "VCVARSALL=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat"
    )
    
    if defined VCVARSALL (
        echo [OK] Visual Studio 2022 encontrado
        echo [INFO] Configurando entorno...
        call "%VCVARSALL%" x64 >nul 2>nul
    ) else (
        echo [X] ERROR: Visual Studio 2022 no encontrado
        echo.
        echo Por favor instala Visual Studio 2022 con:
        echo   - Desktop development with C++
        echo.
        pause
        exit /b 1
    )
)
echo.

echo [4/6] Detectando generador...
where ninja >nul 2>nul
if %ERRORLEVEL% equ 0 (
    set GENERATOR=Ninja
    echo [OK] Ninja encontrado - usando Ninja generator
) else (
    set GENERATOR=Visual Studio 17 2022
    echo [INFO] Ninja no encontrado - usando Visual Studio generator
)
echo.

echo [5/6] Configurando proyecto...
if not exist build mkdir build

if "%GENERATOR%"=="Ninja" (
    cmake -S . -B build -G "Ninja" -DCMAKE_BUILD_TYPE=Release -DVULKAN_SDK="%VULKAN_SDK%"
) else (
    cmake -S . -B build -G "Visual Studio 17 2022" -A x64 -DVULKAN_SDK="%VULKAN_SDK%"
)

if %ERRORLEVEL% neq 0 (
    echo.
    echo [X] ERROR: Configuracion fallida
    echo.
    echo Intenta ejecutar desde "Developer Command Prompt for VS 2022"
    pause
    exit /b 1
)
echo [OK] Proyecto configurado
echo.

echo [6/6] Compilando REACTOR Framework...
echo.
cmake --build build --config Release

if %ERRORLEVEL% neq 0 (
    echo.
    echo [X] ERROR: Compilacion fallida
    echo.
    echo Verifica los errores arriba y consulta TROUBLESHOOTING.md
    pause
    exit /b 1
)

echo.
echo ==========================================
echo   ✓ REACTOR Framework listo!
echo ==========================================
echo.
echo Configuracion detectada:
echo   • Vulkan SDK: %VULKAN_SDK%
echo   • Generador: %GENERATOR%
echo   • Build: Release
echo.
echo Ejecutables compilados:
if "%GENERATOR%"=="Ninja" (
    echo   • build\examples\triangle\reactor-triangle.exe
    echo   • build\examples\sandbox\reactor-sandbox.exe
) else (
    echo   • build\examples\triangle\Release\reactor-triangle.exe
    echo   • build\examples\sandbox\Release\reactor-sandbox.exe
)
echo.
echo Para ejecutar el ejemplo:
if "%GENERATOR%"=="Ninja" (
    echo   build\examples\triangle\reactor-triangle.exe
) else (
    echo   build\Release\examples\triangle\reactor-triangle.exe
)
echo.
echo Para usar el template starter:
echo   cd templates\starter
echo   setup.bat
echo.
pause
