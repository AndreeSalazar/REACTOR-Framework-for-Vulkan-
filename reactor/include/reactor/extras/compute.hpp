#pragma once
#include "../math.hpp"
#include <memory>
#include <string>

namespace reactor {

// Forward declarations
class MemoryAllocator;
class Buffer;

/**
 * @brief ComputeShader - Compute shader helper
 * 
 * Uso simple:
 * ComputeShader compute(allocator, "shader.comp.spv");
 * compute.setBuffer(0, inputBuffer);
 * compute.setBuffer(1, outputBuffer);
 * compute.dispatch(256, 1, 1);
 */
class ComputeShader {
public:
    ComputeShader(std::shared_ptr<MemoryAllocator> allocator, const std::string& shaderPath);
    ~ComputeShader();
    
    /**
     * @brief Bind buffers
     */
    void setBuffer(uint32_t binding, Buffer* buffer);
    void setUniform(uint32_t binding, const void* data, size_t size);
    
    /**
     * @brief Dispatch
     */
    void dispatch(uint32_t x, uint32_t y = 1, uint32_t z = 1);
    
    /**
     * @brief Helpers
     */
    static ComputeShader particleUpdate(std::shared_ptr<MemoryAllocator> allocator);
    static ComputeShader imageProcess(std::shared_ptr<MemoryAllocator> allocator);

private:
    std::shared_ptr<MemoryAllocator> allocator;
    std::string shaderPath;
};

/**
 * @brief ComputeHelper - Helpers para operaciones compute comunes
 */
class ComputeHelper {
public:
    /**
     * @brief Operaciones comunes
     */
    static void fillBuffer(Buffer* buffer, float value);
    static void copyBuffer(Buffer* src, Buffer* dst);
    static void prefixSum(Buffer* buffer);
};

} // namespace reactor
