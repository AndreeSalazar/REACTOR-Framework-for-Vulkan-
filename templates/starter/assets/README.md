# Assets Directory

Coloca aquí tus recursos:

## 📁 Estructura Recomendada

```
assets/
├── shaders/          # Shaders GLSL
│   ├── shader.vert
│   └── shader.frag
├── textures/         # Texturas e imágenes
│   ├── diffuse.png
│   └── normal.png
└── models/           # Modelos 3D
    └── model.gltf
```

## 🎨 Shaders

Crea tus shaders en GLSL:

**shader.vert**:
```glsl
#version 450

layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec3 inColor;

layout(location = 0) out vec3 fragColor;

void main() {
    gl_Position = vec4(inPosition, 0.0, 1.0);
    fragColor = inColor;
}
```

**shader.frag**:
```glsl
#version 450

layout(location = 0) in vec3 fragColor;
layout(location = 0) out vec4 outColor;

void main() {
    outColor = vec4(fragColor, 1.0);
}
```

## 🖼️ Texturas

Formatos soportados:
- PNG
- JPG
- TGA
- BMP

## 🎮 Modelos

Formatos soportados:
- GLTF/GLB
- OBJ
- FBX (próximamente)

## 🔧 Compilar Shaders

```bash
# Compilar manualmente con glslc
glslc shader.vert -o shader.vert.spv
glslc shader.frag -o shader.frag.spv

# O usar el sistema de build de REACTOR (próximamente)
reactor compile-shaders
```

## 📦 Assets en Build

Los assets se copian automáticamente al directorio de build cuando compilas el proyecto.
