#pragma once
#include "../reactor.hpp"
#include <functional>
#include <memory>

namespace reactor {

/**
 * @brief Game - Capa final de abstracción
 * 
 * A (Vulkan) -> B (REACTOR) -> C (Game)
 * 
 * Uso ULTRA simple:
 * class MyGame : public Game {
 *     void onCreate() override {
 *         auto cube = createCube();
 *         cube->setColor(1, 0, 0);
 *     }
 * };
 * 
 * int main() {
 *     MyGame game;
 *     game.run();
 * }
 */
class Game {
public:
    Game(const std::string& title = "REACTOR Game", int width = 1280, int height = 720);
    virtual ~Game();
    
    // Método principal - Solo llamar run()
    void run();
    
    // Lifecycle (override en tu juego)
    virtual void onCreate() {}
    virtual void onUpdate(float deltaTime) {}
    virtual void onRender() {}
    virtual void onDestroy() {}
    
    // API ultra simple - Crear objetos
    class GameObject* createCube(const std::string& name = "Cube");
    class GameObject* createSphere(const std::string& name = "Sphere");
    class GameObject* createPlane(const std::string& name = "Plane");
    class GameObject* createEmpty(const std::string& name = "GameObject");
    
    // Cámara y luces
    class GameObject* getMainCamera();
    class GameObject* createLight(const std::string& name = "Light");
    
    // Input simple
    bool isKeyPressed(int key);
    bool isKeyDown(int key);
    Vec2 getMousePosition();
    
    // Configuración
    void setBackgroundColor(float r, float g, float b);
    void setTargetFPS(int fps);
    
    // Getters
    float getDeltaTime() const { return deltaTime; }
    int getFPS() const { return currentFPS; }
    Scene& getScene() { return *scene; }

protected:
    // Internos (ocultos del usuario)
    std::unique_ptr<Window> window;
    std::unique_ptr<VulkanContext> ctx;
    std::unique_ptr<Scene> scene;
    std::unique_ptr<class EasyRenderer> renderer;
    std::unique_ptr<class GameObject> mainCamera;
    
    float deltaTime{0.0f};
    int currentFPS{0};
    bool running{true};
    
    void initializeEngine();
    void mainLoop();
    void cleanup();
};

/**
 * @brief GameObject - Objeto de juego simple (como Unity)
 */
class GameObject {
public:
    GameObject(const std::string& name, Scene* scene);
    
    // Transform
    Vec3& position() { return transform.position; }
    Vec3& rotation() { return transform.rotation; }
    Vec3& scale() { return transform.scale; }
    
    void setPosition(float x, float y, float z);
    void setRotation(float x, float y, float z);
    void setScale(float x, float y, float z);
    
    void move(float x, float y, float z);
    void rotate(float x, float y, float z);
    
    // Propiedades visuales
    void setColor(float r, float g, float b);
    void setVisible(bool visible);
    
    // Componentes
    template<typename T>
    T* addComponent();
    
    template<typename T>
    T* getComponent();
    
    // Nombre
    const std::string& getName() const { return name; }
    void setName(const std::string& newName) { name = newName; }

private:
    std::string name;
    Scene* scene;
    Entity* entity{nullptr};
    SimpleTransform transform;
    Vec3 color{1, 1, 1};
    bool visible{true};
    
    friend class Game;
};

/**
 * @brief GamePresets - Presets para juegos instantáneos
 */
class GamePresets {
public:
    // Crear juego 3D básico
    static void setup3DGame(Game& game);
    
    // Crear juego 2D básico
    static void setup2DGame(Game& game);
    
    // Agregar controles FPS
    static void addFPSControls(Game& game, GameObject* camera);
    
    // Agregar iluminación básica
    static void addBasicLighting(Game& game);
};

} // namespace reactor
