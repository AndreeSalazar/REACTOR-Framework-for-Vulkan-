// REACTOR Editor - Demo estilo Blender + Unreal Engine 5
#include "reactor/reactor.hpp"
#include "reactor/editor/editor.hpp"

using namespace reactor;

/**
 * @brief Mi Editor Personalizado
 * 
 * Hereda de Editor para personalizar el comportamiento
 */
class MyEditor : public Editor {
public:
    MyEditor() : Editor("Mi Proyecto REACTOR") {}
    
    void onEditorStart() override {
        std::cout << "\n=== EDITOR INICIADO ===\n";
        std::cout << "Estilo: Blender + Unreal Engine 5\n";
        std::cout << "Controles:\n";
        std::cout << "  - Click en Scene Hierarchy para seleccionar objetos\n";
        std::cout << "  - Modifica propiedades en Properties panel\n";
        std::cout << "  - Arrastra assets desde Asset Browser\n";
        std::cout << "  - Usa gizmos en Viewport para transformar\n\n";
        
        // Aplicar tema Blender
        EditorPresets::themeBlenderDark();
        
        // Crear objetos de ejemplo
        auto cube = game->createCube("Cube");
        cube->setPosition(0, 0, 0);
        cube->setColor(1, 0, 0);
        
        auto sphere = game->createSphere("Sphere");
        sphere->setPosition(3, 0, 0);
        sphere->setColor(0, 0, 1);
        
        auto light = game->createLight("MainLight");
        light->setPosition(5, 10, 5);
    }
    
    void onEditorUpdate(float deltaTime) override {
        // Lógica personalizada del editor
    }
    
    void onEditorRender() override {
        // Rendering personalizado
    }
    
    void onEditorShutdown() override {
        std::cout << "\n=== EDITOR CERRADO ===\n";
    }
};

/**
 * @brief Main - Editor Visual
 * 
 * Solo 3 líneas para un editor completo
 */
int main() {
    try {
        MyEditor editor;
        editor.run();
        return 0;
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
        return 1;
    }
}
