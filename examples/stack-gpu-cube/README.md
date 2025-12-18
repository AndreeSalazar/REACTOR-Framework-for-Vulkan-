# Stack-GPU-OP: Cubo 3D con SDF Ray Marching

## 🎯 Descripción

Este ejemplo demuestra **Stack-GPU-OP** - la integración de tecnologías de ADead-GPU implementadas **100% en Vulkan puro**.

**NO usa DirectX 12** - Todo está implementado con Vulkan.

## ✨ Tecnologías Utilizadas

### De ADead-GPU (adaptadas a Vulkan):
- ✅ **SDF Ray Marching** - Renderizado matemático (ADead-Vector3D)
- ✅ **SDF Anti-Aliasing** - Anti-aliasing perfecto usando `fwidth()` (ADead-AA)
- ✅ **CSG Operations** - Union, subtract, intersect (smooth variants)

### De REACTOR:
- ✅ **React-Style API** - Builder pattern, componentes declarativos
- ✅ **Vulkan Core** - Context, swapchain, render pass, pipelines
- ✅ **RAII** - Gestión automática de recursos
- ✅ **GLM Math** - Camera, Transform, matrices

## 🎨 Características del Cubo

- **Renderizado SDF**: El cubo es una función matemática, no una malla
- **Infinitamente escalable**: Zoom sin pixelado
- **Anti-aliasing perfecto**: Bordes suaves usando SDF-AA
- **Rotación animada**: 45°/s en Y, 30°/s en X
- **Iluminación simple**: Diffuse lighting

## 🏗️ Arquitectura

```
Stack-GPU-OP Cube
├── SDF Scene (CPU)
│   └── Box primitive (1x1x1)
│
├── Ray Marcher (GPU)
│   ├── Fullscreen quad (vertex shader)
│   └── Ray marching (fragment shader)
│       ├── Scene SDF evaluation
│       ├── Normal calculation
│       ├── Lighting
│       └── Anti-aliasing (fwidth)
│
└── Vulkan Pipeline
    ├── Render pass
    ├── Swapchain
    └── Command buffers
```

## 🚀 Compilar y Ejecutar

### Requisitos
- Vulkan SDK 1.4.328.1
- GLFW3, GLM (instalados con vcpkg)
- glslc (shader compiler)

### Compilar

```bash
# Recompilar proyecto
cmake --build build --config Release --target stack-gpu-cube

# Los shaders se compilan automáticamente
```

### Ejecutar

```bash
build\examples\stack-gpu-cube\Release\stack-gpu-cube.exe
```

## 📊 Salida Esperada

```
==========================================
  Stack-GPU-OP: Cubo 3D con SDF
  Vulkan Puro + ADead-Vector3D
==========================================

[✓] Ventana creada
[✓] Vulkan inicializado
[✓] Swapchain creado
[✓] Render pass creado
[✓] Escena SDF creada (cubo)
[✓] Ray marcher creado
[✓] Sincronización configurada

==========================================
  [✓] Stack-GPU-OP listo!
==========================================
Renderizando cubo con SDF Ray Marching...
Controles: ESC para salir

FPS: 2500 | Rotación: 45°
FPS: 2480 | Rotación: 90°
FPS: 2520 | Rotación: 135°
...
```

## 🎮 Controles

- **ESC** - Salir de la aplicación

## 🔧 Configuración

Puedes modificar el ray marcher en `main.cpp`:

```cpp
auto raymarcher = reactor::sdf::RayMarcher::create(ctx.device(), renderPass.handle())
    .resolution(800, 600)      // Resolución
    .maxSteps(128)             // Pasos de ray marching (calidad)
    .antialiasing(true)        // SDF Anti-Aliasing
    .softShadows(false)        // Soft shadows (futuro)
    .ambientOcclusion(false)   // AO (futuro)
    .build();
```

## 📝 Código Clave

### Crear Escena SDF (React-Style)

```cpp
auto scene = reactor::sdf::SDFScene::create()
    .addBox(reactor::sdf::Box(
        glm::vec3(0.0f, 0.0f, 0.0f),  // Centro
        glm::vec3(1.0f, 1.0f, 1.0f)   // Tamaño
    ))
    .build();
```

### Renderizar

```cpp
// En render loop
raymarcher.render(cmd, scene, view, proj);
```

## 🎯 Ventajas vs Renderizado Tradicional

| Feature | Tradicional (Mallas) | Stack-GPU-OP (SDF) |
|---------|---------------------|-------------------|
| **Tamaño** | ~1MB por modelo | ~1KB |
| **Escalabilidad** | Pixelado al zoom | Infinita |
| **Anti-aliasing** | MSAA (costoso) | Perfecto (gratis) |
| **Formas** | Solo triángulos | Cualquier matemática |
| **Memoria** | Alta | Mínima |

## 🔬 Detalles Técnicos

### SDF del Cubo

```glsl
float sdBox(vec3 p, vec3 center, vec3 size) {
    vec3 q = abs(p - center) - size;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}
```

### Ray Marching Loop

```glsl
for (uint i = 0; i < maxSteps; i++) {
    vec3 p = ro + rd * t;
    float d = sceneSDF(p);
    
    if (d < epsilon) {
        // Hit! Calcular iluminación
        return color;
    }
    
    t += d;  // Sphere tracing
}
```

### Anti-Aliasing

```glsl
float sdfAntialiasing(float dist) {
    float fw = fwidth(dist);
    return smoothstep(-fw, fw, dist);
}
```

## 🚀 Próximas Mejoras

- [ ] Texturas en caras del cubo (como en la imagen de referencia)
- [ ] Soft shadows con cone tracing
- [ ] Ambient occlusion
- [ ] Múltiples primitivas (escena compleja)
- [ ] ISR integration (75% performance boost)

## 📚 Referencias

- **ADead-Vector3D**: Renderizado SDF matemático
- **ADead-AA**: Anti-aliasing con SDF
- **REACTOR**: Framework Vulkan con React-Style API

---

**Stack-GPU-OP v0.1.0** - Vulkan Puro + ADead-GPU Technologies
