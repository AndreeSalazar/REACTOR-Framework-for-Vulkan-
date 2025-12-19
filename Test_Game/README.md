# Test Game - Standalone Demo

Este es un proyecto de prueba **independiente** que demuestra las capacidades básicas de GLFW y Vulkan sin depender de la librería REACTOR.

## 🎯 Objetivo

Verificar que el entorno de desarrollo está correctamente configurado y probar funcionalidades básicas:

- ✅ Sistema de ventanas (GLFW)
- ✅ Vulkan SDK disponible
- ✅ Datos de geometría (cubo 3D)
- ✅ Sistema de input
- ✅ Loop de renderizado (sin rendering real)

## 🏗️ Estructura

```
Test_Game/
├── main.cpp              # Código principal del juego
├── CMakeLists.txt        # Configuración de build
├── shaders/              # Shaders GLSL
│   ├── cube.vert        # Vertex shader
│   └── cube.frag        # Fragment shader
├── build.bat            # Script de compilación
├── compile-shaders.bat  # Script para compilar shaders
└── README.md            # Este archivo
```

## 🚀 Cómo Compilar

### Opción 1: Quick Start (Recomendado)

Ejecuta desde la raíz del proyecto o desde Test_Game:

```batch
# Desde la raíz del proyecto REACTOR
Test_Game\quick-start.bat

# O desde Test_Game
cd Test_Game
quick-start.bat
```

### Opción 2: Paso a Paso

```batch
cd Test_Game
compile-shaders.bat
build.bat
run.bat
```

### Opción 3: Solo ejecutar (si ya compilaste)

```batch
cd Test_Game
run.bat
```

**Nota Importante:** 
- Este es un proyecto **standalone** que NO requiere compilar la librería REACTOR
- El ejecutable se genera en: `Test_Game\build\Debug\test-game.exe`
- Los shaders están incluidos pero no se usan en esta versión simplificada
- GLFW se descarga automáticamente si no está instalado

## 🎮 Controles

- **ESC** - Salir de la aplicación
- **SPACE** - Pausar/Reanudar rotación del cubo
- **FLECHA ↑** - Aumentar velocidad de rotación
- **FLECHA ↓** - Disminuir velocidad de rotación

## 📋 Características Demostradas

### 1. Window Management
- Creación de ventana con GLFW
- Configuración de tamaño y título
- Event polling
- Ventana funcional

### 2. Vulkan Verification
- Verificación de Vulkan SDK instalado
- Enumeración de extensiones disponibles
- Confirmación de que el entorno está listo

### 3. Data Structures
- Estructura de vértices del cubo
- Datos preparados para rendering futuro
- 8 vértices con posición y color

### 4. Input System
- Keyboard callbacks funcionales
- Control interactivo en tiempo real
- Estado de rotación y velocidad
- Respuesta inmediata a teclas

### 5. Render Loop
- Loop principal funcional
- Cálculo de FPS
- Actualización de estado
- Simulación de rotación

## 🔧 Dependencias

Este proyecto standalone requiere:

- **Vulkan SDK** - Debe estar instalado en el sistema
- **GLFW** - Se descarga automáticamente con CMake FetchContent
- **CMake 3.15+** - Para configurar el proyecto
- **Visual Studio 2022** - Compilador C++

Las dependencias se gestionan automáticamente excepto Vulkan SDK.

## 📝 Notas

- Este proyecto es **completamente independiente** de REACTOR
- **NO requiere** compilar la librería REACTOR
- Perfecto para verificar que el entorno de desarrollo funciona
- Esta es una versión simplificada - no renderiza el cubo visualmente
- Los shaders están incluidos pero no se usan en esta versión

## 🎓 Próximos Pasos

Este es un punto de partida para verificar el entorno. Próximos pasos:

1. **Integrar con REACTOR** - Una vez que REACTOR compile sin errores
2. **Agregar rendering real** - Usar Vulkan para dibujar el cubo
3. **Implementar pipeline gráfico** - Shaders, buffers, comandos
4. **Agregar texturas** - Sistema de texturas
5. **Implementar cámara 3D** - Transformaciones MVP
6. **Agregar más geometría** - Múltiples objetos
7. **Implementar física** - Colisiones y movimiento

**Nota:** Actualmente REACTOR tiene errores de compilación en los módulos SDF. Una vez resueltos, este proyecto puede migrar a usar REACTOR completamente.

## 🐛 Troubleshooting

### La ventana aparece negra
- Asegúrate de compilar los shaders con `compile-shaders.bat`
- Verifica que los archivos `.spv` existan en `shaders/`

### Error al compilar
- Verifica que el proyecto principal REACTOR esté compilado
- Asegúrate de tener Vulkan SDK instalado
- Revisa que todas las dependencias estén en `build/`

### Error de Vulkan
- Verifica que tu GPU soporte Vulkan
- Actualiza los drivers de tu tarjeta gráfica
- Revisa que Vulkan SDK esté correctamente instalado

### Error: "test-game.exe no encontrado"
**Causa:** No se ha compilado el proyecto

**Solución:**
```batch
Test_Game\build.bat
```

**Ubicación del ejecutable:** `build\Test_Game\Debug\test-game.exe`

### Error: "No se puede abrir el archivo incluir: 'reactor/reactor.hpp'"
**Causa:** Estás intentando usar una versión antigua que dependía de REACTOR

**Solución:**
- La versión actual es **standalone** y NO requiere REACTOR
- Asegúrate de tener la última versión de `main.cpp`
- Recompila con `build.bat`

### FPS muy alto (millones)
**Causa:** No hay rendering real, solo cálculos

**Solución:**
- Esto es normal en esta versión simplificada
- El FPS será realista cuando se implemente rendering con Vulkan

## 📚 Referencias

- [REACTOR Documentation](../META/DOCUMENTATION_INDEX.md)
- [Vulkan Tutorial](https://vulkan-tutorial.com/)
- [GLFW Documentation](https://www.glfw.org/documentation.html)
- [GLM Documentation](https://github.com/g-truc/glm)
