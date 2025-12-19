@echo off
setlocal

REM Cambiar al directorio donde esta el script
cd /d "%~dp0"

echo ==========================================
echo   Compilando Shaders - Test Game
echo ==========================================
echo.
echo Directorio actual: %CD%
echo.

if not exist "shaders" (
    echo Error: Carpeta shaders no encontrada
    echo Buscando en: "%CD%\shaders"
    exit /b 1
)

cd shaders

echo [1/2] Compilando vertex shader...
glslc cube.vert -o cube.vert.spv
if %ERRORLEVEL% NEQ 0 (
    echo Error al compilar cube.vert
    cd ..
    exit /b 1
)
echo       OK - cube.vert.spv

echo [2/2] Compilando fragment shader...
glslc cube.frag -o cube.frag.spv
if %ERRORLEVEL% NEQ 0 (
    echo Error al compilar cube.frag
    cd ..
    exit /b 1
)
echo       OK - cube.frag.spv

cd ..

echo.
echo ==========================================
echo   Shaders compilados exitosamente
echo ==========================================
echo.
