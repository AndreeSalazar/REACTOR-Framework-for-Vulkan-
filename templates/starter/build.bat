@echo off
REM REACTOR Starter Template - Build Script

echo ==========================================
echo   REACTOR Starter - Build
echo ==========================================
echo.

REM Verificar que setup se haya ejecutado
if not exist build (
    echo ERROR: Directorio build/ no encontrado
    echo.
    echo Por favor ejecuta primero: setup.bat
    echo.
    pause
    exit /b 1
)

REM Determinar tipo de build
set BUILD_TYPE=Release
if "%1"=="debug" set BUILD_TYPE=Debug
if "%1"=="Debug" set BUILD_TYPE=Debug

echo Compilando proyecto (%BUILD_TYPE%)...
echo.

REM Compilar
cmake --build build --config %BUILD_TYPE%

if %ERRORLEVEL% neq 0 (
    echo.
    echo ==========================================
    echo   ERROR: Compilacion fallida
    echo ==========================================
    echo.
    echo Verifica:
    echo   1. Que setup.bat se haya ejecutado correctamente
    echo   2. Que no haya errores de sintaxis en src/main.cpp
    echo   3. Ejecuta diagnose.bat para mas informacion
    echo.
    pause
    exit /b 1
)

echo.
echo ==========================================
echo   ✓ Compilacion exitosa!
echo ==========================================
echo.
echo Para ejecutar:
echo   run.bat
echo.
echo O manualmente:
if "%BUILD_TYPE%"=="Release" (
    if exist "build\reactor-starter.exe" (
        echo   build\reactor-starter.exe
    ) else (
        echo   build\Release\reactor-starter.exe
    )
) else (
    if exist "build\reactor-starter.exe" (
        echo   build\reactor-starter.exe
    ) else (
        echo   build\Debug\reactor-starter.exe
    )
)
echo.
pause
