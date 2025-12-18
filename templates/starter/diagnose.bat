@echo off
REM REACTOR Starter Template - Diagnostic Script

echo ==========================================
echo   REACTOR Starter - Diagnostics
echo ==========================================
echo.

echo [1] Verificando Vulkan SDK...
if defined VULKAN_SDK (
    echo [OK] VULKAN_SDK = %VULKAN_SDK%
    if exist "%VULKAN_SDK%\Bin\vulkaninfo.exe" (
        echo [OK] vulkaninfo.exe encontrado
    ) else (
        echo [X] vulkaninfo.exe no encontrado
    )
) else (
    echo [X] VULKAN_SDK no configurado
    echo.
    echo SOLUCION:
    echo   1. Instala Vulkan SDK desde: https://vulkan.lunarg.com/
    echo   2. Reinicia tu terminal
    echo   3. Ejecuta este script nuevamente
)
echo.

echo [2] Verificando CMake...
where cmake >nul 2>nul
if %ERRORLEVEL% equ 0 (
    cmake --version | findstr /C:"version"
    echo [OK] CMake encontrado
) else (
    echo [X] CMake no encontrado
    echo.
    echo SOLUCION:
    echo   Instala CMake desde: https://cmake.org/download/
)
echo.

echo [3] Verificando Ninja...
where ninja >nul 2>nul
if %ERRORLEVEL% equ 0 (
    ninja --version
    echo [OK] Ninja encontrado
) else (
    echo [INFO] Ninja no encontrado (opcional)
    echo        Se usara Visual Studio generator
)
echo.

echo [4] Verificando Visual Studio...
where cl >nul 2>nul
if %ERRORLEVEL% equ 0 (
    cl 2>&1 | findstr /C:"Version"
    echo [OK] MSVC compiler encontrado
) else (
    echo [X] MSVC compiler no encontrado en PATH
    echo.
    echo SOLUCION:
    echo   1. Instala Visual Studio 2022 con "Desktop development with C++"
    echo   2. O ejecuta desde "Developer Command Prompt for VS 2022"
)
echo.

echo [5] Verificando REACTOR Framework...
if exist "..\..\reactor\include\reactor\reactor.hpp" (
    echo [OK] REACTOR Framework encontrado
    echo      Ubicacion: ..\..\reactor\
) else (
    echo [X] REACTOR Framework no encontrado
    echo.
    echo SOLUCION:
    echo   Este template debe estar en: REACTOR/templates/starter/
)
echo.

echo [6] Verificando estructura del proyecto...
if exist "src\main.cpp" (
    echo [OK] src\main.cpp existe
) else (
    echo [X] src\main.cpp no encontrado
)

if exist "CMakeLists.txt" (
    echo [OK] CMakeLists.txt existe
) else (
    echo [X] CMakeLists.txt no encontrado
)
echo.

echo [7] Verificando GPU y drivers...
if defined VULKAN_SDK (
    if exist "%VULKAN_SDK%\Bin\vulkaninfo.exe" (
        echo Ejecutando vulkaninfo...
        echo.
        "%VULKAN_SDK%\Bin\vulkaninfo.exe" --summary
    )
)
echo.

echo ==========================================
echo   Diagnostico completo
echo ==========================================
echo.
echo Si hay errores [X], sigue las soluciones indicadas.
echo.
echo Para mas ayuda:
echo   - Consulta TROUBLESHOOTING.md en el directorio raiz de REACTOR
echo   - Ejecuta ..\..\diagnose.bat para diagnostico completo del framework
echo.
pause
