# 🎯 ISR Implementation Plan - v0.5.0

**Stack-GPU-OP - Intelligent Shading Rate**  
**Fecha de inicio**: 18 de Diciembre, 2025  
**Versión objetivo**: v0.5.0  
**Tiempo estimado**: 2-3 semanas

---

## 📋 Overview

Implementar el sistema ISR (Intelligent Shading Rate) completo de ADead-GPU en Vulkan usando compute shaders. El sistema ya tiene headers y shaders completos, falta la implementación en C++.

### Estado Actual

✅ **Completado**:
- Headers: `importance.hpp`, `adaptive.hpp`, `temporal.hpp`, `isr_system.hpp`
- Shaders: `importance.comp`, `adaptive.comp`, `temporal.comp`
- Builder API diseñado
- Estructura de clases definida

⏳ **Pendiente**:
- Implementaciones C++ (`.cpp` files)
- Descriptor sets y uniform buffers
- Pipeline integration
- Ejemplo demo

---

## 🏗️ Arquitectura ISR

### Pipeline de 3 Stages

```
Frame N Input
     ↓
┌─────────────────────────────────┐
│  Stage 1: Importance Compute    │
│  - Gradientes de luminancia     │
│  - Motion vectors                │
│  - Edge detection                │
│  Output: Importance Map          │
└─────────────────────────────────┘
     ↓
┌─────────────────────────────────┐
│  Stage 2: Adaptive Compute      │
│  - Pixel sizing basado en map   │
│  - Threshold application         │
│  - Resolution adaptation         │
│  Output: Pixel Size Map          │
└─────────────────────────────────┘
     ↓
┌─────────────────────────────────┐
│  Stage 3: Temporal Compute      │
│  - Coherencia temporal           │
│  - History buffer smoothing      │
│  - Flicker reduction             │
│  Output: Final Shading Rate      │
└─────────────────────────────────┘
     ↓
Rendering con Variable Rate Shading
```

---

## 📝 Tareas Detalladas

### 1. Importance Calculation (`importance.cpp`)

**Archivo**: `reactor/src/isr/importance.cpp`

#### Funciones a Implementar

```cpp
ImportanceCalculator::ImportanceCalculator(VulkanContext& ctx, uint32_t width, uint32_t height)
- Crear compute pipeline con importance.comp
- Crear descriptor set layout (input image, output buffer)
- Crear uniform buffer para parámetros
- Allocar importance map buffer (width * height * sizeof(float))

void ImportanceCalculator::calculate(VkCommandBuffer cmd, VkImageView inputImage)
- Bind compute pipeline
- Update descriptor set con input image
- Push constants (width, height, threshold)
- Dispatch compute shader (workgroups: width/16, height/16, 1)
- Memory barrier (compute → compute)

ImportanceMap ImportanceCalculator::getResult()
- Return importance map buffer handle
- Provide CPU readback si es necesario
```

#### Resources Necesarios
- **Compute Pipeline**: `importance.comp.spv`
- **Descriptor Set**: 
  - Binding 0: Input image (sampled)
  - Binding 1: Output buffer (storage)
- **Uniform Buffer**: Parámetros (threshold, etc.)
- **Storage Buffer**: Importance map (R32_SFLOAT)

---

### 2. Adaptive Pixel Sizing (`adaptive.cpp`)

**Archivo**: `reactor/src/isr/adaptive.cpp`

#### Funciones a Implementar

```cpp
AdaptivePixelSizer::AdaptivePixelSizer(VulkanContext& ctx, uint32_t width, uint32_t height)
- Crear compute pipeline con adaptive.comp
- Crear descriptor set layout (importance map, output buffer)
- Crear uniform buffer para thresholds
- Allocar pixel size map buffer

void AdaptivePixelSizer::compute(VkCommandBuffer cmd, const ImportanceMap& importanceMap)
- Bind compute pipeline
- Update descriptor set con importance map
- Push constants (min/max pixel size, thresholds)
- Dispatch compute shader
- Memory barrier

PixelSizeMap AdaptivePixelSizer::getResult()
- Return pixel size map buffer
```

