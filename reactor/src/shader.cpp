#include "reactor/shader.hpp"
#include <fstream>
#include <stdexcept>

namespace reactor {

std::vector<uint32_t> Shader::loadSPIRV(const std::string& filepath) {
    std::ifstream file(filepath, std::ios::ate | std::ios::binary);
    
    if (!file.is_open()) {
        throw std::runtime_error("failed to open shader file: " + filepath);
    }
    
    size_t fileSize = static_cast<size_t>(file.tellg());
    std::vector<uint32_t> buffer(fileSize / sizeof(uint32_t));
    
    file.seekg(0);
    file.read(reinterpret_cast<char*>(buffer.data()), fileSize);
    file.close();
    
    return buffer;
}

Shader::Shader(VkDevice device, const std::vector<uint32_t>& spirv, ShaderStage stage)
    : device(device), shaderStage(stage) {
    
    VkShaderModuleCreateInfo createInfo{};
    createInfo.sType = VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO;
    createInfo.codeSize = spirv.size() * sizeof(uint32_t);
    createInfo.pCode = spirv.data();
    
    if (vkCreateShaderModule(device, &createInfo, nullptr, &shaderModule) != VK_SUCCESS) {
        throw std::runtime_error("failed to create shader module");
    }
}

Shader::Shader(VkDevice device, const std::string& filepath, ShaderStage stage)
    : Shader(device, loadSPIRV(filepath), stage) {}

Shader::~Shader() {
    if (shaderModule != VK_NULL_HANDLE) {
        vkDestroyShaderModule(device, shaderModule, nullptr);
    }
}

Shader::Shader(Shader&& other) noexcept
    : device(other.device)
    , shaderModule(other.shaderModule)
    , shaderStage(other.shaderStage) {
    other.shaderModule = VK_NULL_HANDLE;
}

Shader& Shader::operator=(Shader&& other) noexcept {
    if (this != &other) {
        if (shaderModule != VK_NULL_HANDLE) {
            vkDestroyShaderModule(device, shaderModule, nullptr);
        }
        device = other.device;
        shaderModule = other.shaderModule;
        shaderStage = other.shaderStage;
        other.shaderModule = VK_NULL_HANDLE;
    }
    return *this;
}

VkPipelineShaderStageCreateInfo Shader::getStageInfo() const {
    VkPipelineShaderStageCreateInfo stageInfo{};
    stageInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
    stageInfo.stage = static_cast<VkShaderStageFlagBits>(shaderStage);
    stageInfo.module = shaderModule;
    stageInfo.pName = "main";
    return stageInfo;
}

}
