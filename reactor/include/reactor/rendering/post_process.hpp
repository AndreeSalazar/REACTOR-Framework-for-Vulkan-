#pragma once
#include <string>
#include <vector>
#include <memory>

namespace reactor {

// Forward declarations
class MemoryAllocator;

/**
 * @brief PostProcessEffect - Efecto de post-procesamiento
 * 
 * Base class para efectos como bloom, blur, etc.
 */
class PostProcessEffect {
public:
    virtual ~PostProcessEffect() = default;
    
    /**
     * @brief Aplicar efecto
     */
    virtual void apply() = 0;
    
    /**
     * @brief Activar/Desactivar
     */
    void setEnabled(bool enabled) { isEnabled = enabled; }
    bool enabled() const { return isEnabled; }

protected:
    bool isEnabled{true};
};

/**
 * @brief PostProcessStack - Stack de efectos
 * 
 * Uso simple:
 * PostProcessStack stack;
 * stack.addEffect<BloomEffect>();
 * stack.addEffect<TonemapEffect>();
 * stack.apply();
 */
class PostProcessStack {
public:
    PostProcessStack() = default;
    
    /**
     * @brief Agregar efecto
     */
    template<typename T, typename... Args>
    T* addEffect(Args&&... args);
    
    /**
     * @brief Aplicar todos los efectos
     */
    void apply();
    
    /**
     * @brief Limpiar
     */
    void clear() { effects.clear(); }
    
    /**
     * @brief Stats
     */
    size_t count() const { return effects.size(); }

private:
    std::vector<std::unique_ptr<PostProcessEffect>> effects;
};

/**
 * @brief Efectos predefinidos
 */
class BloomEffect : public PostProcessEffect {
public:
    float threshold{1.0f};
    float intensity{1.0f};
    
    void apply() override;
};

class TonemapEffect : public PostProcessEffect {
public:
    enum class Mode {
        Reinhard,
        ACES,
        Uncharted2
    };
    
    Mode mode{Mode::ACES};
    float exposure{1.0f};
    
    void apply() override;
};

class BlurEffect : public PostProcessEffect {
public:
    int radius{5};
    
    void apply() override;
};

} // namespace reactor
