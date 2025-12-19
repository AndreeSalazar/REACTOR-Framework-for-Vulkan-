#include "reactor/game/game.hpp"
#include "reactor/rendering/easy_renderer.hpp"
#include <iostream>
#include <chrono>

namespace reactor {

Game::Game(const std::string& title, int width, int height) {
    std::cout << "===========================================\n";
    std::cout << "  REACTOR Game Engine\n";
    std::cout << "  A -> B -> C Architecture\n";
    std::cout << "  A (Vulkan) -> B (REACTOR) -> C (Game)\n";
    std::cout << "===========================================\n\n";
    
    initializeEngine();
}

Game::~Game() {
    cleanup();
}

void Game::initializeEngine() {
    std::cout << "[Game] Inicializando motor...\n";
    
    // Window
    Window::init();
    window = std::make_unique<Window>(WindowConfig{
        .title = "REACTOR Game",
        .width = 1280,
        .height = 720,
        .vsync = true
    });
    
    // Vulkan
    ctx = std::make_unique<VulkanContext>(true);
    ctx->init();
    
    // Scene
    scene = std::make_unique<Scene>("MainScene");
    
    // Renderer (FASE 8)
    renderer = std::make_unique<EasyRenderer>(*ctx, *window);
    
    // Cámara principal
    auto cameraEntity = scene->createEntity("MainCamera");
    cameraEntity->transform().position = Vec3(0, 2, 5);
    auto& cam = cameraEntity->addComponent<Camera>();
    cam.fov = 60.0f;
    cam.aspectRatio = 16.0f / 9.0f;
    cam.nearPlane = 0.1f;
    cam.farPlane = 100.0f;
    
    mainCamera = std::make_unique<GameObject>("MainCamera", scene.get());
    
    std::cout << "[Game] ✓ Motor inicializado\n\n";
}

void Game::run() {
    std::cout << "[Game] Llamando onCreate()...\n";
    onCreate();
    
    std::cout << "[Game] Iniciando game loop...\n\n";
    mainLoop();
}

void Game::mainLoop() {
    auto lastTime = std::chrono::high_resolution_clock::now();
    int frameCount = 0;
    auto lastFPSTime = lastTime;
    
    while (!window->shouldClose() && running) {
        // Delta time
        auto currentTime = std::chrono::high_resolution_clock::now();
        deltaTime = std::chrono::duration<float>(currentTime - lastTime).count();
        lastTime = currentTime;
        
        // Events
        window->pollEvents();
        
        // Update
        onUpdate(deltaTime);
        scene->update(deltaTime);
        
        // Render
        renderer->beginFrame();
        onRender();
        renderer->endFrame();
        
        // FPS
        frameCount++;
        auto elapsed = std::chrono::duration<float>(currentTime - lastFPSTime).count();
        if (elapsed >= 1.0f) {
            currentFPS = static_cast<int>(frameCount / elapsed);
            frameCount = 0;
            lastFPSTime = currentTime;
        }
    }
    
    std::cout << "\n[Game] Llamando onDestroy()...\n";
    onDestroy();
}

void Game::cleanup() {
    std::cout << "[Game] Limpiando motor...\n";
    
    mainCamera.reset();
    renderer.reset();
    scene.reset();
    ctx.reset();
    
    if (window) {
        window.reset();
        Window::terminate();
    }
    
    std::cout << "[Game] ✓ Limpieza completada\n";
}

GameObject* Game::createCube(const std::string& name) {
    auto obj = new GameObject(name, scene.get());
    std::cout << "[Game] Cubo creado: " << name << "\n";
    return obj;
}

GameObject* Game::createSphere(const std::string& name) {
    auto obj = new GameObject(name, scene.get());
    std::cout << "[Game] Esfera creada: " << name << "\n";
    return obj;
}

GameObject* Game::createPlane(const std::string& name) {
    auto obj = new GameObject(name, scene.get());
    std::cout << "[Game] Plano creado: " << name << "\n";
    return obj;
}

GameObject* Game::createEmpty(const std::string& name) {
    auto obj = new GameObject(name, scene.get());
    std::cout << "[Game] GameObject creado: " << name << "\n";
    return obj;
}

GameObject* Game::getMainCamera() {
    return mainCamera.get();
}

GameObject* Game::createLight(const std::string& name) {
    auto obj = new GameObject(name, scene.get());
    std::cout << "[Game] Luz creada: " << name << "\n";
    return obj;
}

bool Game::isKeyPressed(int key) {
    // TODO: Implementar con Input de FASE 5
    return false;
}

bool Game::isKeyDown(int key) {
    // TODO: Implementar con Input de FASE 5
    return false;
}

Vec2 Game::getMousePosition() {
    // TODO: Implementar
    return Vec2(0, 0);
}

void Game::setBackgroundColor(float r, float g, float b) {
    if (renderer) {
        renderer->setClearColor(r, g, b);
    }
}

void Game::setTargetFPS(int fps) {
    std::cout << "[Game] Target FPS: " << fps << "\n";
}

// GameObject implementation
GameObject::GameObject(const std::string& name, Scene* scene)
    : name(name), scene(scene) {
    entity = scene->createEntity(name);
}

void GameObject::setPosition(float x, float y, float z) {
    transform.position = Vec3(x, y, z);
    if (entity) {
        entity->transform().position = transform.position;
    }
}

void GameObject::setRotation(float x, float y, float z) {
    transform.rotation = Vec3(x, y, z);
    if (entity) {
        entity->transform().rotation = transform.rotation;
    }
}

void GameObject::setScale(float x, float y, float z) {
    transform.scale = Vec3(x, y, z);
    if (entity) {
        entity->transform().scale = transform.scale;
    }
}

void GameObject::move(float x, float y, float z) {
    transform.position += Vec3(x, y, z);
    if (entity) {
        entity->transform().position = transform.position;
    }
}

void GameObject::rotate(float x, float y, float z) {
    transform.rotation += Vec3(x, y, z);
    if (entity) {
        entity->transform().rotation = transform.rotation;
    }
}

void GameObject::setColor(float r, float g, float b) {
    color = Vec3(r, g, b);
}

void GameObject::setVisible(bool v) {
    visible = v;
}

// GamePresets implementation
void GamePresets::setup3DGame(Game& game) {
    std::cout << "[GamePresets] Configurando juego 3D...\n";
    game.setBackgroundColor(0.2f, 0.3f, 0.4f);
    addBasicLighting(game);
}

void GamePresets::setup2DGame(Game& game) {
    std::cout << "[GamePresets] Configurando juego 2D...\n";
    game.setBackgroundColor(0.1f, 0.1f, 0.1f);
}

void GamePresets::addFPSControls(Game& game, GameObject* camera) {
    std::cout << "[GamePresets] Controles FPS agregados\n";
    // TODO: Implementar controles FPS
}

void GamePresets::addBasicLighting(Game& game) {
    std::cout << "[GamePresets] Iluminación básica agregada\n";
    auto light = game.createLight("DirectionalLight");
    light->setPosition(5, 10, 5);
}

} // namespace reactor
