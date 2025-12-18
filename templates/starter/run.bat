@echo off
REM REACTOR Starter Template - Run Script

echo ==========================================
echo   REACTOR Starter - Run
echo ==========================================
echo.

REM Buscar ejecutable
set EXE_PATH=

if exist "build\reactor-starter.exe" (
    set EXE_PATH=build\reactor-starter.exe
) else if exist "build\Release\reactor-starter.exe" (
    set EXE_PATH=build\Release\reactor-starter.exe
) else if exist "build\Debug\reactor-starter.exe" (
    set EXE_PATH=build\Debug\reactor-starter.exe
)

if "%EXE_PATH%"=="" (
    echo ERROR: Ejecutable no encontrado
    echo.
    echo Por favor ejecuta primero:
    echo   1. setup.bat
    echo   2. build.bat
    echo.
    pause
    exit /b 1
)

echo Ejecutando: %EXE_PATH%
echo.
echo ==========================================
echo.

REM Ejecutar aplicacion
"%EXE_PATH%"

set EXIT_CODE=%ERRORLEVEL%

echo.
echo ==========================================
echo   Aplicacion finalizada (codigo: %EXIT_CODE%)
echo ==========================================
echo.

if %EXIT_CODE% neq 0 (
    echo La aplicacion termino con errores.
    echo.
    echo Verifica:
    echo   1. Que Vulkan SDK este instalado correctamente
    echo   2. Que los drivers de GPU esten actualizados
    echo   3. Ejecuta diagnose.bat para mas informacion
    echo   4. Consulta TROUBLESHOOTING.md
    echo.
)

pause
