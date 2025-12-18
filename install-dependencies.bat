@echo off
REM REACTOR Framework - Instalador de Dependencias

echo ==========================================
echo   REACTOR - Instalador de Dependencias
echo ==========================================
echo.
echo Este script instalara las dependencias necesarias
echo para REACTOR Framework usando vcpkg.
echo.
echo Dependencias a instalar:
echo   - GLFW3 (Sistema de ventanas)
echo   - GLM (Matematicas)
echo   - STB (Carga de imagenes)
echo.
pause

echo.
echo [1/5] Verificando vcpkg...
echo.

REM Buscar vcpkg
set VCPKG_ROOT=
if exist "vcpkg\vcpkg.exe" (
    set VCPKG_ROOT=vcpkg
    echo [OK] vcpkg encontrado en: vcpkg\
) else if exist "..\vcpkg\vcpkg.exe" (
    set VCPKG_ROOT=..\vcpkg
    echo [OK] vcpkg encontrado en: ..\vcpkg\
) else if exist "C:\vcpkg\vcpkg.exe" (
    set VCPKG_ROOT=C:\vcpkg
    echo [OK] vcpkg encontrado en: C:\vcpkg\
) else (
    echo [INFO] vcpkg no encontrado, descargando...
    goto :install_vcpkg
)

goto :vcpkg_ready

:install_vcpkg
echo.
echo [2/5] Descargando vcpkg...
git clone https://github.com/Microsoft/vcpkg.git
if %ERRORLEVEL% neq 0 (
    echo [X] ERROR: No se pudo clonar vcpkg
    echo Asegurate de tener Git instalado
    pause
    exit /b 1
)

echo.
echo [3/5] Compilando vcpkg...
cd vcpkg
call bootstrap-vcpkg.bat
if %ERRORLEVEL% neq 0 (
    echo [X] ERROR: No se pudo compilar vcpkg
    pause
    exit /b 1
)
cd ..
set VCPKG_ROOT=vcpkg

:vcpkg_ready
echo.
echo [4/5] Instalando dependencias desde vcpkg.json...
echo.

REM En manifest mode, vcpkg lee vcpkg.json automáticamente
echo Instalando todas las dependencias (GLFW3, GLM, STB)...
"%VCPKG_ROOT%\vcpkg.exe" install --triplet x64-windows
if %ERRORLEVEL% neq 0 (
    echo [X] ERROR: No se pudieron instalar las dependencias
    echo.
    echo Intentando con modo clasico...
    "%VCPKG_ROOT%\vcpkg.exe" install glfw3:x64-windows glm:x64-windows stb:x64-windows --classic
    if %ERRORLEVEL% neq 0 (
        echo [X] ERROR: Instalacion fallida
        pause
        exit /b 1
    )
)

echo.
echo [5/5] Integrando con CMake...
"%VCPKG_ROOT%\vcpkg.exe" integrate install

echo.
echo ==========================================
echo   ✓ Dependencias instaladas!
echo ==========================================
echo.
echo Dependencias instaladas:
echo   ✓ GLFW3 - Sistema de ventanas
echo   ✓ GLM - Matematicas 3D
echo   ✓ STB - Carga de imagenes
echo.
echo Siguiente paso:
echo   1. Ejecuta: quick-setup.bat
echo   2. O manualmente:
echo      cmake -S . -B build -DCMAKE_TOOLCHAIN_FILE=%VCPKG_ROOT%\scripts\buildsystems\vcpkg.cmake
echo      cmake --build build --config Release
echo.
pause
