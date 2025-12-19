@echo off
setlocal

REM Cambiar al directorio donde esta el script
cd /d "%~dp0"

echo ==========================================
echo   Test Game - Quick Start
echo ==========================================
echo.
echo Directorio actual: %CD%
echo.

echo Este script compilara y ejecutara el Test Game
echo.
pause

echo [1/3] Compilando shaders...
call "%~dp0compile-shaders.bat"
if %ERRORLEVEL% NEQ 0 (
    echo Error al compilar shaders
    pause
    exit /b 1
)

echo.
echo [2/3] Compilando proyecto...
call "%~dp0build.bat"
if %ERRORLEVEL% NEQ 0 (
    echo Error al compilar proyecto
    pause
    exit /b 1
)

echo.
echo [3/3] Ejecutando Test Game...
echo.
if exist "build\Debug\test-game.exe" (
    build\Debug\test-game.exe
) else (
    echo Error: No se encontro test-game.exe
    echo Buscando en: "%CD%\build\Debug\test-game.exe"
    echo Verifica que la compilacion haya sido exitosa
    pause
    exit /b 1
)

echo.
echo ==========================================
echo   Test Game finalizado
==========================================
pause
