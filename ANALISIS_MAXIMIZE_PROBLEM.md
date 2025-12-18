# Análisis Profundo: Problema de Maximización

## 🔍 Problema Identificado

La aplicación se marca como "(No responde)" cuando se intenta maximizar la ventana.

## 🎯 Causa Raíz

**`vkDeviceWaitIdle` bloquea el hilo principal durante demasiado tiempo**

### Análisis Detallado:

1. **Ubicación del problema:**
   - Línea 576 en `main.cpp`: `vkDeviceWaitIdle(ctx.device())` se llama durante el resize
   - Esta función espera a que TODAS las operaciones GPU terminen
   - Durante este tiempo (puede ser 16-33ms o más), el hilo principal está bloqueado
   - Windows no recibe mensajes de ventana, causando "No responde"

2. **Flujo problemático actual:**
   ```
   Detecta resize → Espera fences (100ms timeout) → vkDeviceWaitIdle (BLOQUEO) → Recrea swapchain
   ```

3. **Problemas adicionales:**
   - No se usa `oldSwapchain` al recrear (ineficiente)
   - Espera redundante: primero fences, luego `vkDeviceWaitIdle`
   - No hay forma de procesar eventos durante `vkDeviceWaitIdle`

## ✅ Solución Propuesta

1. **Eliminar `vkDeviceWaitIdle` completamente durante resize**
2. **Usar solo fences con timeouts cortos y procesar eventos**
3. **Usar `oldSwapchain` para mejor rendimiento**
4. **Procesar eventos entre esperas de fences**

## 🔧 Implementación

### Cambios necesarios:

1. Modificar `Swapchain` para aceptar `oldSwapchain` en el constructor
2. Reemplazar `vkDeviceWaitIdle` con espera de fences con timeout
3. Procesar eventos durante las esperas
4. Simplificar la lógica de sincronización

