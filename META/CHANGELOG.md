# 📝 CHANGELOG - Stack-GPU-OP

Todos los cambios notables del proyecto serán documentados aquí.

El formato está basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.0.0/),
y este proyecto adhiere a [Semantic Versioning](https://semver.org/lang/es/).

---

## [0.3.0] - 2025-12-18 ✅ CUBO 3D FUNCIONANDO

### ✨ Agregado
- **Cube Renderer completo** con vertex/index buffers
- **Shaders con MVP matrices** usando push constants
- **Rotación animada** del cubo (45°/s Y, 30°/s X)
- **Iluminación por vértice** con colores cyan/teal y gris
- **Ejemplo stack-gpu-cube** completamente funcional
- Documentación META completa

### 🔧 Corregido
- **Window surface creation** - Agregadas extensiones GLFW a instancia
- **Swapchain extension** - Agregada a device
- **Shader loading paths** - Corregidas rutas relativas
- **Sincronización** - Per-image fences para evitar race conditions
- **Pipeline creation** - Vertex input attributes correctos

### 📊 Performance
- **74-80 FPS** constantes
- **8 vértices, 36 índices** (cubo optimizado)
- **1 draw call** por frame

### 📚 Documentación
- Creado `META/META.md` - Overview completo del proyecto
- Creado `META/ROADMAP.md` - Plan de desarrollo
- Creado `META/CHANGELOG.md` - Este archivo
- Actualizado `README_STACK_GPU_OP.md`

---

## [0.2.0] - 2025-12-18 - Stack-GPU-OP Headers

### ✨ Agregado
- **ISR System** - Headers completos (4 archivos)
  - `importance.hpp` - Cálculo de importancia
  - `adaptive.hpp` - Pixel sizing adaptivo
  - `temporal.hpp` - Coherencia temporal
  - `isr_system.hpp` - Sistema completo
- **ISR Shaders** - Compute shaders GLSL (3 archivos)
  - `importance.comp` - Sobel, normal variance, depth, motion
  - `adaptive.comp` - Shading rate (1x1 a 8x8)
  - `temporal.comp` - Temporal coherence (90% blend)
- **SDF Rendering** - Sistema completo
  - `primitives.hpp` - 6 primitivas SDF
  - `raymarcher.hpp` - Ray marching engine
  - `primitives.cpp` - Implementación
  - `raymarcher.cpp` - Implementación
  - `primitives.glsl` - Biblioteca GLSL
- **CSG Operations** - Union, Subtract, Intersect, Smooth variants

### 📚 Documentación
- Creado `STACK-GPU-OP.md` - Arquitectura completa
- Creado `STACK-GPU-OP-RESUMEN.md` - Resumen de implementación
- Creado `README_STACK_GPU_OP.md` - Guía de uso

---

## [0.1.0] - 2025-12-18 - REACTOR Core

### ✨ Agregado
- **Vulkan Context** con auto-detección de SDK
- **Memory Allocator** (VMA-style)
- **Buffer management** (Vertex, Index, Uniform)
- **Image management**
- **Shader loading** (SPIR-V)
- **Pipeline creation** (Graphics, Compute)
- **Command buffers**
- **Synchronization** (Fences, Semaphores)
- **Render pass**
- **Swapchain**
- **Window system** (GLFW integration)
- **Math utilities** (GLM integration)
- **Build system** (CMake + vcpkg)
- **Ejemplos básicos** (triangle, sandbox, cube-simple)

### 📚 Documentación
- Creado `README.md` principal
- Creado `BUILD_INSTRUCTIONS.md`
- Creado `QUICK_START.md`
- Creado `EMPEZAR_AQUI.md`
- Creado `GUIA_COMPLETA.md`
- Creado `LICENSE` (MIT)

### 🔧 Build System
- CMake con auto-detección de Vulkan SDK
- vcpkg para gestión de dependencias
- Scripts de automatización (quick-setup.bat, build.bat, etc.)
- Compilación automática de shaders con glslc

---

## [Unreleased] - Próximas Versiones

### 🎯 v0.4.0 - Mejoras Visuales (Planeado)
- Depth buffer
- Texturas (como LunarG cube)
- Phong shading
- MSAA

### 🎯 v0.5.0 - ISR Completo (Planeado)
- Implementación completa de ISR
- Uniforms y descriptors
- Integración con pipeline
- Ejemplo ISR demo

### 🎯 v0.6.0 - SDF Ray Marching (Planeado)
- Pipeline completo de ray marching
- Múltiples primitivas
- Iluminación avanzada
- Sombras y AO

---

## Tipos de Cambios

- `✨ Agregado` - Para nuevas características
- `🔧 Corregido` - Para correcciones de bugs
- `🔄 Cambiado` - Para cambios en funcionalidad existente
- `🗑️ Deprecado` - Para características que serán removidas
- `❌ Removido` - Para características removidas
- `🔒 Seguridad` - Para correcciones de seguridad
- `📊 Performance` - Para mejoras de rendimiento
- `📚 Documentación` - Para cambios en documentación

---

## Versionado

Este proyecto usa [Semantic Versioning](https://semver.org/lang/es/):

- **MAJOR** (X.0.0) - Cambios incompatibles en la API
- **MINOR** (0.X.0) - Nuevas características compatibles
- **PATCH** (0.0.X) - Correcciones de bugs compatibles

---

<div align="center">

**Stack-GPU-OP v0.3.0**

*REACTOR (Vulkan) + ADead-GPU Technologies*

*Actualizado*: 18 de Diciembre, 2025

</div>