#### Resources Necesarios
- **Compute Pipeline**: `adaptive.comp.spv`
- **Descriptor Set**:
  - Binding 0: Importance map (storage buffer)
  - Binding 1: Output pixel size map (storage buffer)
- **Uniform Buffer**: Thresholds (high, medium, low)
- **Storage Buffer**: Pixel size map (R32_UINT)

---

### 3. Temporal Coherence (`temporal.cpp`)

**Archivo**: `reactor/src/isr/temporal.cpp`

#### Funciones a Implementar

```cpp
TemporalCoherence::TemporalCoherence(VulkanContext& ctx, uint32_t width, uint32_t height)
- Crear compute pipeline con temporal.comp
- Crear descriptor set layout (current, history, output)
- Allocar history buffer (double buffered)
- Crear uniform buffer para blend factor

void TemporalCoherence::apply(VkCommandBuffer cmd, const PixelSizeMap& currentMap)
- Bind compute pipeline
- Update descriptor set (current, history)
- Push constants (blend factor, frame index)
- Dispatch compute shader
- Memory barrier
- Swap history buffers

ShadingRateMap TemporalCoherence::getResult()
- Return final shading rate map
```

#### Resources Necesarios
- **Compute Pipeline**: `temporal.comp.spv`
- **Descriptor Set**:
  - Binding 0: Current pixel size map
  - Binding 1: History buffer (read)
  - Binding 2: Output shading rate map (write)
- **Uniform Buffer**: Blend factor
- **Storage Buffers**: History (double buffered)

---

### 4. ISR System Integration (`isr_system.cpp`)

**Archivo**: `reactor/src/isr/isr_system.cpp`

#### Builder Implementation

```cpp
ISRSystem::Builder ISRSystem::create(VulkanContext& ctx)
- Return builder instance

Builder& Builder::resolution(uint32_t width, uint32_t height)
- Store resolution
- Return *this

Builder& Builder::thresholds(float high, float medium, float low)
- Store thresholds
- Return *this

Builder& Builder::temporalBlend(float factor)
- Store blend factor
- Return *this

std::unique_ptr<ISRSystem> Builder::build()
- Create ImportanceCalculator
- Create AdaptivePixelSizer
- Create TemporalCoherence
- Setup descriptor pools
- Return ISRSystem instance
```

#### System Methods

```cpp
void ISRSystem::process(VkCommandBuffer cmd, VkImageView inputImage)
- Stage 1: importance.calculate(cmd, inputImage)
- Barrier
- Stage 2: adaptive.compute(cmd, importance.getResult())
- Barrier
- Stage 3: temporal.apply(cmd, adaptive.getResult())
- Barrier

ShadingRateMap ISRSystem::getShadingRateMap()
- Return temporal.getResult()

void ISRSystem::updateParameters(const ISRParameters& params)
- Update uniform buffers
- Thresholds, blend factor, etc.
```

---

## 🔧 Vulkan Resources Checklist

### Compute Pipelines
- [ ] `importance.comp.spv` pipeline
- [ ] `adaptive.comp.spv` pipeline
- [ ] `temporal.comp.spv` pipeline

### Descriptor Sets
- [ ] Importance descriptor set layout
- [ ] Adaptive descriptor set layout
- [ ] Temporal descriptor set layout
- [ ] Descriptor pool (suficiente para 3 sets)

### Buffers
- [ ] Importance map buffer (storage)
- [ ] Pixel size map buffer (storage)
- [ ] History buffer A (storage)
- [ ] History buffer B (storage)
- [ ] Shading rate output buffer (storage)
- [ ] Uniform buffer para parámetros

