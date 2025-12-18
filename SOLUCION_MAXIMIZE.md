# Solución Final para Problema de Maximización

## Problema Identificado
La aplicación se marca como "(No responde)" al maximizar porque el hilo principal se bloquea durante la recreación del swapchain.

## Solución Implementada

### Cambios Realizados:

1. **Reemplazo de `vkDeviceWaitIdle` por `vkQueueWaitIdle`**
   - `vkQueueWaitIdle` solo espera la cola de gráficos, no todo el dispositivo
   - Es más rápido y menos bloqueante que `vkDeviceWaitIdle`
   - Se usa en todos los lugares donde se recrea el swapchain

2. **Procesamiento de Eventos**
   - Se llama a `window.pollEvents()` ANTES y DESPUÉS de `vkQueueWaitIdle`
   - Esto mantiene la aplicación responsiva durante la espera

3. **Uso de `oldSwapchain`**
   - El constructor de `Swapchain` ahora acepta un parámetro `oldSwapchain`
   - Permite al driver optimizar la recreación del swapchain

4. **Limpieza Simplificada**
   - Se eliminaron las esperas complejas de fences con loops
   - La lógica es más simple y directa

## Ubicaciones de los Cambios

- **Durante resize detectado**: Línea ~552
- **En errores de acquire**: Línea ~659
- **En errores de present**: Línea ~829
- **En recreateSwapchain**: Se usa `oldSwapchain` para optimización

## Resultado Esperado

La aplicación debería:
- Responder correctamente al maximizar
- No marcarse como "(No responde)"
- Mantener el framerate estable durante el resize
- Procesar eventos de Windows correctamente

