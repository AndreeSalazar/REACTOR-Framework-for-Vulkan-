#include "reactor/editor/editor.hpp"
#include "reactor/game/game.hpp"
#include "reactor/tools/ui_system.hpp"
#include <iostream>

#ifdef REACTOR_HAS_WINDOW
#include <imgui.h>
#endif

namespace reactor {

Editor::Editor(const std::string& projectName)
    : projectName(projectName) {
    
    std::cout << "===========================================\n";
    std::cout << "  REACTOR Editor - Estilo Blender + UE5\n";
    std::cout << "  Visual Editor para Desarrollo en Tiempo Real\n";
    std::cout << "===========================================\n\n";
    
    // Crear game instance
    game = std::make_unique<Game>(projectName, 1920, 1080);
    
    // Inicializar UI
    ui = std::make_unique<UISystem>();
    
    // Escanear assets
    scanAssets();
    
    std::cout << "[Editor] ✓ Editor inicializado\n\n";
}

Editor::~Editor() {
    std::cout << "[Editor] Cerrando editor...\n";
}

void Editor::run() {
    std::cout << "[Editor] Iniciando editor visual...\n\n";
    
    onEditorStart();
    
    // Main loop
    while (running) {
        // TODO: Check window close
        
        float deltaTime = game->getDeltaTime();
        
        // Update
        handleInput();
        onEditorUpdate(deltaTime);
        
        // Render UI
#ifdef REACTOR_HAS_WINDOW
        ui->newFrame();
        
        // Docking space (requires ImGui docking branch)
        // ImGui::DockSpaceOverViewport(ImGui::GetMainViewport());
        
        renderMenuBar();
        renderSceneHierarchy();
        renderPropertiesPanel();
        renderViewport();
        renderAssetBrowser();
        renderConsole();
        renderToolbar();
        
        ui->render();
#endif
        
        onEditorRender();
    }
    
    onEditorShutdown();
}

void Editor::renderMenuBar() {
#ifdef REACTOR_HAS_WINDOW
    if (ImGui::BeginMainMenuBar()) {
        if (ImGui::BeginMenu("File")) {
            if (ImGui::MenuItem("New Scene", "Ctrl+N")) {
                log("Nueva escena creada");
            }
            if (ImGui::MenuItem("Open Scene", "Ctrl+O")) {
                log("Abrir escena");
            }
            if (ImGui::MenuItem("Save Scene", "Ctrl+S")) {
                log("Escena guardada");
            }
            ImGui::Separator();
            if (ImGui::MenuItem("Exit", "Alt+F4")) {
                running = false;
            }
            ImGui::EndMenu();
        }
        
        if (ImGui::BeginMenu("Edit")) {
            if (ImGui::MenuItem("Undo", "Ctrl+Z")) {}
            if (ImGui::MenuItem("Redo", "Ctrl+Y")) {}
            ImGui::Separator();
            if (ImGui::MenuItem("Preferences")) {
                log("Abriendo preferencias");
            }
            ImGui::EndMenu();
        }
        
        if (ImGui::BeginMenu("GameObject")) {
            if (ImGui::MenuItem("Create Cube")) {
                auto cube = game->createCube("Cube");
                log("Cubo creado");
            }
            if (ImGui::MenuItem("Create Sphere")) {
                auto sphere = game->createSphere("Sphere");
                log("Esfera creada");
            }
            if (ImGui::MenuItem("Create Light")) {
                auto light = game->createLight("Light");
                log("Luz creada");
            }
            ImGui::EndMenu();
        }
        
        if (ImGui::BeginMenu("Window")) {
            if (ImGui::MenuItem("Layout: Blender")) {
                EditorPresets::layoutBlenderStyle(*this);
            }
            if (ImGui::MenuItem("Layout: Unreal")) {
                EditorPresets::layoutUnrealStyle(*this);
            }
            ImGui::EndMenu();
        }
        
        ImGui::EndMainMenuBar();
    }
#endif
}

void Editor::renderSceneHierarchy() {
#ifdef REACTOR_HAS_WINDOW
    ImGui::Begin("Scene Hierarchy");
    
    ImGui::Text("Scene: Main Scene");
    ImGui::Separator();
    
    // Lista de objetos (placeholder)
    if (ImGui::TreeNode("Scene Objects")) {
        if (ImGui::Selectable("Main Camera")) {
            selectedObject = game->getMainCamera();
        }
        
        // Más objetos aquí
        
        ImGui::TreePop();
    }
    
    ImGui::End();
#endif
}

void Editor::renderPropertiesPanel() {
#ifdef REACTOR_HAS_WINDOW
    ImGui::Begin("Properties");
    
    if (selectedObject) {
        ImGui::Text("Object: %s", selectedObject->getName().c_str());
        ImGui::Separator();
        
        // Transform
        if (ImGui::CollapsingHeader("Transform", ImGuiTreeNodeFlags_DefaultOpen)) {
            Vec3& pos = selectedObject->position();
            Vec3& rot = selectedObject->rotation();
            Vec3& scl = selectedObject->scale();
            
            ImGui::DragFloat3("Position", &pos.x, 0.1f);
            ImGui::DragFloat3("Rotation", &rot.x, 1.0f);
            ImGui::DragFloat3("Scale", &scl.x, 0.1f);
        }
        
        // Más propiedades aquí
        
    } else {
        ImGui::TextDisabled("No object selected");
    }
    
    ImGui::End();
#endif
}

void Editor::renderViewport() {
#ifdef REACTOR_HAS_WINDOW
    ImGui::Begin("Viewport");
    
    ImGui::Text("3D Viewport - Rendering aquí");
    ImGui::Text("FPS: %d", game->getFPS());
    
    // Gizmos
    renderGizmos();
    
    ImGui::End();
#endif
}

void Editor::renderAssetBrowser() {
#ifdef REACTOR_HAS_WINDOW
    ImGui::Begin("Asset Browser");
    
    ImGui::Text("Assets: %zu", assets.size());
    ImGui::Separator();
    
    for (const auto& asset : assets) {
        if (ImGui::Selectable(asset.name.c_str())) {
            log("Asset seleccionado: " + asset.name);
        }
    }
    
    ImGui::End();
#endif
}

void Editor::renderConsole() {
#ifdef REACTOR_HAS_WINDOW
    ImGui::Begin("Console");
    
    for (const auto& msg : consoleMessages) {
        ImGui::TextUnformatted(msg.c_str());
    }
    
    if (ImGui::GetScrollY() >= ImGui::GetScrollMaxY()) {
        ImGui::SetScrollHereY(1.0f);
    }
    
    ImGui::End();
#endif
}

void Editor::renderToolbar() {
#ifdef REACTOR_HAS_WINDOW
    ImGui::Begin("Toolbar", nullptr, ImGuiWindowFlags_NoTitleBar);
    
    if (ImGui::Button("Translate")) currentGizmo = GizmoMode::Translate;
    ImGui::SameLine();
    if (ImGui::Button("Rotate")) currentGizmo = GizmoMode::Rotate;
    ImGui::SameLine();
    if (ImGui::Button("Scale")) currentGizmo = GizmoMode::Scale;
    
    ImGui::End();
#endif
}

void Editor::renderGizmos() {
#ifdef REACTOR_HAS_WINDOW
    if (!selectedObject) return;
    
    switch (currentGizmo) {
        case GizmoMode::Translate:
            ImGui::Text("Gizmo: Translate");
            break;
        case GizmoMode::Rotate:
            ImGui::Text("Gizmo: Rotate");
            break;
        case GizmoMode::Scale:
            ImGui::Text("Gizmo: Scale");
            break;
    }
#endif
}

void Editor::handleInput() {
    // TODO: Implementar input handling
}

void Editor::scanAssets() {
    // TODO: Escanear directorio de assets
    assets.push_back({"cube.obj", "assets/models/cube.obj", "Model"});
    assets.push_back({"texture.png", "assets/textures/texture.png", "Texture"});
}

void Editor::log(const std::string& message) {
    consoleMessages.push_back("[Editor] " + message);
    std::cout << "[Editor] " << message << std::endl;
}

// EditorPresets implementation
void EditorPresets::layoutBlenderStyle(Editor& editor) {
    std::cout << "[EditorPresets] Aplicando layout estilo Blender\n";
    // TODO: Configurar layout
}

void EditorPresets::layoutUnrealStyle(Editor& editor) {
    std::cout << "[EditorPresets] Aplicando layout estilo Unreal\n";
    // TODO: Configurar layout
}

void EditorPresets::layoutMinimal(Editor& editor) {
    std::cout << "[EditorPresets] Aplicando layout minimal\n";
    // TODO: Configurar layout
}

void EditorPresets::themeBlenderDark() {
#ifdef REACTOR_HAS_WINDOW
    ImGui::StyleColorsDark();
    // TODO: Personalizar colores estilo Blender
#endif
}

void EditorPresets::themeUnrealDark() {
#ifdef REACTOR_HAS_WINDOW
    ImGui::StyleColorsDark();
    // TODO: Personalizar colores estilo Unreal
#endif
}

void EditorPresets::themeLight() {
#ifdef REACTOR_HAS_WINDOW
    ImGui::StyleColorsLight();
#endif
}

// SceneEditor implementation
SceneEditor::SceneEditor(Scene& scene)
    : scene(scene) {
}

void SceneEditor::selectObject(GameObject* obj) {
    selected = obj;
}

void SceneEditor::drawTranslateGizmo() {
    // TODO: Implementar gizmo de traducción
}

void SceneEditor::drawRotateGizmo() {
    // TODO: Implementar gizmo de rotación
}

void SceneEditor::drawScaleGizmo() {
    // TODO: Implementar gizmo de escala
}

} // namespace reactor
