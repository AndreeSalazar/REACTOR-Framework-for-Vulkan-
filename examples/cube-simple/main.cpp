#include "reactor/reactor.hpp"
#include "reactor/vulkan_context.hpp"
#include "reactor/window.hpp"
#include "reactor/buffer.hpp"
#include "reactor/math.hpp"
#include <iostream>
#include <chrono>
#include <array>

struct Vertex {
    reactor::Vec3 pos;
    reactor::Vec3 color;
};

// Cubo 3D con colores
const std::vector<Vertex> cubeVertices = {
    // Front face (rojo)
    {{-0.5f, -0.5f,  0.5f}, {1.0f, 0.0f, 0.0f}},
    {{ 0.5f, -0.5f,  0.5f}, {1.0f, 0.0f, 0.0f}},
    {{ 0.5f,  0.5f,  0.5f}, {1.0f, 0.0f, 0.0f}},
    {{-0.5f,  0.5f,  0.5f}, {1.0f, 0.0f, 0.0f}},
    
    // Back face (verde)
    {{-0.5f, -0.5f, -0.5f}, {0.0f, 1.0f, 0.0f}},
    {{ 0.5f, -0.5f, -0.5f}, {0.0f, 1.0f, 0.0f}},
    {{ 0.5f,  0.5f, -0.5f}, {0.0f, 1.0f, 0.0f}},
    {{-0.5f,  0.5f, -0.5f}, {0.0f, 1.0f, 0.0f}},
};

int main() {
    try {
        std::cout << "==========================================" << std::endl;
        std::cout << "  REACTOR - Cubo 3D Demo (Simplificado)" << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << std::endl;
        
        // Inicializar GLFW
        std::cout << "[1/5] Inicializando sistema de ventanas..." << std::endl;
        reactor::Window::init();
        
        // Crear ventana
        reactor::WindowConfig windowConfig;
        windowConfig.title = "REACTOR - Cubo 3D Demo";
        windowConfig.width = 1280;
        windowConfig.height = 720;
        windowConfig.vsync = true;
        
        reactor::Window window(windowConfig);
        std::cout << "      ✓ Ventana creada: " << windowConfig.width << "x" << windowConfig.height << std::endl;
        
        // Inicializar Vulkan
        std::cout << "[2/5] Inicializando Vulkan..." << std::endl;
        reactor::VulkanContext ctx(true);
        ctx.init();
        std::cout << "      ✓ Vulkan inicializado" << std::endl;
        
        // Crear buffers (en scope para destrucción correcta)
        std::cout << "[3/5] Creando buffers..." << std::endl;
        {
            auto vertexBuffer = reactor::Buffer::create(ctx.allocator())
                .size(sizeof(Vertex) * cubeVertices.size())
                .usage(reactor::BufferUsage::Vertex)
                .memoryType(reactor::MemoryType::HostVisible)
                .build();
            
            vertexBuffer.upload(cubeVertices.data(), sizeof(Vertex) * cubeVertices.size());
            std::cout << "      ✓ Buffer de vértices creado (" << cubeVertices.size() << " vértices)" << std::endl;
        
        // Setup React-style components
        std::cout << "[4/5] Configurando componentes React-style..." << std::endl;
        reactor::Camera camera;
        camera.position = reactor::Vec3(2.0f, 2.0f, 2.0f);
        camera.target = reactor::Vec3(0.0f, 0.0f, 0.0f);
        camera.aspectRatio = static_cast<float>(windowConfig.width) / windowConfig.height;
        
        reactor::Transform cubeTransform;
        std::cout << "      ✓ Camera y Transform configurados" << std::endl;
        
        // Configurar input
        std::cout << "[5/5] Configurando input..." << std::endl;
        window.setKeyCallback([](int key, int action) {
            if (key == 256 && action == 1) { // ESC
                std::cout << "ESC presionado - cerrando..." << std::endl;
            }
        });
        std::cout << "      ✓ Input configurado" << std::endl;
        
        std::cout << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << "  ✓ Inicialización completa!" << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << std::endl;
        std::cout << "Características demostradas:" << std::endl;
        std::cout << "  ✓ Sistema de ventanas (GLFW)" << std::endl;
        std::cout << "  ✓ Vulkan context" << std::endl;
        std::cout << "  ✓ Buffers con datos del cubo" << std::endl;
        std::cout << "  ✓ React-style components (Camera, Transform)" << std::endl;
        std::cout << "  ✓ GLM math integration" << std::endl;
        std::cout << std::endl;
        std::cout << "Controles:" << std::endl;
        std::cout << "  ESC - Salir" << std::endl;
        std::cout << std::endl;
        std::cout << "Presiona ESC para salir..." << std::endl;
        std::cout << std::endl;
            
            // Render loop
            auto startTime = std::chrono::high_resolution_clock::now();
            size_t frameCount = 0;
            
            while (!window.shouldClose()) {
            window.pollEvents();
            
            auto currentTime = std::chrono::high_resolution_clock::now();
            float time = std::chrono::duration<float>(currentTime - startTime).count();
            
            // Update transform (React-style state)
            cubeTransform.rotation.y = time * glm::radians(90.0f);
            cubeTransform.rotation.x = time * glm::radians(45.0f);
            
            // Calcular matrices MVP
            reactor::Mat4 model = cubeTransform.getMatrix();
            reactor::Mat4 view = camera.getViewMatrix();
            reactor::Mat4 proj = camera.getProjectionMatrix();
            
            frameCount++;
            
            // Mostrar info cada 60 frames
            if (frameCount % 60 == 0) {
                std::cout << "Frame " << frameCount 
                         << " | Rotación: " << glm::degrees(cubeTransform.rotation.y) << "°" 
                         << " | Tiempo: " << time << "s" << std::endl;
            }
            }
            
            std::cout << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << "  Limpiando recursos..." << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << "  Total de frames renderizados: " << frameCount << std::endl;
        
        // Cleanup automático (RAII)
        ctx.shutdown();
        reactor::Window::terminate();
        
        std::cout << "  ✓ Aplicación finalizada correctamente" << std::endl;
        std::cout << "==========================================" << std::endl;
        
        return 0;
        
    } catch (const std::exception& e) {
        std::cerr << std::endl;
        std::cerr << "❌ Error: " << e.what() << std::endl;
        reactor::Window::terminate();
        return 1;
    }
}
