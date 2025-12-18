# 🚀 Cómo Ejecutar Stack-GPU-OP (Independiente)

## Guía Rápida - Sin Depender de Nadie

### ✅ Ejecutar el Cubo 3D (Ya Compilado)

Si ya compilaste el proyecto, simplemente:

```bash
# Opción 1: Desde la raíz del proyecto
cd build\examples\stack-gpu-cube\Release
.\stack-gpu-cube.exe

# Opción 2: Ruta completa
C:\Users\andre\OneDrive\Documentos\REACTOR (Framework for Vulkan)\build\examples\stack-gpu-cube\Release\stack-gpu-cube.exe
```

### 🔧 Compilar y Ejecutar

Si hiciste cambios en el código:

```bash
# 1. Compilar solo el cubo (rápido - 10 segundos)
cmake --build build --config Release --target stack-gpu-cube

# 2. Ejecutar
cd build\examples\stack-gpu-cube\Release
.\stack-gpu-cube.exe
```

### 🆕 Compilar Desde Cero (Primera Vez)

Si es la primera vez o borraste la carpeta `build`:

```bash
# Opción A: Todo automático (recomendado)
quick-setup.bat

# Opción B: Paso a paso
configure.bat          # Configurar CMake
build.bat              # Compilar todo
cd build\examples\stack-gpu-cube\Release
.\stack-gpu-cube.exe   # Ejecutar
```

---

## 🎮 Controles del Cubo

Una vez ejecutando:

- **Tecla 1**: Modo Normal (Phong Shading)
- **Tecla 2**: Modo Wireframe
- **Tecla 3**: Modo Normales RGB
- **Tecla 4**: Modo Depth Buffer
- **Tecla 5**: Modo ISR Importance Map
- **Tecla 6**: Modo ISR Pixel Sizing
- **Tecla 7**: Modo ISR Temporal
- **ESC**: Salir

---

## 📁 Ubicaciones Importantes

### Ejecutables
```
build\examples\stack-gpu-cube\Release\stack-gpu-cube.exe    ← Cubo 3D
build\examples\triangle\Release\reactor-triangle.exe        ← Triángulo básico
build\examples\sandbox\Release\reactor-sandbox.exe          ← Sandbox
```

### Código Fuente
```
examples\stack-gpu-cube\main.cpp              ← Código principal del cubo
examples\stack-gpu-cube\cube_renderer.cpp     ← Renderer del cubo
shaders\cube\cube_debug.frag                  ← Fragment shader con 7 modos
shaders\cube\cube.vert                        ← Vertex shader
```

### Shaders Compilados
```
build\examples\stack-gpu-cube\Release\shaders\cube.vert.spv    ← Vertex shader SPIR-V
build\examples\stack-gpu-cube\Release\shaders\cube.frag.spv    ← Fragment shader SPIR-V
```

---

## 🔍 Solución de Problemas

### ❌ "No se encuentra el archivo .exe"

**Problema**: El ejecutable no existe.

**Solución**:
```bash
# Compilar primero
cmake --build build --config Release --target stack-gpu-cube

# Luego ejecutar
cd build\examples\stack-gpu-cube\Release
.\stack-gpu-cube.exe
```

### ❌ "Failed to load shader"

**Problema**: Los shaders no están compilados.

**Solución**:
```bash
# Recompilar todo (incluye shaders)
cmake --build build --config Release --target stack-gpu-cube
```

### ❌ "Vulkan initialization failed"

**Problema**: Vulkan SDK no encontrado.

**Solución**:
1. Verificar que Vulkan SDK esté instalado: `C:\VulkanSDK\1.4.328.1`
2. Reconfigurar: `configure.bat`
3. Compilar: `build.bat`

### ❌ Ventana negra o no se ve nada

**Problema**: Shaders no cargados correctamente.

**Solución**:
```bash
# Limpiar y recompilar
cmake --build build --config Release --target clean
cmake --build build --config Release --target stack-gpu-cube
```

---

## 📊 Verificar que Todo Funciona

Cuando ejecutes `stack-gpu-cube.exe`, deberías ver:

```
==========================================
  Stack-GPU-OP: Debug Visualizer
  Vulkan + ADead-GPU ISR
==========================================

[✓] Ventana creada (1920x1080 maximizada)
[✓] Vulkan inicializado
[✓] Swapchain creado
[✓] Depth buffer creado
[✓] Render pass creado (con depth)
[✓] Cube renderer creado
[✓] Sincronización configurada

==========================================
  Stack-GPU-OP Debug Visualizer Listo!
==========================================

CONTROLES:
  [1] Normal - Phong Shading
  [2] Wireframe
  ...
  
FPS: 75 | Modo: Normal
```

Y verás un **cubo 3D rotando** con iluminación Phong en una ventana maximizada de 1920x1080.

---

## 🚀 Comandos Útiles

```bash
# Compilar solo el cubo (rápido)
cmake --build build --config Release --target stack-gpu-cube

# Compilar todo el proyecto
cmake --build build --config Release

# Limpiar compilación
cmake --build build --config Release --target clean

# Reconfigurar CMake
cmake -B build -S . -DCMAKE_BUILD_TYPE=Release

# Ver todos los targets disponibles
cmake --build build --target help
```

---

## 📝 Notas

- **Primera compilación**: ~30 segundos
- **Recompilaciones**: ~10 segundos
- **Tamaño ejecutable**: ~200 KB
- **FPS esperado**: 74-75 FPS
- **Resolución**: 1920x1080 (maximizada)

---

¡Listo! Ahora puedes ejecutar el cubo 3D completamente independiente. 🎉
