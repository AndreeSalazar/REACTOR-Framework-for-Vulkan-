# Scripts de Test_Game

## 📜 Scripts Disponibles

### `quick-start.bat` ⭐ (Recomendado)
**Uso:** Ejecuta desde cualquier ubicación
```batch
Test_Game\quick-start.bat
```

**Qué hace:**
1. Cambia automáticamente al directorio correcto
2. Compila los shaders (cube.vert, cube.frag)
3. Configura y compila el proyecto con CMake
4. Ejecuta test-game.exe

**Cuándo usar:** Primera vez o cuando hagas cambios en código/shaders

---

### `compile-shaders.bat`
**Uso:** Desde cualquier ubicación
```batch
Test_Game\compile-shaders.bat
```

**Qué hace:**
- Compila `cube.vert` → `cube.vert.spv`
- Compila `cube.frag` → `cube.frag.spv`

**Requisito:** Vulkan SDK instalado (incluye `glslc`)

**Cuándo usar:** Solo cuando modifiques los shaders

---

### `build.bat`
**Uso:** Desde cualquier ubicación
```batch
Test_Game\build.bat
```

**Qué hace:**
1. Crea carpeta `build/` si no existe
2. Ejecuta CMake para generar proyecto Visual Studio
3. Compila el proyecto en modo Debug

**Cuándo usar:** Cuando modifiques `main.cpp` o `CMakeLists.txt`

---

### `run.bat`
**Uso:** Desde cualquier ubicación
```batch
Test_Game\run.bat
```

**Qué hace:**
- Ejecuta `build\Debug\test-game.exe`
- Verifica que el ejecutable exista

**Cuándo usar:** Solo para ejecutar (sin compilar)

---

## 🔧 Características de los Scripts

### ✅ Funcionan desde cualquier directorio
Todos los scripts usan `cd /d "%~dp0"` para cambiar al directorio correcto automáticamente.

### ✅ Validación de errores
Cada paso verifica si tuvo éxito antes de continuar.

### ✅ Mensajes informativos
Muestran el directorio actual y el progreso de cada operación.

### ✅ Manejo de rutas absolutas
Usan `%~dp0` para obtener la ruta del script, no importa desde dónde se ejecuten.

---

## 🎯 Flujo de Trabajo Recomendado

### Primera vez:
```batch
Test_Game\quick-start.bat
```

### Modificaste shaders:
```batch
Test_Game\compile-shaders.bat
Test_Game\run.bat
```

### Modificaste código C++:
```batch
Test_Game\build.bat
Test_Game\run.bat
```

### Solo quieres ejecutar:
```batch
Test_Game\run.bat
```

---

## 🐛 Solución de Problemas

### Error: "glslc no se reconoce"
**Causa:** Vulkan SDK no instalado o no en PATH

**Solución:**
1. Instala Vulkan SDK desde https://vulkan.lunarg.com/
2. Reinicia el terminal
3. Verifica: `glslc --version`

### Error: "cmake no se reconoce"
**Causa:** CMake no instalado o no en PATH

**Solución:**
1. Instala CMake desde https://cmake.org/download/
2. Durante instalación, marca "Add CMake to PATH"
3. Reinicia el terminal

### Error: "No se encontró Visual Studio"
**Causa:** Visual Studio 2022 no instalado

**Solución:**
1. Instala Visual Studio 2022 Community
2. Incluye "Desktop development with C++"
3. O cambia el generador en `build.bat`:
   ```batch
   cmake .. -G "Visual Studio 16 2019" -A x64
   ```

### Error: "test-game.exe no encontrado"
**Causa:** No se ha compilado el proyecto

**Solución:**
```batch
Test_Game\build.bat
```

---

## 📝 Notas Técnicas

### Variables de entorno usadas:
- `%~dp0` - Directorio donde está el script
- `%CD%` - Directorio actual de trabajo
- `%ERRORLEVEL%` - Código de salida del último comando

### Comandos clave:
- `setlocal` - Crea scope local para variables
- `cd /d "%~dp0"` - Cambia al directorio del script
- `call` - Ejecuta otro batch y espera su finalización
- `if %ERRORLEVEL% NEQ 0` - Verifica si hubo error

---

## 🚀 Integración con IDE

Puedes ejecutar estos scripts directamente desde:

- **VS Code:** Terminal integrado
- **Visual Studio:** Developer Command Prompt
- **Explorador de Windows:** Doble clic en el .bat
- **Cualquier terminal:** PowerShell, CMD, Git Bash

Todos funcionarán correctamente sin importar el directorio actual.
