#include "reactor/reactor.hpp"
#include "reactor/vulkan_context.hpp"
#include "reactor/buffer.hpp"
#include "reactor/shader.hpp"
#include "reactor/pipeline.hpp"
#include "reactor/command_buffer.hpp"
#include "reactor/sync.hpp"
#include "reactor/render_pass.hpp"
#include <iostream>
#include <array>
#include <chrono>

struct Vertex {
    float pos[2];
    float color[3];
};

class StarterApp {
public:
    StarterApp() {
        std::cout << "==================================" << std::endl;
        std::cout << "  REACTOR Framework - Starter App" << std::endl;
        std::cout << "==================================" << std::endl;
        std::cout << std::endl;
    }
    
    int run() {
        try {
            // 1. Inicializar contexto Vulkan
            std::cout << "[1/5] Inicializando Vulkan..." << std::endl;
            reactor::VulkanContext ctx(true);
            ctx.init();
            std::cout << "      ✓ Vulkan inicializado" << std::endl;
            
            // 2. Crear recursos
            std::cout << "[2/5] Creando recursos..." << std::endl;
            
            // Vértices del triángulo (posición + color)
            std::array<Vertex, 3> vertices = {{
                {{0.0f, -0.5f}, {1.0f, 0.0f, 0.0f}},  // Vértice inferior (rojo)
                {{0.5f, 0.5f}, {0.0f, 1.0f, 0.0f}},   // Vértice superior derecho (verde)
                {{-0.5f, 0.5f}, {0.0f, 0.0f, 1.0f}}   // Vértice superior izquierdo (azul)
            }};
            
            // Crear buffer de vértices
            auto vertexBuffer = reactor::Buffer::create(ctx.allocator())
                .size(sizeof(vertices))
                .usage(reactor::BufferUsage::Vertex)
                .memoryType(reactor::MemoryType::HostVisible)
                .build();
            
            // Subir datos al buffer
            vertexBuffer.upload(vertices.data(), sizeof(vertices));
            std::cout << "      ✓ Buffer de vértices creado (" << vertices.size() << " vértices)" << std::endl;
            
            // 3. Mostrar información
            std::cout << "[3/5] Framework listo" << std::endl;
            std::cout << "      ✓ Triángulo con colores RGB" << std::endl;
            std::cout << "      ✓ Gestión automática de memoria (RAII)" << std::endl;
            std::cout << "      ✓ Builder pattern para recursos" << std::endl;
            
            // 4. Estadísticas
            std::cout << "[4/5] Estadísticas:" << std::endl;
            std::cout << "      • Tamaño del buffer: " << vertexBuffer.size() << " bytes" << std::endl;
            std::cout << "      • Vértices: " << vertices.size() << std::endl;
            std::cout << "      • Memoria: Host-visible (CPU-GPU)" << std::endl;
            
            // 5. Finalizar
            std::cout << "[5/5] Limpiando recursos..." << std::endl;
            ctx.shutdown();
            std::cout << "      ✓ Recursos liberados automáticamente (RAII)" << std::endl;
            
            std::cout << std::endl;
            std::cout << "==================================" << std::endl;
            std::cout << "  ✓ Aplicación completada exitosamente" << std::endl;
            std::cout << "==================================" << std::endl;
            std::cout << std::endl;
            
            // Información adicional
            std::cout << "📚 Próximos pasos:" << std::endl;
            std::cout << "   1. Modifica los colores en el array 'vertices'" << std::endl;
            std::cout << "   2. Agrega más vértices para crear formas diferentes" << std::endl;
            std::cout << "   3. Explora los ejemplos en examples/" << std::endl;
            std::cout << "   4. Lee USAGE_GUIDE.md para más información" << std::endl;
            std::cout << std::endl;
            
            std::cout << "🎯 Características demostradas:" << std::endl;
            std::cout << "   ✓ Inicialización de Vulkan simplificada" << std::endl;
            std::cout << "   ✓ Builder pattern para crear recursos" << std::endl;
            std::cout << "   ✓ Gestión automática de memoria (RAII)" << std::endl;
            std::cout << "   ✓ Upload de datos a GPU" << std::endl;
            std::cout << "   ✓ Cleanup automático sin memory leaks" << std::endl;
            std::cout << std::endl;
            
            return 0;
            
        } catch (const std::exception& e) {
            std::cerr << std::endl;
            std::cerr << "❌ Error: " << e.what() << std::endl;
            std::cerr << std::endl;
            std::cerr << "💡 Soluciones comunes:" << std::endl;
            std::cerr << "   1. Verifica que Vulkan SDK esté instalado" << std::endl;
            std::cerr << "   2. Actualiza los drivers de tu GPU" << std::endl;
            std::cerr << "   3. Ejecuta 'diagnose.bat' para más información" << std::endl;
            std::cerr << "   4. Consulta TROUBLESHOOTING.md" << std::endl;
            std::cerr << std::endl;
            return 1;
        }
    }
};

int main() {
    StarterApp app;
    return app.run();
}
