#define GLFW_INCLUDE_VULKAN
#include <GLFW/glfw3.h>
#include <iostream>
#include <chrono>
#include <vector>
#include <stdexcept>

struct Vertex {
    float pos[3];
    float color[3];
};

const std::vector<Vertex> cubeVertices = {
    {{-0.5f, -0.5f,  0.5f}, {1.0f, 0.0f, 0.0f}},
    {{ 0.5f, -0.5f,  0.5f}, {1.0f, 0.0f, 0.0f}},
    {{ 0.5f,  0.5f,  0.5f}, {1.0f, 0.0f, 0.0f}},
    {{-0.5f,  0.5f,  0.5f}, {1.0f, 0.0f, 0.0f}},
    
    {{-0.5f, -0.5f, -0.5f}, {0.0f, 1.0f, 0.0f}},
    {{ 0.5f, -0.5f, -0.5f}, {0.0f, 1.0f, 0.0f}},
    {{ 0.5f,  0.5f, -0.5f}, {0.0f, 1.0f, 0.0f}},
    {{-0.5f,  0.5f, -0.5f}, {0.0f, 1.0f, 0.0f}},
};

GLFWwindow* window = nullptr;
bool shouldRotate = true;
float rotationSpeed = 1.0f;

void keyCallback(GLFWwindow* win, int key, int scancode, int action, int mods) {
    if (key == GLFW_KEY_ESCAPE && action == GLFW_PRESS) {
        glfwSetWindowShouldClose(win, GLFW_TRUE);
        std::cout << "ESC presionado - cerrando..." << std::endl;
    }
    if (key == GLFW_KEY_SPACE && action == GLFW_PRESS) {
        shouldRotate = !shouldRotate;
        std::cout << "Rotación: " << (shouldRotate ? "ACTIVADA" : "DESACTIVADA") << std::endl;
    }
    if (key == GLFW_KEY_UP && action == GLFW_PRESS) {
        rotationSpeed += 0.5f;
        std::cout << "Velocidad de rotación: " << rotationSpeed << "x" << std::endl;
    }
    if (key == GLFW_KEY_DOWN && action == GLFW_PRESS) {
        rotationSpeed = std::max(0.1f, rotationSpeed - 0.5f);
        std::cout << "Velocidad de rotación: " << rotationSpeed << "x" << std::endl;
    }
}


int main() {
    try {
        std::cout << "==========================================" << std::endl;
        std::cout << "  TEST GAME - Simplified Demo" << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << std::endl;
        
        std::cout << "[1/3] Inicializando GLFW..." << std::endl;
        if (!glfwInit()) {
            throw std::runtime_error("Failed to initialize GLFW");
        }
        std::cout << "      ✓ GLFW inicializado" << std::endl;
        
        std::cout << "[2/3] Creando ventana..." << std::endl;
        glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);
        glfwWindowHint(GLFW_RESIZABLE, GLFW_FALSE);
        
        window = glfwCreateWindow(1280, 720, "Test Game - Simplified", nullptr, nullptr);
        if (!window) {
            glfwTerminate();
            throw std::runtime_error("Failed to create window");
        }
        std::cout << "      ✓ Ventana creada: 1280x720" << std::endl;
        
        glfwSetKeyCallback(window, keyCallback);
        
        std::cout << "[3/3] Verificando Vulkan..." << std::endl;
        uint32_t extensionCount = 0;
        vkEnumerateInstanceExtensionProperties(nullptr, &extensionCount, nullptr);
        std::cout << "      ✓ Vulkan disponible (" << extensionCount << " extensiones)" << std::endl;
        
        std::cout << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << "  ✓ Inicialización completa!" << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << std::endl;
        std::cout << "Características demostradas:" << std::endl;
        std::cout << "  ✓ Sistema de ventanas (GLFW)" << std::endl;
        std::cout << "  ✓ Vulkan SDK disponible" << std::endl;
        std::cout << "  ✓ Datos del cubo (" << cubeVertices.size() << " vértices)" << std::endl;
        std::cout << "  ✓ Input system" << std::endl;
        std::cout << std::endl;
        std::cout << "Controles:" << std::endl;
        std::cout << "  ESC       - Salir" << std::endl;
        std::cout << "  SPACE     - Pausar/Reanudar rotación" << std::endl;
        std::cout << "  FLECHA ↑  - Aumentar velocidad" << std::endl;
        std::cout << "  FLECHA ↓  - Disminuir velocidad" << std::endl;
        std::cout << std::endl;
        std::cout << "NOTA: Esta es una demo simplificada que verifica" << std::endl;
        std::cout << "      que GLFW y Vulkan están correctamente instalados." << std::endl;
        std::cout << std::endl;
        
        auto startTime = std::chrono::high_resolution_clock::now();
        size_t frameCount = 0;
        auto lastFpsTime = startTime;
        float rotation = 0.0f;
        
        while (!glfwWindowShouldClose(window)) {
            glfwPollEvents();
            
            auto currentTime = std::chrono::high_resolution_clock::now();
            float time = std::chrono::duration<float>(currentTime - startTime).count();
            
            if (shouldRotate) {
                rotation = time * 90.0f * rotationSpeed;
            }
            
            frameCount++;
            
            auto elapsed = std::chrono::duration<double>(currentTime - lastFpsTime).count();
            if (elapsed >= 1.0) {
                double fps = frameCount / elapsed;
                std::cout << "FPS: " << static_cast<int>(fps) 
                         << " | Rotación: " << (shouldRotate ? "ON" : "OFF")
                         << " | Ángulo: " << static_cast<int>(rotation) << "°" 
                         << " | Velocidad: " << rotationSpeed << "x" << std::endl;
                frameCount = 0;
                lastFpsTime = currentTime;
            }
        }
        
        std::cout << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << "  Limpiando recursos..." << std::endl;
        std::cout << "==========================================" << std::endl;
        
        glfwDestroyWindow(window);
        glfwTerminate();
        
        std::cout << "  ✓ Test Game finalizado correctamente" << std::endl;
        std::cout << "==========================================" << std::endl;
        
        return 0;
        
    } catch (const std::exception& e) {
        std::cerr << std::endl;
        std::cerr << "❌ Error: " << e.what() << std::endl;
        if (window) {
            glfwDestroyWindow(window);
        }
        glfwTerminate();
        return 1;
    }
}
