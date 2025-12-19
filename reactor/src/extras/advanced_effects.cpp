#include "reactor/extras/advanced_effects.hpp"
#include "reactor/memory_allocator.hpp"
#include <iostream>

namespace reactor {

VolumetricLighting::VolumetricLighting(std::shared_ptr<MemoryAllocator> allocator) {
    std::cout << "[VolumetricLighting] Created" << std::endl;
}

void VolumetricLighting::render() {
    std::cout << "[VolumetricLighting] Rendering (density: " << density << ")" << std::endl;
}

ScreenSpaceReflections::ScreenSpaceReflections(std::shared_ptr<MemoryAllocator> allocator) {
    std::cout << "[ScreenSpaceReflections] Created" << std::endl;
}

void ScreenSpaceReflections::render() {
    std::cout << "[ScreenSpaceReflections] Rendering (steps: " << maxSteps << ")" << std::endl;
}

MotionBlur::MotionBlur(std::shared_ptr<MemoryAllocator> allocator) {
    std::cout << "[MotionBlur] Created" << std::endl;
}

void MotionBlur::render() {
    std::cout << "[MotionBlur] Rendering (samples: " << samples << ")" << std::endl;
}

DepthOfField::DepthOfField(std::shared_ptr<MemoryAllocator> allocator) {
    std::cout << "[DepthOfField] Created" << std::endl;
}

void DepthOfField::render() {
    std::cout << "[DepthOfField] Rendering (focal: " << focalDistance << ")" << std::endl;
}

} // namespace reactor
