@echo off
REM REACTOR Framework - Verificacion Rapida

echo ==========================================
echo   REACTOR - Verificacion del Sistema
echo ==========================================
echo.

set ALL_OK=1

echo [1] Vulkan SDK...
if exist "C:\VulkanSDK\1.4.328.1\Include\vulkan\vulkan.h" (
    echo     [OK] Vulkan SDK 1.4.328.1 instalado
    echo     Ubicacion: C:\VulkanSDK\1.4.328.1
) else (
    echo     [X] Vulkan SDK 1.4.328.1 no encontrado
    set ALL_OK=0
)
echo.

echo [2] CMake...
where cmake >nul 2>nul
if %ERRORLEVEL% equ 0 (
    for /f "tokens=3" %%v in ('cmake --version ^| findstr /C:"version"') do echo     [OK] CMake %%v
) else (
    echo     [X] CMake no encontrado
    set ALL_OK=0
)
echo.

echo [3] Compilador...
where cl >nul 2>nul
if %ERRORLEVEL% equ 0 (
    echo     [OK] MSVC encontrado
) else (
    echo     [INFO] MSVC no en PATH (normal si no estas en Developer Command Prompt)
)
echo.

echo [4] REACTOR Framework...
if exist "reactor\include\reactor\reactor.hpp" (
    echo     [OK] Framework encontrado
) else (
    echo     [X] Framework no encontrado
    set ALL_OK=0
)
echo.

echo [5] Ejemplos compilados...
if exist "build\examples\triangle\reactor-triangle.exe" (
    echo     [OK] Ejemplo triangle compilado (Ninja)
) else if exist "build\examples\triangle\Release\reactor-triangle.exe" (
    echo     [OK] Ejemplo triangle compilado (Visual Studio)
) else if exist "build\Release\examples\triangle\reactor-triangle.exe" (
    echo     [OK] Ejemplo triangle compilado (Visual Studio)
) else (
    echo     [INFO] Ejemplos no compilados aun
    echo     Ejecuta: quick-setup.bat
)
echo.

echo ==========================================
if %ALL_OK%==1 (
    echo   ✓ Sistema listo para REACTOR
) else (
    echo   ! Algunos componentes faltan
)
echo ==========================================
echo.

if %ALL_OK%==1 (
    echo Siguiente paso:
    echo   1. Si no has compilado: quick-setup.bat
    echo   2. Para crear proyecto: cd templates\starter
    echo   3. Ver documentacion: EMPEZAR_AQUI.md
) else (
    echo Para resolver problemas:
    echo   1. Instala componentes faltantes
    echo   2. Ejecuta: quick-setup.bat
    echo   3. Consulta: TROUBLESHOOTING.md
)
echo.
pause
