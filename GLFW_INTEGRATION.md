# REACTOR Framework - Integración con GLFW

## 🪟 ¿Qué es GLFW?

**GLFW** (Graphics Library Framework) es una biblioteca multiplataforma para crear ventanas, contextos OpenGL/Vulkan y manejar input.

**Sitio oficial**: https://www.glfw.org/

## ✨ Características en REACTOR

### 1. Sistema de Ventanas

```cpp
#include "reactor/window.hpp"

// Configuración declarativa (React-style)
reactor::WindowConfig config;
config.title = "Mi Aplicación";
config.width = 1280;
config.height = 720;
config.fullscreen = false;
config.resizable = true;
config.vsync = true;

// Crear ventana
reactor::Window window(config);
```

### 2. Integración con Vulkan

```cpp
// Crear surface automáticamente
VkSurfaceKHR surface = window.createSurface(ctx.instance());

// Obtener tamaño del framebuffer
int width, height;
window.getFramebufferSize(&width, &height);
```

### 3. Input Handling

```cpp
// Callbacks de teclado
window.setKeyCallback([](int key, int action) {
    if (key == GLFW_KEY_ESCAPE && action == GLFW_PRESS) {
        // Salir
    }
});

// Callbacks de mouse
window.setMouseButtonCallback([](int button, int action) {
    if (button == GLFW_MOUSE_BUTTON_LEFT && action == GLFW_PRESS) {
        // Click izquierdo
    }
});

// Callbacks de movimiento de mouse
window.setMouseMoveCallback([](double x, double y) {
    // Posición del mouse
});

// Callbacks de resize
window.setResizeCallback([](int width, int height) {
    // Ventana redimensionada
});
```

### 4. Render Loop

```cpp
// Inicializar GLFW (una vez)
reactor::Window::init();

// Crear ventana
reactor::Window window(config);

// Loop principal
while (!window.shouldClose()) {
    window.pollEvents();  // Procesar eventos
    
    // Tu código de renderizado aquí
    render();
}

// Cleanup
reactor::Window::terminate();
```

## 📋 Códigos de Teclas Comunes

```cpp
// Teclas especiales
GLFW_KEY_ESCAPE      // ESC
GLFW_KEY_SPACE       // Espacio
GLFW_KEY_ENTER       // Enter
GLFW_KEY_TAB         // Tab
GLFW_KEY_BACKSPACE   // Backspace

// Flechas
GLFW_KEY_UP          // Flecha arriba
GLFW_KEY_DOWN        // Flecha abajo
GLFW_KEY_LEFT        // Flecha izquierda
GLFW_KEY_RIGHT       // Flecha derecha

// Letras (A-Z)
GLFW_KEY_A           // A
GLFW_KEY_W           // W
GLFW_KEY_S           // S
GLFW_KEY_D           // D

// Números (0-9)
GLFW_KEY_0           // 0
GLFW_KEY_1           // 1
// ...

// Modificadores
GLFW_KEY_LEFT_SHIFT  // Shift izquierdo
GLFW_KEY_LEFT_CONTROL // Ctrl izquierdo
GLFW_KEY_LEFT_ALT    // Alt izquierdo
```

## 🎮 Acciones de Input

```cpp
GLFW_PRESS    // Tecla presionada
GLFW_RELEASE  // Tecla soltada
GLFW_REPEAT   // Tecla mantenida (repetición)
```

## 🖱️ Botones de Mouse

```cpp
GLFW_MOUSE_BUTTON_LEFT    // Click izquierdo
GLFW_MOUSE_BUTTON_RIGHT   // Click derecho
GLFW_MOUSE_BUTTON_MIDDLE  // Click central (rueda)
```

## 🎯 Ejemplo Completo: Control de Cámara

