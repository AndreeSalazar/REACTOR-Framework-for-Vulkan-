#include "reactor/rendering/post_process.hpp"
#include "reactor/vulkan_context.hpp"
#include <iostream>
#include <cmath>

namespace reactor {

// ==================== PostProcessStack ====================

PostProcessStack::PostProcessStack(VulkanContext& ctx) : ctx(&ctx) {
    std::cout << "[PostProcessStack] Inicializado" << std::endl;
}

PostProcessStack::~PostProcessStack() {
    effects.clear();
}

void PostProcessStack::apply() {
    for (auto& effect : effects) {
        if (effect->enabled()) {
            effect->apply();
        }
    }
}

void PostProcessStack::enableAll(bool enabled) {
    for (auto& effect : effects) {
        effect->setEnabled(enabled);
    }
}

PostProcessStack::Stats PostProcessStack::getStats() const {
    Stats stats;
    stats.totalEffects = effects.size();
    for (const auto& effect : effects) {
        if (effect->enabled()) {
            stats.enabledEffects++;
        }
    }
    return stats;
}

// ==================== BloomEffect ====================

BloomEffect::BloomEffect(float threshold, float intensity)
    : threshold(threshold), intensity(intensity) {
}

void BloomEffect::apply() {
    std::cout << "[PostProcess] Bloom (threshold: " << threshold 
              << ", intensity: " << intensity 
              << ", passes: " << blurPasses << ")" << std::endl;
}

BloomEffect::BloomParams BloomEffect::getParams() const {
    return {threshold, intensity, radius, blurPasses};
}

// ==================== TonemapEffect ====================

TonemapEffect::TonemapEffect(Mode mode, float exposure)
    : mode(mode), exposure(exposure) {
}

void TonemapEffect::apply() {
    const char* modeName = "Unknown";
    switch (mode) {
        case Mode::Reinhard: modeName = "Reinhard"; break;
        case Mode::ACES: modeName = "ACES"; break;
        case Mode::Uncharted2: modeName = "Uncharted2"; break;
        case Mode::Filmic: modeName = "Filmic"; break;
        case Mode::Linear: modeName = "Linear"; break;
    }
    std::cout << "[PostProcess] Tonemap (" << modeName 
              << ", exposure: " << exposure 
              << ", gamma: " << gamma << ")" << std::endl;
}

TonemapEffect::TonemapParams TonemapEffect::getParams() const {
    return {static_cast<int>(mode), exposure, gamma, whitePoint};
}

const char* TonemapEffect::getGLSLFunction(Mode mode) {
    switch (mode) {
        case Mode::Reinhard:
            return R"(
vec3 tonemapReinhard(vec3 color) {
    return color / (color + vec3(1.0));
}
)";
        case Mode::ACES:
            return R"(
vec3 tonemapACES(vec3 color) {
    const float a = 2.51;
    const float b = 0.03;
    const float c = 2.43;
    const float d = 0.59;
    const float e = 0.14;
    return clamp((color * (a * color + b)) / (color * (c * color + d) + e), 0.0, 1.0);
}
)";
        case Mode::Uncharted2:
            return R"(
vec3 tonemapUncharted2(vec3 x) {
    const float A = 0.15;
    const float B = 0.50;
    const float C = 0.10;
    const float D = 0.20;
    const float E = 0.02;
    const float F = 0.30;
    return ((x*(A*x+C*B)+D*E)/(x*(A*x+B)+D*F))-E/F;
}
)";
        case Mode::Filmic:
            return R"(
vec3 tonemapFilmic(vec3 color) {
    color = max(vec3(0.0), color - 0.004);
    return (color * (6.2 * color + 0.5)) / (color * (6.2 * color + 1.7) + 0.06);
}
)";
        default:
            return "vec3 tonemapLinear(vec3 color) { return color; }";
    }
}

// ==================== BlurEffect ====================

BlurEffect::BlurEffect(int radius) : radius(radius) {
    sigma = static_cast<float>(radius) / 2.0f;
}

void BlurEffect::apply() {
    std::cout << "[PostProcess] Blur (radius: " << radius 
              << ", sigma: " << sigma << ")" << std::endl;
}

std::vector<float> BlurEffect::getKernel() const {
    std::vector<float> kernel(radius * 2 + 1);
    float sum = 0.0f;
    
    for (int i = -radius; i <= radius; i++) {
        float x = static_cast<float>(i);
        float g = std::exp(-(x * x) / (2.0f * sigma * sigma));
        kernel[i + radius] = g;
        sum += g;
    }
    
    // Normalizar
    for (float& k : kernel) {
        k /= sum;
    }
    
    return kernel;
}

// ==================== VignetteEffect ====================

VignetteEffect::VignetteEffect(float intensity, float radius)
    : intensity(intensity), radius(radius) {
}

void VignetteEffect::apply() {
    std::cout << "[PostProcess] Vignette (intensity: " << intensity 
              << ", radius: " << radius << ")" << std::endl;
}

// ==================== ChromaticAberrationEffect ====================

ChromaticAberrationEffect::ChromaticAberrationEffect(float intensity)
    : intensity(intensity) {
}

void ChromaticAberrationEffect::apply() {
    std::cout << "[PostProcess] ChromaticAberration (intensity: " << intensity << ")" << std::endl;
}

// ==================== FilmGrainEffect ====================

FilmGrainEffect::FilmGrainEffect(float intensity)
    : intensity(intensity) {
}

void FilmGrainEffect::apply() {
    std::cout << "[PostProcess] FilmGrain (intensity: " << intensity << ")" << std::endl;
}

// ==================== FXAAEffect ====================

FXAAEffect::FXAAEffect() = default;

void FXAAEffect::apply() {
    std::cout << "[PostProcess] FXAA (quality: " << subpixelQuality << ")" << std::endl;
}

// ==================== SSAOEffect ====================

SSAOEffect::SSAOEffect(int samples, float radius)
    : samples(samples), radius(radius) {
}

void SSAOEffect::apply() {
    std::cout << "[PostProcess] SSAO (samples: " << samples 
              << ", radius: " << radius << ")" << std::endl;
}

} // namespace reactor
