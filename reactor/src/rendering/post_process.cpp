#include "reactor/rendering/post_process.hpp"
#include <iostream>

namespace reactor {

void PostProcessStack::apply() {
    for (auto& effect : effects) {
        if (effect->enabled()) {
            effect->apply();
        }
    }
}

void BloomEffect::apply() {
    std::cout << "[PostProcess] Applying Bloom (threshold: " << threshold << ", intensity: " << intensity << ")" << std::endl;
}

void TonemapEffect::apply() {
    const char* modeName = "Unknown";
    switch (mode) {
        case Mode::Reinhard: modeName = "Reinhard"; break;
        case Mode::ACES: modeName = "ACES"; break;
        case Mode::Uncharted2: modeName = "Uncharted2"; break;
    }
    std::cout << "[PostProcess] Applying Tonemap (" << modeName << ", exposure: " << exposure << ")" << std::endl;
}

void BlurEffect::apply() {
    std::cout << "[PostProcess] Applying Blur (radius: " << radius << ")" << std::endl;
}

} // namespace reactor
