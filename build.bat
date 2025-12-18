@echo off
REM REACTOR Framework - Build Script for Windows

echo ========================================
echo REACTOR Framework - Build
echo ========================================
echo.

REM Check if build directory exists
if not exist build (
    echo Build directory not found. Running configuration...
    call configure.bat
    if %ERRORLEVEL% neq 0 exit /b 1
)

REM Determine build configuration
set BUILD_TYPE=Release
if "%1"=="debug" set BUILD_TYPE=Debug
if "%1"=="Debug" set BUILD_TYPE=Debug

echo Building REACTOR Framework (%BUILD_TYPE%)...
echo.

cmake --build build --config %BUILD_TYPE%

if %ERRORLEVEL% neq 0 (
    echo.
    echo ========================================
    echo ERROR: Build failed
    echo ========================================
    exit /b 1
)

echo.
echo ========================================
echo Build successful!
echo ========================================
echo.
echo Executables:
if "%BUILD_TYPE%"=="Release" (
    echo   build\examples\sandbox\Release\reactor-sandbox.exe
    echo   build\examples\triangle\Release\reactor-triangle.exe
) else (
    echo   build\examples\sandbox\Debug\reactor-sandbox.exe
    echo   build\examples\triangle\Debug\reactor-triangle.exe
)
echo.
