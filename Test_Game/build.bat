@echo off
setlocal

REM Cambiar al directorio donde esta el script
cd /d "%~dp0"

echo ==========================================
echo   Building Test Game (Standalone)
echo ==========================================
echo.
echo Directorio: %CD%
echo.

if not exist "build" (
    echo Creando carpeta build...
    mkdir build
)

cd build

echo [1/2] Configurando CMake...
cmake .. -G "Visual Studio 17 2022" -A x64
if %ERRORLEVEL% NEQ 0 (
    echo Error al configurar CMake
    cd ..
    exit /b 1
)

echo [2/2] Compilando test-game...
cmake --build . --config Debug
if %ERRORLEVEL% NEQ 0 (
    echo Error al compilar test-game
    cd ..
    exit /b 1
)

cd ..

echo.
echo ==========================================
echo   Build completado exitosamente
echo ==========================================
echo.
echo Para ejecutar:
echo   build\Debug\test-game.exe
echo.
