#include "reactor/extras/compute.hpp"
#include "reactor/memory_allocator.hpp"
#include "reactor/buffer.hpp"
#include <iostream>

namespace reactor {

ComputeShader::ComputeShader(std::shared_ptr<MemoryAllocator> allocator, const std::string& shaderPath)
    : allocator(allocator), shaderPath(shaderPath) {
    std::cout << "[ComputeShader] Created: " << shaderPath << std::endl;
}

ComputeShader::~ComputeShader() = default;

void ComputeShader::setBuffer(uint32_t binding, Buffer* buffer) {
    std::cout << "[ComputeShader] Set buffer at binding " << binding << std::endl;
}

void ComputeShader::setUniform(uint32_t binding, const void* data, size_t size) {
    std::cout << "[ComputeShader] Set uniform at binding " << binding << " (" << size << " bytes)" << std::endl;
}

void ComputeShader::dispatch(uint32_t x, uint32_t y, uint32_t z) {
    std::cout << "[ComputeShader] Dispatch (" << x << ", " << y << ", " << z << ")" << std::endl;
}

ComputeShader ComputeShader::particleUpdate(std::shared_ptr<MemoryAllocator> allocator) {
    std::cout << "[ComputeShader] Created particle update preset" << std::endl;
    return ComputeShader(allocator, "particle_update.comp.spv");
}

ComputeShader ComputeShader::imageProcess(std::shared_ptr<MemoryAllocator> allocator) {
    std::cout << "[ComputeShader] Created image process preset" << std::endl;
    return ComputeShader(allocator, "image_process.comp.spv");
}

void ComputeHelper::fillBuffer(Buffer* buffer, float value) {
    std::cout << "[ComputeHelper] Fill buffer with value: " << value << std::endl;
}

void ComputeHelper::copyBuffer(Buffer* src, Buffer* dst) {
    std::cout << "[ComputeHelper] Copy buffer" << std::endl;
}

void ComputeHelper::prefixSum(Buffer* buffer) {
    std::cout << "[ComputeHelper] Prefix sum" << std::endl;
}

} // namespace reactor