```cpp
#include "reactor/window.hpp"
#include "reactor/math.hpp"

class CameraController {
public:
    reactor::Camera camera;
    float moveSpeed = 5.0f;
    float rotateSpeed = 0.1f;
    
    void setupInput(reactor::Window& window) {
        // Movimiento con WASD
        window.setKeyCallback([this](int key, int action) {
            if (action == GLFW_PRESS || action == GLFW_REPEAT) {
                reactor::Vec3 forward = glm::normalize(camera.target - camera.position);
                reactor::Vec3 right = glm::normalize(glm::cross(forward, camera.up));
                
                switch (key) {
                    case GLFW_KEY_W:
                        camera.position += forward * moveSpeed * deltaTime;
                        break;
                    case GLFW_KEY_S:
                        camera.position -= forward * moveSpeed * deltaTime;
                        break;
                    case GLFW_KEY_A:
                        camera.position -= right * moveSpeed * deltaTime;
                        break;
                    case GLFW_KEY_D:
                        camera.position += right * moveSpeed * deltaTime;
                        break;
                    case GLFW_KEY_SPACE:
                        camera.position.y += moveSpeed * deltaTime;
                        break;
                    case GLFW_KEY_LEFT_SHIFT:
                        camera.position.y -= moveSpeed * deltaTime;
                        break;
                }
            }
        });
        
        // Rotación con mouse
        window.setMouseMoveCallback([this](double x, double y) {
            static double lastX = x;
            static double lastY = y;
            
            double deltaX = x - lastX;
            double deltaY = y - lastY;
            
            lastX = x;
            lastY = y;
            
            // Rotar cámara
            // ... implementar rotación
        });
    }
};
```

## 🔧 Configuración Avanzada

### Modo Fullscreen

```cpp
reactor::WindowConfig config;
config.fullscreen = true;

reactor::Window window(config);
```

### Cambiar Título Dinámicamente

```cpp
window.setTitle("FPS: " + std::to_string(fps));
```

### Redimensionar Ventana

```cpp
window.setSize(1920, 1080);
```

### Ocultar/Mostrar Cursor

```cpp
// En GLFW directo (acceso al handle)
glfwSetInputMode(window.handle(), GLFW_CURSOR, GLFW_CURSOR_DISABLED);
```

## 📊 Monitoreo de Performance

```cpp
// Calcular FPS
auto lastTime = std::chrono::high_resolution_clock::now();
int frameCount = 0;

while (!window.shouldClose()) {
    auto currentTime = std::chrono::high_resolution_clock::now();
    frameCount++;
    
    auto elapsed = std::chrono::duration<double>(currentTime - lastTime).count();
    if (elapsed >= 1.0) {
        double fps = frameCount / elapsed;
        window.setTitle("REACTOR - FPS: " + std::to_string(static_cast<int>(fps)));
        frameCount = 0;
        lastTime = currentTime;
    }
    
    window.pollEvents();
    render();
}
```

## 🐛 Troubleshooting

### Error: "Failed to initialize GLFW"

**Solución**: Asegúrate de llamar `reactor::Window::init()` antes de crear ventanas.

```cpp
reactor::Window::init();  // ← Importante
reactor::Window window(config);
```

### Error: "Failed to create window"

**Causas posibles**:
- Resolución inválida
- Monitor no disponible en fullscreen
- Drivers de GPU desactualizados

**Solución**:
```cpp
// Usar resolución segura
config.width = 1280;
config.height = 720;
config.fullscreen = false;
```

### Ventana no responde

**Solución**: Asegúrate de llamar `pollEvents()` en cada frame.

```cpp
while (!window.shouldClose()) {
    window.pollEvents();  // ← Necesario
    render();
}
```

## 📚 Recursos Adicionales

- **Documentación oficial**: https://www.glfw.org/documentation.html
- **Tutorial de GLFW**: https://www.glfw.org/docs/latest/quick.html
- **Ejemplos**: `examples/cube/main.cpp`, `examples/rendering/main.cpp`

## 🎯 Próximos Pasos

1. **Implementar control de cámara** - Usa WASD + mouse
2. **Agregar gamepad support** - GLFW soporta joysticks
3. **Implementar drag & drop** - Para cargar archivos
4. **Multi-window** - Múltiples ventanas simultáneas

---

**REACTOR + GLFW** = Sistema de ventanas potente y fácil de usar para Vulkan 🚀
