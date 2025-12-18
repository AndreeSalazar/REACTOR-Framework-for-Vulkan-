# 🤝 Contributing to Stack-GPU-OP

Gracias por tu interés en contribuir a Stack-GPU-OP. Este documento te guiará en el proceso.

---

## 🎯 Filosofía del Proyecto

### Principios Fundamentales

1. **100% Vulkan Puro**
   - NO mezclar DirectX 12
   - NO usar wrappers de DirectX
   - Implementación nativa en Vulkan

2. **Fidelidad a ADead-GPU**
   - Mantener las ideas originales de ADead-GPU
   - Adaptar idiomáticamente a Vulkan
   - Mejorar donde sea posible

3. **React-Style API**
   - Builder pattern fluido
   - Componentes declarativos
   - RAII automático

4. **Cross-Platform First**
   - Código que funcione en Windows, Linux, macOS
   - Sin #ifdef platform-specific innecesarios
   - Usar abstracciones de Vulkan

---

## 🚀 Cómo Empezar

### 1. Setup del Entorno

```bash
# Clonar repositorio
git clone https://github.com/TU_USUARIO/stack-gpu-op.git
cd stack-gpu-op

# Instalar dependencias
.\install-dependencies.bat  # Windows
# o
./install-dependencies.sh   # Linux/macOS

# Compilar
.\quick-setup.bat  # Windows
# o
./quick-setup.sh   # Linux/macOS
```

### 2. Familiarizarse con el Código

Lee en orden:
1. `META/STACK_GPU_OP_VISION.md` - Visión del proyecto
2. `META/META.md` - Overview completo
3. `STACK-GPU-OP.md` - Arquitectura técnica
4. `USAGE_GUIDE.md` - Cómo usar la API

### 3. Ejecutar Ejemplos

```bash
# Cubo 3D
cd build\examples\stack-gpu-cube\Release
.\stack-gpu-cube.exe

# Otros ejemplos
cd build\examples\triangle\Release
.\reactor-triangle.exe
```

---

## 📝 Proceso de Contribución

### 1. Crear Issue

Antes de empezar a codear:
- Busca si ya existe un issue similar
- Crea un nuevo issue describiendo:
  - Qué quieres implementar/arreglar
  - Por qué es necesario
  - Cómo planeas hacerlo

### 2. Fork y Branch

```bash
# Fork el repositorio en GitHub

# Clonar tu fork
git clone https://github.com/TU_USUARIO/stack-gpu-op.git
cd stack-gpu-op

# Crear branch
git checkout -b feature/nombre-descriptivo
# o
git checkout -b bugfix/nombre-descriptivo
```

### 3. Hacer Cambios

- Sigue el estilo de código existente
- Agrega tests si es posible
- Actualiza documentación
- Commits pequeños y descriptivos

### 4. Testing

```bash
# Compilar
cmake --build build --config Release

# Ejecutar ejemplos
cd build\examples\stack-gpu-cube\Release
.\stack-gpu-cube.exe

# Verificar que todo funciona
```

### 5. Commit

```bash
git add .
git commit -m "✨ feat: descripción breve del cambio"
```

