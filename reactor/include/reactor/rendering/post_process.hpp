#pragma once
#include "../math.hpp"
#include <vulkan/vulkan.h>
#include <string>
#include <vector>
#include <memory>
#include <functional>

namespace reactor {

// Forward declarations
class VulkanContext;

/**
 * @brief PostProcessEffect - Efecto de post-procesamiento base
 * 
 * Clase base para efectos como bloom, blur, tonemap, etc.
 * Cada efecto puede tener su propio compute shader o fragment shader.
 */
class PostProcessEffect {
public:
    virtual ~PostProcessEffect() = default;
    
    virtual void apply() = 0;
    virtual const char* getName() const = 0;
    
    void setEnabled(bool enabled) { isEnabled = enabled; }
    bool enabled() const { return isEnabled; }
    
    // Para efectos que necesitan parámetros en shaders
    virtual void updateUniforms() {}

protected:
    bool isEnabled{true};
};

/**
 * @brief PostProcessStack - Stack completo de post-procesamiento
 * 
 * Maneja una cadena de efectos de post-procesamiento.
 * Soporta ping-pong rendering entre framebuffers.
 * 
 * Uso:
 * ```cpp
 * PostProcessStack stack(ctx);
 * stack.addEffect<BloomEffect>(1.0f, 1.5f);
 * stack.addEffect<TonemapEffect>(TonemapEffect::Mode::ACES, 1.2f);
 * stack.addEffect<VignetteEffect>(0.5f);
 * 
 * // En render loop:
 * stack.process(inputImage, outputImage);
 * ```
 */
class PostProcessStack {
public:
    PostProcessStack() = default;
    explicit PostProcessStack(VulkanContext& ctx);
    ~PostProcessStack();
    
    template<typename T, typename... Args>
    T* addEffect(Args&&... args) {
        auto effect = std::make_unique<T>(std::forward<Args>(args)...);
        T* ptr = effect.get();
        effects.push_back(std::move(effect));
        return ptr;
    }
    
    void apply();
    void clear() { effects.clear(); }
    size_t count() const { return effects.size(); }
    
    // Obtener efecto por tipo
    template<typename T>
    T* getEffect() {
        for (auto& effect : effects) {
            if (auto* ptr = dynamic_cast<T*>(effect.get())) {
                return ptr;
            }
        }
        return nullptr;
    }
    
    // Habilitar/deshabilitar todos
    void enableAll(bool enabled);
    
    // Stats
    struct Stats {
        size_t totalEffects = 0;
        size_t enabledEffects = 0;
        float totalTimeMs = 0.0f;
    };
    Stats getStats() const;

private:
    VulkanContext* ctx{nullptr};
    std::vector<std::unique_ptr<PostProcessEffect>> effects;
};

// ==================== EFECTOS PREDEFINIDOS ====================

/**
 * @brief BloomEffect - Efecto de brillo/resplandor
 */
class BloomEffect : public PostProcessEffect {
public:
    BloomEffect(float threshold = 1.0f, float intensity = 1.5f);
    
    void apply() override;
    const char* getName() const override { return "Bloom"; }
    
    // Configuración
    float threshold{1.0f};    // Umbral de brillo para bloom
    float intensity{1.5f};    // Intensidad del efecto
    int blurPasses{4};        // Número de pasadas de blur
    float radius{1.0f};       // Radio del blur
    
    // Para shaders
    struct BloomParams {
        float threshold;
        float intensity;
        float radius;
        int blurPasses;
    };
    BloomParams getParams() const;
};

/**
 * @brief TonemapEffect - Mapeo de tonos HDR a LDR
 */
class TonemapEffect : public PostProcessEffect {
public:
    enum class Mode {
        Reinhard,      // Simple, buen balance
        ACES,          // Cinematográfico, usado en películas
        Uncharted2,    // Usado en el juego Uncharted 2
        Filmic,        // Estilo película
        Linear         // Sin tonemapping (para debug)
    };
    
    TonemapEffect(Mode mode = Mode::ACES, float exposure = 1.0f);
    
    void apply() override;
    const char* getName() const override { return "Tonemap"; }
    
    Mode mode{Mode::ACES};
    float exposure{1.0f};
    float gamma{2.2f};
    float whitePoint{11.2f};  // Para Uncharted2
    
    // Para shaders
    struct TonemapParams {
        int mode;
        float exposure;
        float gamma;
        float whitePoint;
    };
    TonemapParams getParams() const;
    
    // GLSL functions para cada modo
    static const char* getGLSLFunction(Mode mode);
};

/**
 * @brief BlurEffect - Efecto de desenfoque gaussiano
 */
class BlurEffect : public PostProcessEffect {
public:
    BlurEffect(int radius = 5);
    
    void apply() override;
    const char* getName() const override { return "Blur"; }
    
    int radius{5};
    float sigma{2.0f};  // Sigma para distribución gaussiana
    bool horizontal{true};  // Para blur separable
    
    // Kernel gaussiano precalculado
    std::vector<float> getKernel() const;
};

/**
 * @brief VignetteEffect - Oscurecimiento de bordes
 */
class VignetteEffect : public PostProcessEffect {
public:
    VignetteEffect(float intensity = 0.5f, float radius = 0.75f);
    
    void apply() override;
    const char* getName() const override { return "Vignette"; }
    
    float intensity{0.5f};
    float radius{0.75f};
    float softness{0.45f};
    Vec3 color{0.0f, 0.0f, 0.0f};  // Color del vignette (negro por defecto)
};

/**
 * @brief ChromaticAberrationEffect - Aberración cromática
 */
class ChromaticAberrationEffect : public PostProcessEffect {
public:
    ChromaticAberrationEffect(float intensity = 0.005f);
    
    void apply() override;
    const char* getName() const override { return "ChromaticAberration"; }
    
    float intensity{0.005f};
    Vec2 direction{1.0f, 0.0f};
};

/**
 * @brief FilmGrainEffect - Grano de película
 */
class FilmGrainEffect : public PostProcessEffect {
public:
    FilmGrainEffect(float intensity = 0.1f);
    
    void apply() override;
    const char* getName() const override { return "FilmGrain"; }
    
    float intensity{0.1f};
    float speed{1.0f};  // Velocidad de animación del grano
};

/**
 * @brief FXAAEffect - Fast Approximate Anti-Aliasing
 */
class FXAAEffect : public PostProcessEffect {
public:
    FXAAEffect();
    
    void apply() override;
    const char* getName() const override { return "FXAA"; }
    
    float subpixelQuality{0.75f};
    float edgeThreshold{0.166f};
    float edgeThresholdMin{0.0833f};
};

/**
 * @brief SSAOEffect - Screen Space Ambient Occlusion
 */
class SSAOEffect : public PostProcessEffect {
public:
    SSAOEffect(int samples = 16, float radius = 0.5f);
    
    void apply() override;
    const char* getName() const override { return "SSAO"; }
    
    int samples{16};
    float radius{0.5f};
    float bias{0.025f};
    float intensity{1.0f};
};

} // namespace reactor
