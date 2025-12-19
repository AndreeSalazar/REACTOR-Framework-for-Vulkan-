// REACTOR Game - Ejemplo ULTRA SIMPLE
// A (Vulkan) -> B (REACTOR) -> C (Game)

#include "reactor/reactor.hpp"
#include "reactor/game/game.hpp"

using namespace reactor;

/**
 * @brief Mi Juego - ULTRA SIMPLE
 * 
 * Solo heredar de Game y override 2 métodos
 */
class MyGame : public Game {
public:
    MyGame() : Game("Mi Primer Juego REACTOR", 1280, 720) {}
    
    void onCreate() override {
        std::cout << "\n=== MI JUEGO - onCreate ===\n";
        
        // Configuración inicial
        GamePresets::setup3DGame(*this);
        
        // Crear objetos (ULTRA SIMPLE)
        cube = createCube("RedCube");
        cube->setPosition(0, 0, 0);
        cube->setColor(1, 0, 0);
        
        sphere = createSphere("BlueSphere");
        sphere->setPosition(3, 0, 0);
        sphere->setColor(0, 0, 1);
        
        plane = createPlane("Ground");
        plane->setPosition(0, -1, 0);
        plane->setScale(10, 1, 10);
        plane->setColor(0.3f, 0.3f, 0.3f);
        
        std::cout << "✓ Objetos creados\n";
    }
    
    void onUpdate(float deltaTime) override {
        // Rotar cubo
        if (cube) {
            cube->rotate(0, deltaTime * 50.0f, 0);
        }
        
        // Mover esfera
        if (sphere) {
            static float time = 0;
            time += deltaTime;
            sphere->setPosition(3 + std::sin(time) * 2, 0, 0);
        }
        
        // FPS cada segundo
        static float fpsTimer = 0;
        fpsTimer += deltaTime;
        if (fpsTimer >= 1.0f) {
            std::cout << "FPS: " << getFPS() << "\n";
            fpsTimer = 0;
        }
    }
    
    void onRender() override {
        // Rendering automático por Game
    }
    
    void onDestroy() override {
        std::cout << "\n=== MI JUEGO - onDestroy ===\n";
        std::cout << "✓ Juego finalizado\n";
    }

private:
    GameObject* cube{nullptr};
    GameObject* sphere{nullptr};
    GameObject* plane{nullptr};
};

/**
 * @brief Main - ULTRA SIMPLE
 * 
 * Solo 3 líneas de código
 */
int main() {
    try {
        MyGame game;
        game.run();
        return 0;
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
        return 1;
    }
}
