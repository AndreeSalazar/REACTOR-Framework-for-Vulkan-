# REACTOR - Animated 3D Cube Example

## 🎯 Descripción

Este ejemplo demuestra un **cubo 3D completamente animado** usando REACTOR Framework con un enfoque React-Style.

## ✨ Características

- ✅ **Cubo 3D con 6 caras de colores**
- ✅ **Animación automática** (rotación continua)
- ✅ **Cámara 3D** con perspectiva
- ✅ **Transformaciones MVP** (Model-View-Projection)
- ✅ **Shaders GLSL** compilados automáticamente
- ✅ **React-Style components** (Transform, Camera)
- ✅ **Uniform buffers** para matrices
- ✅ **Index buffer** para optimización

## 🎨 Colores del Cubo

- **Frontal**: Rojo
- **Trasera**: Verde
- **Superior**: Azul
- **Inferior**: Amarillo
- **Derecha**: Magenta
- **Izquierda**: Cyan

## 🚀 Compilar y Ejecutar

### Opción 1: Desde el directorio raíz

```bash
# Compilar todo el proyecto
cmake --build build --config Release

# Ejecutar
build\examples\cube\Release\reactor-cube.exe
```

### Opción 2: Recompilar solo este ejemplo

```bash
cmake --build build --config Release --target reactor-cube
build\examples\cube\Release\reactor-cube.exe
```

## 📁 Estructura

```
cube/
├── main.cpp              # Aplicación principal
├── shaders/
│   ├── cube.vert        # Vertex shader (GLSL)
│   └── cube.frag        # Fragment shader (GLSL)
├── CMakeLists.txt       # Build configuration
└── README.md            # Este archivo
```

## 🎓 Conceptos Demostrados

### 1. React-Style Components

```cpp
// Transform component (como React state)
reactor::Transform cubeTransform;
cubeTransform.rotation.y = time * glm::radians(90.0f);

// Camera component (como React props)
reactor::Camera camera;
camera.position = reactor::Vec3(2.0f, 2.0f, 2.0f);
```

### 2. Vertex Data Structure

```cpp
struct Vertex {
    reactor::Vec3 pos;    // Posición 3D
    reactor::Vec3 color;  // Color RGB
};
```

### 3. MVP Matrices

```cpp
reactor::UniformBufferObject ubo;
ubo.model = cubeTransform.getMatrix();  // Transformación del objeto
ubo.view = camera.getViewMatrix();      // Posición de cámara
ubo.proj = camera.getProjectionMatrix(); // Proyección perspectiva
```

### 4. Render Loop

```cpp
while (!window.shouldClose()) {
    // 1. Update state (React-style)
    cubeTransform.rotation.y += deltaTime;
    
    // 2. Update uniforms
    uniformBuffer.upload(&ubo, sizeof(ubo));
    
    // 3. Record commands
    cmd.bindPipeline(pipeline);
    cmd.bindVertexBuffers(vertexBuffer);
    cmd.bindIndexBuffer(indexBuffer);
    cmd.drawIndexed(indices.size());
    
    // 4. Present
    swapchain.present();
}
```

## 🎮 Controles

- **ESC** - Salir de la aplicación
- El cubo rota automáticamente

## 📊 Performance

- **FPS Counter** - Muestra en consola cada segundo
- **Rotation Angle** - Muestra ángulo actual de rotación
- **VSync** - Habilitado por defecto (60 FPS)

## 🔧 Personalización

### Cambiar Velocidad de Rotación

```cpp
// En main.cpp, línea ~200
cubeTransform.rotation.y = time * glm::radians(90.0f);  // 90° por segundo
// Cambiar a:
cubeTransform.rotation.y = time * glm::radians(180.0f); // 180° por segundo
```

### Cambiar Posición de Cámara

```cpp
// En main.cpp, línea ~170
camera.position = reactor::Vec3(2.0f, 2.0f, 2.0f);
// Cambiar a:
camera.position = reactor::Vec3(5.0f, 3.0f, 5.0f); // Más lejos
```

### Cambiar Colores

```cpp
// En main.cpp, líneas ~30-60
// Modificar los valores RGB de cada cara
{{-0.5f, -0.5f,  0.5f}, {1.0f, 0.0f, 0.0f}},  // Rojo
// Cambiar a:
{{-0.5f, -0.5f,  0.5f}, {0.0f, 1.0f, 1.0f}},  // Cyan
```

## 🎯 Próximos Pasos

1. **Agregar texturas** - Usar STB para cargar imágenes
2. **Iluminación** - Implementar Phong/PBR
3. **Múltiples cubos** - Instancing
4. **Input interactivo** - Rotar con mouse
5. **Física** - Integrar Bullet3

## 📚 Recursos

- [Vulkan Tutorial - Uniform Buffers](https://vulkan-tutorial.com/Uniform_buffers)
- [GLM Documentation](https://github.com/g-truc/glm)
- [GLSL Reference](https://www.khronos.org/opengl/wiki/OpenGL_Shading_Language)

## 🐛 Troubleshooting

### Shaders no compilan

```bash
# Verificar que glslc esté disponible
%VULKAN_SDK%\Bin\glslc.exe --version

# Compilar manualmente
cd examples\cube\shaders
glslc cube.vert -o cube.vert.spv
glslc cube.frag -o cube.frag.spv
```

### Cubo no se ve

- Verifica que la cámara esté posicionada correctamente
- Asegúrate de que el cubo no esté fuera del frustum
- Revisa que depth test esté habilitado

### Performance bajo

- Deshabilita validation layers en Release
- Verifica que VSync esté configurado correctamente
- Usa buffers device-local para mejor performance

---

**¡Disfruta experimentando con REACTOR!** 🎉
