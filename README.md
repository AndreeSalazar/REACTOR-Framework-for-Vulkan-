# REACTOR (Framework para Vulkan)

REACTOR es un framework inspirado en el modelo declarativo de React aplicado a Vulkan puro. Su objetivo es ofrecer control total de Vulkan sin perder su esencia, pero con una API más sencilla para construir escenas, pipelines y sincronización de forma componible.

## Objetivos
- Declaratividad para describir recursos y operaciones de render.
- Control explícito de dispositivos, colas, sincronización y memoria.
- Integración directa con herramientas del Vulkan SDK.

## Estructura
- `reactor/include/reactor`: API pública.
- `reactor/src`: implementación interna.
- `examples/sandbox`: ejemplo mínimo de inicialización.

## Requisitos
- Vulkan SDK (`VULKAN_SDK` configurado).
- CMake (`>=3.24`) y Ninja.
- MSVC Build Tools o Visual Studio 2022.

## Compilación
```sh
cmake -S . -B build -G "Ninja" -DCMAKE_BUILD_TYPE=Release
cmake --build build
```

## Ejecución
```sh
build\examples\sandbox\reactor-sandbox.exe
```

