#pragma once
#include <string>
#include <glm/glm.hpp>

namespace debug {

enum class VisualizationMode {
    Normal = 0,           // Renderizado normal con Phong
    Wireframe = 1,        // Wireframe (solo bordes)
    Normals = 2,          // Visualización de normales (colores RGB)
    Depth = 3,            // Visualización de depth buffer (escala de grises)
    ImportanceMap = 4,    // ISR: Mapa de importancia (mapa de calor)
    PixelSizing = 5,      // ISR: Tamaño de píxeles adaptativos
    TemporalCoherence = 6 // ISR: Coherencia temporal
};

class DebugOverlay {
public:
    DebugOverlay() = default;
    
    // Cambiar modo de visualización
    void setMode(VisualizationMode mode) { currentMode = mode; }
    VisualizationMode getMode() const { return currentMode; }
    
    // Ciclar entre modos (con teclas 1-7)
    void nextMode() {
        int mode = static_cast<int>(currentMode);
        mode = (mode + 1) % 7;
        currentMode = static_cast<VisualizationMode>(mode);
    }
    
    void prevMode() {
        int mode = static_cast<int>(currentMode);
        mode = (mode - 1 + 7) % 7;
        currentMode = static_cast<VisualizationMode>(mode);
    }
    
    // Toggle overlay de texto
    void toggleTextOverlay() { showTextOverlay = !showTextOverlay; }
    bool isTextOverlayVisible() const { return showTextOverlay; }
    
    // Obtener nombre del modo actual
    std::string getModeName() const {
        switch (currentMode) {
            case VisualizationMode::Normal: return "1. Normal (Phong Shading)";
            case VisualizationMode::Wireframe: return "2. Wireframe";
            case VisualizationMode::Normals: return "3. Normals (RGB)";
            case VisualizationMode::Depth: return "4. Depth Buffer";
            case VisualizationMode::ImportanceMap: return "5. ISR: Importance Map";
            case VisualizationMode::PixelSizing: return "6. ISR: Pixel Sizing";
            case VisualizationMode::TemporalCoherence: return "7. ISR: Temporal";
            default: return "Unknown";
        }
    }
    
    // Obtener descripción del modo
    std::string getModeDescription() const {
        switch (currentMode) {
            case VisualizationMode::Normal:
                return "Renderizado normal con iluminacion Phong (Ambient + Diffuse + Specular)";
            case VisualizationMode::Wireframe:
                return "Solo bordes del cubo (sin relleno)";
            case VisualizationMode::Normals:
                return "Normales como colores RGB (X=R, Y=G, Z=B)";
            case VisualizationMode::Depth:
                return "Profundidad en escala de grises (cerca=blanco, lejos=negro)";
            case VisualizationMode::ImportanceMap:
                return "Mapa de calor ISR (rojo=alta importancia, azul=baja)";
            case VisualizationMode::PixelSizing:
                return "Tamano de pixeles adaptativos (grande=bajo detalle, pequeno=alto)";
            case VisualizationMode::TemporalCoherence:
                return "Suavizado temporal entre frames (reduce flickering)";
            default:
                return "";
        }
    }
    
    // Stats para mostrar
    struct Stats {
        int fps = 0;
        float rotation = 0.0f;
        int vertices = 24;
        int triangles = 12;
        glm::vec3 cameraPos = glm::vec3(3.0f, 3.0f, 3.0f);
        float frameTime = 0.0f;
    };
    
    void updateStats(const Stats& stats) { currentStats = stats; }
    const Stats& getStats() const { return currentStats; }
    
private:
    VisualizationMode currentMode = VisualizationMode::Normal;
    bool showTextOverlay = true;
    Stats currentStats;
};

} // namespace debug
