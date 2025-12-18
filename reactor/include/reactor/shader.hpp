#pragma once
#include <vulkan/vulkan.h>
#include <string>
#include <vector>
#include <memory>

namespace reactor {

enum class ShaderStage {
    Vertex = VK_SHADER_STAGE_VERTEX_BIT,
    Fragment = VK_SHADER_STAGE_FRAGMENT_BIT,
    Compute = VK_SHADER_STAGE_COMPUTE_BIT,
    Geometry = VK_SHADER_STAGE_GEOMETRY_BIT,
    TessControl = VK_SHADER_STAGE_TESSELLATION_CONTROL_BIT,
    TessEval = VK_SHADER_STAGE_TESSELLATION_EVALUATION_BIT
};

class Shader {
public:
    Shader(VkDevice device, const std::vector<uint32_t>& spirv, ShaderStage stage);
    Shader(VkDevice device, const std::string& filepath, ShaderStage stage);
    ~Shader();

    Shader(const Shader&) = delete;
    Shader& operator=(const Shader&) = delete;
    Shader(Shader&& other) noexcept;
    Shader& operator=(Shader&& other) noexcept;

    VkShaderModule module() const { return shaderModule; }
    ShaderStage stage() const { return shaderStage; }
    VkPipelineShaderStageCreateInfo getStageInfo() const;

    static std::vector<uint32_t> loadSPIRV(const std::string& filepath);

private:
    VkDevice device;
    VkShaderModule shaderModule{VK_NULL_HANDLE};
    ShaderStage shaderStage;
};

}