### Memory Barriers
- [ ] Compute → Compute barriers entre stages
- [ ] Compute → Graphics barrier antes de rendering

---

## 📊 Example Application

### Archivo: `examples/stack-gpu-isr/main.cpp`

```cpp
int main() {
    // Setup Vulkan context
    reactor::VulkanContext ctx = ...;
    
    // Create ISR system
    auto isr = reactor::ISRSystem::create(ctx)
        .resolution(1920, 1080)
        .thresholds(0.8f, 0.5f, 0.2f)
        .temporalBlend(0.9f)
        .build();
    
    // Render loop
    while (!window.shouldClose()) {
        // Render scene to offscreen image
        renderScene(cmd, offscreenImage);
        
        // Process ISR
        isr->process(cmd, offscreenImage);
        
        // Get shading rate map
        auto shadingRateMap = isr->getShadingRateMap();
        
        // Use for variable rate shading
        applyVariableRateShading(cmd, shadingRateMap);
        
        // Final render
        finalRender(cmd);
    }
}
```

### Features del Ejemplo
- Visualización de importance map (debug view)
- Visualización de pixel size map (color coded)
- Toggle ISR on/off para comparación
- Performance metrics (FPS con/sin ISR)
- UI para ajustar thresholds en tiempo real

---

## 🎯 Milestones

### Week 1: Core Implementation
- [ ] Día 1-2: `importance.cpp` completo
- [ ] Día 3-4: `adaptive.cpp` completo
- [ ] Día 5-7: `temporal.cpp` completo

### Week 2: System Integration
- [ ] Día 8-10: `isr_system.cpp` completo
- [ ] Día 11-12: Descriptor sets y buffers
- [ ] Día 13-14: Memory barriers y synchronization

### Week 3: Example & Polish
- [ ] Día 15-17: Ejemplo `stack-gpu-isr`
- [ ] Día 18-19: Debug visualization
- [ ] Día 20-21: Performance testing y optimization

---

## 📈 Success Criteria

### Funcionalidad
- ✅ ISR system procesa frames correctamente
- ✅ Importance map detecta áreas de alta/baja importancia
- ✅ Adaptive sizing ajusta pixel sizes apropiadamente
- ✅ Temporal coherence reduce flickering
- ✅ Shading rate map es válido para VRS

### Performance
- ✅ ISR overhead < 2ms por frame (1080p)
- ✅ Memory usage razonable (~10-20 MB)
- ✅ No stuttering ni frame drops

### Calidad Visual
- ✅ Sin artefactos visuales notables
- ✅ Transiciones suaves entre shading rates
- ✅ Coherencia temporal estable

---

## 🔍 Testing Strategy

### Unit Tests
- Importance calculation con imágenes sintéticas
- Adaptive sizing con importance maps conocidos
- Temporal coherence con secuencias de frames

### Integration Tests
- Pipeline completo con cubo 3D
- Performance profiling
- Visual comparison con/sin ISR

### Stress Tests
- Resoluciones altas (4K)
- Escenas complejas
- Rapid motion

---

## 📚 Referencias

### ADead-GPU Original
- ISR paper/documentation
- DirectX 12 implementation
- Performance benchmarks

### Vulkan Resources
- Compute shader best practices
- Memory barrier guidelines
- Variable Rate Shading extension

### Stack-GPU-OP Docs
- `META/STACK_GPU_OP_VISION.md` - ISR overview
- `reactor/include/reactor/isr/` - Headers
- `shaders/isr/` - Compute shaders

---

## 🚀 Next Steps After v0.5.0

Una vez completado ISR:
1. Integrar con cubo 3D (v0.5.1)
2. Optimizar performance (v0.5.2)
3. Avanzar a SDF Ray Marching (v0.6.0)

---

<div align="center">

**ISR Implementation Plan**

*De Headers a Sistema Completo*

*Stack-GPU-OP v0.5.0*

**¡Vamos a implementarlo!** 🚀

</div>
