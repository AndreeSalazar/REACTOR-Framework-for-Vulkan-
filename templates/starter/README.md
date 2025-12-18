# REACTOR Starter Template

Este es el template más simple para comenzar con REACTOR Framework.

## 🚀 Quick Start (5 minutos)

### Paso 1: Verificar Requisitos

```bash
# Verificar que tienes todo instalado
reactor-check.bat
```

**Necesitas**:
- ✅ Vulkan SDK instalado
- ✅ Visual Studio 2022 o Build Tools
- ✅ CMake 3.24+

### Paso 2: Configurar Proyecto

```bash
# Configurar automáticamente
setup.bat
```

### Paso 3: Compilar

```bash
# Compilar proyecto
build.bat
```

### Paso 4: Ejecutar

```bash
# Ejecutar aplicación
run.bat
```

¡Eso es todo! Tu aplicación Vulkan está corriendo.

## 📁 Estructura del Proyecto

```
mi-app/
├── src/
│   └── main.cpp              # Tu aplicación principal
├── assets/
│   ├── shaders/              # Shaders GLSL
│   ├── textures/             # Texturas
│   └── models/               # Modelos 3D
├── CMakeLists.txt            # Configuración de build
├── setup.bat                 # Script de configuración
├── build.bat                 # Script de compilación
└── run.bat                   # Script de ejecución
```

## 🎯 Qué Incluye Este Template

- ✅ **Ventana con GLFW** - Lista para usar
- ✅ **Render loop** - Automático
- ✅ **Triángulo de ejemplo** - Funcional
- ✅ **Hot-reload** - Shaders se recargan automáticamente
- ✅ **ImGui integrado** - UI lista para usar
- ✅ **Input handling** - Teclado y mouse

## 📝 Personalizar

### Cambiar el Título de la Ventana

```cpp
// En src/main.cpp
auto app = reactor::App::create()
    .window({
        .title = "Mi Aplicación Increíble",  // ← Cambiar aquí
        .width = 1920,
        .height = 1080
    })
    .build();
```

### Agregar Tus Propios Objetos

```cpp
// Crear un cubo
auto cube = scene.add<reactor::CubeComponent>({
    .position = {0, 0, 0},
    .color = {1, 0, 0},
    .scale = 2.0f
});

// Crear múltiples objetos
for (int i = 0; i < 10; i++) {
    scene.add<reactor::CubeComponent>({
        .position = {i * 2.0f, 0, 0},
        .color = {float(i)/10, 0, 1}
    });
}
```

### Agregar UI con ImGui

```cpp
// En el render loop
ui.render([&]() {
    ImGui::Begin("Mi Panel");
    ImGui::Text("FPS: %.1f", fps);
    ImGui::SliderFloat("Velocidad", &speed, 0.0f, 10.0f);
    if (ImGui::Button("Reset")) {
        reset();
    }
    ImGui::End();
});
```

## 🎨 Próximos Pasos

1. **Modificar `src/main.cpp`** - Agregar tu lógica
2. **Crear shaders** en `assets/shaders/`
3. **Agregar texturas** en `assets/textures/`
4. **Ver ejemplos** en la documentación

## 📚 Documentación

- [USAGE_GUIDE.md](../../USAGE_GUIDE.md) - Guía completa
- [ARCHITECTURE.md](../../ARCHITECTURE.md) - Arquitectura
- [examples/](../../examples/) - Más ejemplos

## 🐛 Problemas?

```bash
# Ejecutar diagnóstico
diagnose.bat

# Ver guía de troubleshooting
# TROUBLESHOOTING.md
```

## 🎓 Tutoriales

### Tutorial 1: Cambiar Color del Triángulo

```cpp
// En src/main.cpp, busca:
std::array<Vertex, 3> vertices = {{
    {{0.0f, -0.5f}, {1.0f, 0.0f, 0.0f}},  // Rojo
    {{0.5f, 0.5f}, {0.0f, 1.0f, 0.0f}},   // Verde
    {{-0.5f, 0.5f}, {0.0f, 0.0f, 1.0f}}   // Azul
}};

// Cambia los colores RGB (valores entre 0.0 y 1.0)
```

### Tutorial 2: Hacer que el Triángulo Rote

```cpp
// Agregar variable de tiempo
float time = 0.0f;

// En el update loop:
time += deltaTime;

// Aplicar rotación
transform.rotation.z = time;
```

### Tutorial 3: Agregar Más Triángulos

```cpp
// Crear múltiples triángulos
for (int i = 0; i < 5; i++) {
    auto triangle = scene.add<TriangleComponent>({
        .position = {i * 1.5f, 0, 0},
        .rotation = i * 45.0f
    });
}
```

## 🚀 Listo para Más?

Explora los ejemplos avanzados:
- `examples/textured-cube/` - Cubo con textura
- `examples/lighting/` - Sistema de iluminación
- `examples/physics/` - Física con Bullet
- `examples/game/` - Juego completo

---

**¡Feliz desarrollo con REACTOR!** 🎉