Ver [Conventional Commits](#conventional-commits) para formato.

### 6. Push y Pull Request

```bash
git push origin feature/nombre-descriptivo
```

En GitHub:
- Crear Pull Request
- Describir cambios
- Referenciar issue relacionado
- Esperar review

---

## 📐 Estilo de Código

### C++

```cpp
// Namespace: snake_case
namespace reactor::sdf {

// Classes: PascalCase
class RayMarcher {
public:
    // Methods: camelCase
    void render(CommandBuffer& cmd);
    
    // Builder: PascalCase
    class Builder {
    public:
        Builder& resolution(uint32_t w, uint32_t h);
        RayMarcher build();
    };
    
private:
    // Members: camelCase con prefijo
    VkDevice device;
    Config config;
};

// Enums: PascalCase
enum class ShaderStage {
    Vertex,
    Fragment,
    Compute
};

// Constants: UPPER_SNAKE_CASE
constexpr uint32_t MAX_FRAMES_IN_FLIGHT = 2;

} // namespace reactor::sdf
```

### GLSL

```glsl
// Constants: UPPER_SNAKE_CASE
#define MAX_STEPS 128
#define EPSILON 0.001

// Uniforms: camelCase
layout(binding = 0) uniform Config {
    mat4 viewMatrix;
    mat4 projMatrix;
    vec3 cameraPos;
} config;

// Functions: camelCase
float sceneSDF(vec3 p) {
    // ...
}

vec3 calcNormal(vec3 p) {
    // ...
}
```

### Convenciones

- **Indentación**: 4 espacios (NO tabs)
- **Líneas**: Max 100 caracteres
- **Headers**: Include guards con `#pragma once`
- **Includes**: Ordenados alfabéticamente
- **Comentarios**: Inglés o español, pero consistente

---

## 🧪 Testing

### Unit Tests (Futuro)

```cpp
TEST(BufferTest, CreateAndUpload) {
    auto buffer = Buffer::create(allocator)
        .size(1024)
        .usage(BufferUsage::Vertex)
        .build();
    
    std::vector<float> data(256);
    buffer.upload(data.data(), sizeof(data));
    
    EXPECT_EQ(buffer.size(), 1024);
}
```

### Integration Tests

- Ejecutar todos los ejemplos
- Verificar que compilan sin warnings
- Verificar que renderizan correctamente
- Verificar FPS razonable (>60)

---

## 📝 Conventional Commits

Formato: `<tipo>: <descripción>`

### Tipos

- `✨ feat:` - Nueva característica
- `🐛 fix:` - Corrección de bug
- `📚 docs:` - Cambios en documentación
- `🎨 style:` - Formato, sin cambios de código
- `♻️ refactor:` - Refactorización
- `⚡ perf:` - Mejora de performance
- `✅ test:` - Agregar/modificar tests
- `🔧 chore:` - Mantenimiento, build, etc.

### Ejemplos

```bash
git commit -m "✨ feat: implement ISR importance calculator"
git commit -m "🐛 fix: shader loading path on Linux"
git commit -m "📚 docs: update ROADMAP with ISR progress"
git commit -m "⚡ perf: optimize SDF ray marching loop"
```

---

## 🎯 Áreas de Contribución

### 1. ISR Implementation (Alta Prioridad)

**Estado**: Headers y shaders completos, implementación pendiente

**Tareas**:
- Implementar `importance.cpp`
- Implementar `adaptive.cpp`
- Implementar `temporal.cpp`
- Crear uniforms y descriptors
- Integrar con pipeline
- Ejemplo funcional

**Skills**: Vulkan, Compute Shaders, Image Processing

### 2. SDF Ray Marching (Media Prioridad)

**Estado**: Básico implementado, falta avanzado

**Tareas**:
- Pipeline completo de ray marching
- Múltiples primitivas en escena
- Iluminación avanzada (Phong, PBR)
- Sombras y ambient occlusion
- Ejemplo complejo

**Skills**: Vulkan, Fragment Shaders, SDF Mathematics

### 3. Advanced Ray Tracing (Baja Prioridad)

**Estado**: No iniciado

**Tareas**:
- Sphere tracing optimizado
- Cone tracing (soft shadows)
- Beam tracing (reflections)
- Hierarchical SDF
- Global Illumination

**Skills**: Vulkan, Compute Shaders, Ray Tracing Theory

### 4. GPU Language (Baja Prioridad)

**Estado**: No iniciado

**Tareas**:
- Lexer y Parser
- AST construction
- IR design
- IR → SPIR-V compiler
- Validation

**Skills**: Compiler Design, SPIR-V, Language Theory

### 5. Documentación (Siempre Bienvenida)

**Tareas**:
- Mejorar guías existentes
- Agregar tutoriales
- Traducir a otros idiomas
- Ejemplos de código
- Videos/GIFs de demos

**Skills**: Escritura técnica, Markdown

### 6. Testing (Siempre Bienvenida)

**Tareas**:
- Unit tests
- Integration tests
- Performance benchmarks
- Cross-platform testing
- CI/CD pipeline

**Skills**: Testing, CMake, GitHub Actions

---

## 🐛 Reportar Bugs

### Template de Issue

```markdown
**Descripción**
Descripción clara del bug.

**Pasos para Reproducir**
1. Compilar con...
2. Ejecutar...
3. Ver error...

**Comportamiento Esperado**
Qué debería pasar.

**Comportamiento Actual**
Qué pasa realmente.

**Screenshots**
Si aplica.

**Entorno**
- OS: Windows 10
- Vulkan SDK: 1.3.280
- GPU: NVIDIA RTX 3060
- Driver: 536.23

**Logs**
```
Pegar logs aquí
```

**Información Adicional**
Cualquier otro contexto.
```

---

## 💡 Proponer Features

### Template de Issue

```markdown
**Feature Request**
Descripción clara de la feature.

**Problema que Resuelve**
Por qué es necesaria.

**Solución Propuesta**
Cómo implementarla.

**Alternativas Consideradas**
Otras opciones.

**Información Adicional**
Contexto, ejemplos, referencias.
```

---

## 📚 Recursos Útiles

### Vulkan
- [Vulkan Tutorial](https://vulkan-tutorial.com/)
- [Vulkan Guide](https://github.com/KhronosGroup/Vulkan-Guide)
- [Vulkan Spec](https://www.khronos.org/registry/vulkan/)

### SDF
- [Inigo Quilez - SDF](https://iquilezles.org/articles/distfunctions/)
- [Shadertoy](https://www.shadertoy.com/)

### Ray Tracing
- [Ray Tracing in One Weekend](https://raytracing.github.io/)
- [Scratchapixel](https://www.scratchapixel.com/)

### ADead-GPU
- Repositorio original: `C:\Users\andre\OneDrive\Documentos\ADead-GPU`
- README.md del proyecto original

---

## 🎉 Reconocimientos

Todos los contribuidores serán listados en:
- README.md
- CHANGELOG.md
- GitHub Contributors

---

## 📄 Licencia

Al contribuir, aceptas que tus contribuciones serán licenciadas bajo la MIT License.

---

<div align="center">

**¡Gracias por contribuir a Stack-GPU-OP!**

*Juntos estamos construyendo el framework GPU del futuro*

</div>
