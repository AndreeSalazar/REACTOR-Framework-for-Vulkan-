@echo off
echo Compilando shaders...

set GLSLC=%VULKAN_SDK%\Bin\glslc.exe

if not exist "%GLSLC%" (
    echo Error: glslc no encontrado en %VULKAN_SDK%\Bin\
    pause
    exit /b 1
)

"%GLSLC%" shaders\cube.vert -o shaders\cube.vert.spv
"%GLSLC%" shaders\cube.frag -o shaders\cube.frag.spv

echo Shaders compilados exitosamente!
pause
