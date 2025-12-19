@echo off
setlocal

REM Cambiar al directorio donde esta el script
cd /d "%~dp0"

echo ==========================================
echo   Ejecutando Test Game
echo ==========================================
echo.

if not exist "build\Debug\test-game.exe" (
    echo Error: test-game.exe no encontrado
    echo.
    echo Buscando en: "%CD%\build\Debug\test-game.exe"
    echo.
    echo Por favor compila primero con:
    echo   1. compile-shaders.bat
    echo   2. build.bat
    echo.
    echo O usa quick-start.bat para hacer todo automaticamente
    pause
    exit /b 1
)

echo Iniciando test-game.exe...
echo.
build\Debug\test-game.exe

echo.
echo ==========================================
echo   Test Game finalizado
echo ==========================================
pause
