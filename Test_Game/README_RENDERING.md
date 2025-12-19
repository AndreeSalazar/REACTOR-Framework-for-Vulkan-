# Por qué la ventana está en blanco

## Problema
Test_Game actualmente solo demuestra las **APIs de REACTOR** (creación de meshes, materiales, luces, etc.) pero **NO tiene código de rendering real**.

Para ver algo en pantalla con Vulkan, necesitas:

1. **Swapchain** - Para presentar imágenes en la ventana
2. **RenderPass** - Para definir cómo renderizar
3. **Framebuffers** - Uno por cada imagen del swapchain
4. **CommandBuffers** - Para grabar comandos de rendering
5. **Pipeline** - Para definir shaders y estado de rendering
6. **Synchronization** - Semáforos y fences para sincronizar

## Solución Temporal

Test_Game es actualmente un **demo de API**, no un motor de rendering completo. Muestra:
- ✅ Cómo crear meshes, materiales, texturas
- ✅ Cómo usar el sistema de escenas y componentes
- ✅ Cómo configurar luces y efectos
- ❌ NO renderiza nada a la pantalla (aún)

## Próximos Pasos

Para tener rendering real, necesitarías:

```cpp
// 1. Crear swapchain
auto swapchain = Swapchain::create(ctx.device(), window.handle())
    .format(VK_FORMAT_B8G8R8A8_SRGB)
    .presentMode(VK_PRESENT_MODE_FIFO_KHR)
    .build();

// 2. Crear render pass
auto renderPass = RenderPass::create(ctx.device())
    .colorAttachment(swapchain.format())
    .build();

// 3. Crear pipeline
auto pipeline = Pipeline::create(ctx.device())
    .shader("vertex.spv", ShaderStage::Vertex)
    .shader("fragment.spv", ShaderStage::Fragment)
    .renderPass(renderPass)
    .build();

// 4. Loop de rendering
while (!window.shouldClose()) {
    // Acquire image
    uint32_t imageIndex = swapchain.acquireNextImage();
    
    // Record commands
    commandBuffer.begin();
    commandBuffer.beginRenderPass(renderPass, framebuffers[imageIndex]);
    commandBuffer.bindPipeline(pipeline);
    commandBuffer.draw(mesh);
    commandBuffer.endRenderPass();
    commandBuffer.end();
    
    // Submit and present
    queue.submit(commandBuffer);
    swapchain.present(imageIndex);
}
```

## Conclusión

Test_Game es un **demo de características**, no un motor completo. Para ver rendering real, necesitarías implementar el loop de rendering completo de Vulkan, lo cual está fuera del alcance de este demo de API.

El objetivo de Test_Game es mostrar **qué tan simple es usar REACTOR** para crear recursos, no implementar un motor de rendering completo.
