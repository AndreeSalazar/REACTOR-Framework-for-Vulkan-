@echo off
REM REACTOR Framework - Configure Script for Windows

echo ========================================
echo REACTOR Framework - Configuration
echo ========================================
echo.

REM Check for Vulkan SDK
if not defined VULKAN_SDK (
    echo [INFO] VULKAN_SDK variable no configurada, buscando instalacion...
    
    REM Buscar versiones comunes de Vulkan SDK
    if exist "C:\VulkanSDK\1.4.328.1" (
        set "VULKAN_SDK=C:\VulkanSDK\1.4.328.1"
        echo [OK] Vulkan SDK encontrado automaticamente: %VULKAN_SDK%
    ) else if exist "C:\VulkanSDK\1.3.290.0" (
        set "VULKAN_SDK=C:\VulkanSDK\1.3.290.0"
        echo [OK] Vulkan SDK encontrado automaticamente: %VULKAN_SDK%
    ) else if exist "C:\VulkanSDK\1.3.280.0" (
        set "VULKAN_SDK=C:\VulkanSDK\1.3.280.0"
        echo [OK] Vulkan SDK encontrado automaticamente: %VULKAN_SDK%
    ) else (
        REM Buscar cualquier version en C:\VulkanSDK
        for /d %%i in ("C:\VulkanSDK\*") do (
            if exist "%%i\Include\vulkan\vulkan.h" (
                set "VULKAN_SDK=%%i"
                echo [OK] Vulkan SDK encontrado automaticamente: %%i
                goto :vulkan_found
            )
        )
        
        echo [X] ERROR: Vulkan SDK no encontrado
        echo Por favor instala Vulkan SDK desde https://vulkan.lunarg.com/
        exit /b 1
    )
)
:vulkan_found
echo [OK] Vulkan SDK: %VULKAN_SDK%

REM Check for CMake
where cmake >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo ERROR: CMake not found in PATH
    echo Please install CMake from https://cmake.org/
    exit /b 1
)
echo [OK] CMake found

REM Check for Ninja (optional)
where ninja >nul 2>nul
if %ERRORLEVEL% equ 0 (
    set GENERATOR=Ninja
    echo [OK] Ninja found - using Ninja generator
) else (
    set GENERATOR=Visual Studio 17 2022
    echo [INFO] Ninja not found - using Visual Studio generator
)

REM Setup Visual Studio environment
if "%GENERATOR%"=="Visual Studio 17 2022" (
    echo.
    echo Setting up Visual Studio 2022 environment...
    
    REM Try to find vcvarsall.bat
    set "VCVARSALL=C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat"
    if not exist "%VCVARSALL%" (
        set "VCVARSALL=C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvarsall.bat"
    )
    if not exist "%VCVARSALL%" (
        set "VCVARSALL=C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvarsall.bat"
    )
    if not exist "%VCVARSALL%" (
        set "VCVARSALL=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat"
    )
    
    if exist "%VCVARSALL%" (
        call "%VCVARSALL%" x64
        echo [OK] Visual Studio environment configured
    ) else (
        echo [WARNING] Could not find vcvarsall.bat
        echo Visual Studio may not be properly configured
    )
)

REM Create build directory
if not exist build mkdir build

REM Configure with CMake
echo.
echo Configuring REACTOR Framework...
echo Generator: %GENERATOR%
echo Build Type: Release
echo.

if "%GENERATOR%"=="Ninja" (
    cmake -S . -B build -G "Ninja" -DCMAKE_BUILD_TYPE=Release -DREACTOR_ENABLE_VALIDATION=ON
) else (
    cmake -S . -B build -G "Visual Studio 17 2022" -A x64 -DREACTOR_ENABLE_VALIDATION=ON
)

if %ERRORLEVEL% neq 0 (
    echo.
    echo ========================================
    echo ERROR: CMake configuration failed
    echo ========================================
    echo.
    echo Troubleshooting:
    echo 1. Make sure Visual Studio 2022 is installed with C++ tools
    echo 2. Try running this from "Developer Command Prompt for VS 2022"
    echo 3. Check that VULKAN_SDK is set correctly
    echo.
    exit /b 1
)

echo.
echo ========================================
echo Configuration successful!
echo ========================================
echo.
echo To build REACTOR:
echo   cmake --build build --config Release
echo.
echo To run examples:
echo   build\examples\triangle\Release\reactor-triangle.exe
echo.
