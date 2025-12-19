// REACTOR Framework - FASE 2 COMPLETO Demo
#include "reactor/reactor.hpp"
#include "reactor/window.hpp"
#include "reactor/vulkan_context.hpp"
#include "reactor/mesh.hpp"
#include "reactor/material.hpp"
#include "reactor/texture.hpp"
#include "reactor/resource_manager.hpp"
#include <iostream>
#include <chrono>

using namespace reactor;

// Estado global simple
bool shouldRotate = true;
float rotationSpeed = 1.0f;


int main() {
    try {
        std::cout << "==========================================" << std::endl;
        std::cout << "  TEST GAME - REACTOR Framework" << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << std::endl;
        
        // [1] Inicializar REACTOR - UNA LÍNEA
        std::cout << "[1/5] Inicializando REACTOR..." << std::endl;
        Window::init();
        
        // [2] Crear ventana - CÓDIGO MUY CORTO
        std::cout << "[2/5] Creando ventana..." << std::endl;
        WindowConfig config;
        config.title = "Test Game - REACTOR";
        config.width = 1280;
        config.height = 720;
        config.vsync = true;
        Window window(config);
        std::cout << "      ✓ Ventana creada" << std::endl;
        
        // [3] Inicializar Vulkan - UNA LÍNEA
        std::cout << "[3/5] Inicializando Vulkan..." << std::endl;
        VulkanContext ctx(true);
        ctx.init();
        std::cout << "      ✓ Vulkan inicializado" << std::endl;
        
        // [4] ResourceManager - CACHE AUTOMÁTICO
        std::cout << "[4/7] Creando ResourceManager..." << std::endl;
        ResourceManager resources(ctx.allocator());
        std::cout << "      ✓ ResourceManager creado" << std::endl;
        
        // [5] Crear geometría - UNA LÍNEA CON REACTOR
        std::cout << "[5/7] Creando geometría con ResourceManager..." << std::endl;
        auto cubeMesh = resources.createCube("cube");
        auto sphereMesh = resources.createSphere("sphere", 16);
        auto planeMesh = resources.createPlane("plane");
        std::cout << "      ✓ Cubo: " << cubeMesh->vertexCount() << " vértices, " << cubeMesh->indexCount() << " índices" << std::endl;
        std::cout << "      ✓ Esfera: " << sphereMesh->vertexCount() << " vértices, " << sphereMesh->indexCount() << " índices" << std::endl;
        std::cout << "      ✓ Plano: " << planeMesh->vertexCount() << " vértices, " << planeMesh->indexCount() << " índices" << std::endl;
        std::cout << "      ✓ Meshes en cache: " << resources.getMeshCount() << std::endl;
        
        // [6] Crear texturas - UNA LÍNEA
        std::cout << "[6/7] Creando texturas..." << std::endl;
        auto albedoTex = Texture::load("textures/albedo.png", ctx.allocator());
        auto normalTex = Texture::load("textures/normal.png", ctx.allocator());
        auto solidTex = Texture::solidColor(1.0f, 0.0f, 0.0f, 1.0f, ctx.allocator());
        std::cout << "      ✓ Albedo: " << albedoTex.path() << " (" << albedoTex.width() << "x" << albedoTex.height() << ")" << std::endl;
        std::cout << "      ✓ Normal: " << normalTex.path() << " (" << normalTex.width() << "x" << normalTex.height() << ")" << std::endl;
        std::cout << "      ✓ Solid: " << solidTex.path() << " (" << solidTex.width() << "x" << solidTex.height() << ")" << std::endl;
        
        // [7] Crear materiales - CÓDIGO MUY CORTO
        std::cout << "[7/7] Creando materiales..." << std::endl;
        auto pbrMat = resources.getMaterial("pbr_red");
        pbrMat->setAlbedo(1.0f, 0.2f, 0.2f).setMetallic(0.8f).setRoughness(0.2f);
        pbrMat->albedoMap = &albedoTex;
        
        auto unlitMat = resources.getMaterial("unlit_green");
        unlitMat->setAlbedo(0.2f, 1.0f, 0.2f);
        
        auto wireMat = resources.getMaterial("wireframe");
        *wireMat = Material::wireframe();
        
        std::cout << "      ✓ Material PBR: albedo(" << pbrMat->albedo.r << ", " << pbrMat->albedo.g << ", " << pbrMat->albedo.b << ")" << std::endl;
        std::cout << "      ✓ Material Unlit creado" << std::endl;
        std::cout << "      ✓ Material Wireframe creado" << std::endl;
        std::cout << "      ✓ Materiales en cache: " << resources.getMaterialCount() << std::endl;
        
        Camera camera;
        camera.position = Vec3(2.0f, 2.0f, 2.0f);
        camera.target = Vec3(0.0f, 0.0f, 0.0f);
        camera.aspectRatio = 1280.0f / 720.0f;
        
        Transform cubeTransform;
        std::cout << "      ✓ Escena configurada" << std::endl;
        
        // Input callbacks - CÓDIGO MUY CORTO
        window.setKeyCallback([](int key, int action) {
            if (key == 256 && action == 1) { // ESC
                std::cout << "ESC - Cerrando..." << std::endl;
            }
            if (key == 32 && action == 1) { // SPACE
                shouldRotate = !shouldRotate;
                std::cout << "Rotación: " << (shouldRotate ? "ON" : "OFF") << std::endl;
            }
            if (key == 265 && action == 1) { // UP
                rotationSpeed += 0.5f;
                std::cout << "Velocidad: " << rotationSpeed << "x" << std::endl;
            }
            if (key == 264 && action == 1) { // DOWN
                rotationSpeed = std::max(0.1f, rotationSpeed - 0.5f);
                std::cout << "Velocidad: " << rotationSpeed << "x" << std::endl;
            }
        });
        
        std::cout << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << "  ✓ REACTOR Inicializado!" << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << std::endl;
        std::cout << "Características REACTOR FASE 2 - 100% COMPLETO:" << std::endl;
        std::cout << "  ✓ Window (GLFW wrapper)" << std::endl;
        std::cout << "  ✓ VulkanContext" << std::endl;
        std::cout << "  ✓ Mesh (Geometría predefinida)" << std::endl;
        std::cout << "  ✓ Material (Sistema PBR)" << std::endl;
        std::cout << "  ✓ Texture (Carga de imágenes)" << std::endl;
        std::cout << "  ✓ ResourceManager (Cache automático)" << std::endl;
        std::cout << "  ✓ Camera & Transform" << std::endl;
        std::cout << "  ✓ Math (GLM wrapper)" << std::endl;
        std::cout << std::endl;
        std::cout << "Stats ResourceManager:" << std::endl;
        std::cout << "  - Meshes: " << resources.getMeshCount() << std::endl;
        std::cout << "  - Texturas: 0 (creadas directamente)" << std::endl;
        std::cout << "  - Materiales: " << resources.getMaterialCount() << std::endl;
        std::cout << std::endl;
        std::cout << "Controles:" << std::endl;
        std::cout << "  ESC   - Salir" << std::endl;
        std::cout << "  SPACE - Pausar/Reanudar" << std::endl;
        std::cout << "  ↑/↓   - Velocidad" << std::endl;
        std::cout << std::endl;
        
        // Render loop - CÓDIGO MUY SIMPLE
        auto startTime = std::chrono::high_resolution_clock::now();
        size_t frameCount = 0;
        auto lastFpsTime = startTime;
        
        while (!window.shouldClose()) {
            window.pollEvents();
            
            auto currentTime = std::chrono::high_resolution_clock::now();
            float time = std::chrono::duration<float>(currentTime - startTime).count();
            
            // Actualizar transform - UNA LÍNEA
            if (shouldRotate) {
                cubeTransform.rotation.y = time * glm::radians(90.0f) * rotationSpeed;
            }
            
            // Calcular matrices MVP - CÓDIGO MUY CORTO
            Mat4 mvp = camera.getProjectionMatrix() * 
                      camera.getViewMatrix() * 
                      cubeTransform.getMatrix();
            
            frameCount++;
            
            auto elapsed = std::chrono::duration<double>(currentTime - lastFpsTime).count();
            if (elapsed >= 1.0) {
                double fps = frameCount / elapsed;
                std::cout << "FPS: " << static_cast<int>(fps) 
                         << " | Rotación: " << (shouldRotate ? "ON" : "OFF")
                         << " | Ángulo: " << static_cast<int>(glm::degrees(cubeTransform.rotation.y)) << "°" 
                         << " | Velocidad: " << rotationSpeed << "x" << std::endl;
                frameCount = 0;
                lastFpsTime = currentTime;
            }
        }
        
        std::cout << std::endl;
        std::cout << "==========================================" << std::endl;
        std::cout << "  Limpiando REACTOR..." << std::endl;
        std::cout << "==========================================" << std::endl;
        
        ctx.shutdown();
        Window::terminate();
        
        std::cout << "  ✓ Test Game finalizado" << std::endl;
        std::cout << "==========================================" << std::endl;
        
        return 0;
        
    } catch (const std::exception& e) {
        std::cerr << std::endl;
        std::cerr << "❌ Error: " << e.what() << std::endl;
        Window::terminate();
        return 1;
    }
}
