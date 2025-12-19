#pragma once
#include <memory>
#include <vector>
#include <string>

namespace reactor {

// Forward declarations
class Game;
class GameObject;
class UISystem;
class Scene;

/**
 * @brief Editor - Visual Editor estilo Blender + Unreal Engine 5
 * 
 * Proporciona un editor visual completo para desarrollo de juegos en tiempo real:
 * - Scene Hierarchy (como Blender Outliner)
 * - Properties Panel (como UE5 Details)
 * - Viewport 3D con gizmos (como ambos)
 * - Asset Browser (como UE5 Content Browser)
 * - Console y Output (como UE5)
 * 
 * Uso:
 * Editor editor;
 * editor.run();
 */
class Editor {
public:
    Editor(const std::string& projectName = "REACTOR Project");
    ~Editor();
    
    // Main loop del editor
    void run();
    
    // Callbacks para personalización
    virtual void onEditorStart() {}
    virtual void onEditorUpdate(float deltaTime) {}
    virtual void onEditorRender() {}
    virtual void onEditorShutdown() {}

private:
    std::unique_ptr<Game> game;
    std::unique_ptr<UISystem> ui;
    
    // Estado del editor
    bool running{true};
    GameObject* selectedObject{nullptr};
    std::string projectName;
    
    // Panels
    void renderMenuBar();
    void renderSceneHierarchy();
    void renderPropertiesPanel();
    void renderViewport();
    void renderAssetBrowser();
    void renderConsole();
    void renderToolbar();
    
    // Gizmos y herramientas
    enum class GizmoMode { Translate, Rotate, Scale };
    GizmoMode currentGizmo{GizmoMode::Translate};
    
    void renderGizmos();
    void handleInput();
    
    // Asset management
    struct AssetEntry {
        std::string name;
        std::string path;
        std::string type;
    };
    std::vector<AssetEntry> assets;
    void scanAssets();
    
    // Console
    std::vector<std::string> consoleMessages;
    void log(const std::string& message);
};

/**
 * @brief EditorPresets - Configuraciones predefinidas del editor
 */
class EditorPresets {
public:
    // Layouts predefinidos
    static void layoutBlenderStyle(Editor& editor);
    static void layoutUnrealStyle(Editor& editor);
    static void layoutMinimal(Editor& editor);
    
    // Temas
    static void themeBlenderDark();
    static void themeUnrealDark();
    static void themeLight();
};

/**
 * @brief SceneEditor - Editor de escenas con manipulación visual
 */
class SceneEditor {
public:
    SceneEditor(Scene& scene);
    
    // Manipulación de objetos
    void selectObject(GameObject* obj);
    GameObject* getSelectedObject() const { return selected; }
    
    // Gizmos
    void drawTranslateGizmo();
    void drawRotateGizmo();
    void drawScaleGizmo();
    
    // Snapping
    void setSnapEnabled(bool enabled) { snapEnabled = enabled; }
    void setSnapValue(float value) { snapValue = value; }

private:
    Scene& scene;
    GameObject* selected{nullptr};
    bool snapEnabled{false};
    float snapValue{1.0f};
};

} // namespace reactor
