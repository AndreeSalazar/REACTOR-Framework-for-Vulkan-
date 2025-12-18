#pragma once
#include "reactor/shader.hpp"
#include <vulkan/vulkan.h>
#include <memory>
#include <vector>

namespace reactor {

enum class Topology {
    PointList = VK_PRIMITIVE_TOPOLOGY_POINT_LIST,
    LineList = VK_PRIMITIVE_TOPOLOGY_LINE_LIST,
    LineStrip = VK_PRIMITIVE_TOPOLOGY_LINE_STRIP,
    TriangleList = VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST,
    TriangleStrip = VK_PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP,
    TriangleFan = VK_PRIMITIVE_TOPOLOGY_TRIANGLE_FAN
};

enum class PolygonMode {
    Fill = VK_POLYGON_MODE_FILL,
    Line = VK_POLYGON_MODE_LINE,
    Point = VK_POLYGON_MODE_POINT
};

enum class CullMode {
    None = VK_CULL_MODE_NONE,
    Front = VK_CULL_MODE_FRONT_BIT,
    Back = VK_CULL_MODE_BACK_BIT,
    FrontAndBack = VK_CULL_MODE_FRONT_AND_BACK
};

enum class BlendMode {
    None,
    Alpha,
    Additive,
    Multiply
};

struct VertexInputBinding {
    uint32_t binding;
    uint32_t stride;
    VkVertexInputRate inputRate{VK_VERTEX_INPUT_RATE_VERTEX};
};

struct VertexInputAttribute {
    uint32_t location;
    uint32_t binding;
    VkFormat format;
    uint32_t offset;
};

class GraphicsPipeline {
public:
    GraphicsPipeline(VkDevice device, VkPipeline pipeline, VkPipelineLayout layout);
    ~GraphicsPipeline();

    GraphicsPipeline(const GraphicsPipeline&) = delete;
    GraphicsPipeline& operator=(const GraphicsPipeline&) = delete;
    GraphicsPipeline(GraphicsPipeline&& other) noexcept;
    GraphicsPipeline& operator=(GraphicsPipeline&& other) noexcept;

    VkPipeline handle() const { return pipeline; }
    VkPipelineLayout layout() const { return pipelineLayout; }

    class Builder {
    public:
        Builder(VkDevice device, VkRenderPass renderPass);
        
        Builder& shader(std::shared_ptr<Shader> shader);
        Builder& vertexInput(const std::vector<VertexInputBinding>& bindings,
                           const std::vector<VertexInputAttribute>& attributes);
        Builder& topology(Topology topo);
        Builder& polygonMode(PolygonMode mode);
        Builder& cullMode(CullMode mode);
        Builder& depthTest(bool enable);
        Builder& depthWrite(bool enable);
        Builder& blending(BlendMode mode);
        Builder& viewport(float width, float height);
        Builder& descriptorSetLayouts(const std::vector<VkDescriptorSetLayout>& layouts);
        Builder& pushConstantRanges(const std::vector<VkPushConstantRange>& ranges);
        
        GraphicsPipeline build();

    private:
        VkDevice dev;
        VkRenderPass renderPass;
        std::vector<std::shared_ptr<Shader>> shaders;
        std::vector<VertexInputBinding> vertexBindings;
        std::vector<VertexInputAttribute> vertexAttributes;
        Topology primTopology{Topology::TriangleList};
        PolygonMode polyMode{PolygonMode::Fill};
        CullMode cullMd{CullMode::Back};
        bool enableDepthTest{false};
        bool enableDepthWrite{false};
        BlendMode blendMode{BlendMode::None};
        float viewportWidth{800.0f};
        float viewportHeight{600.0f};
        std::vector<VkDescriptorSetLayout> descLayouts;
        std::vector<VkPushConstantRange> pushRanges;
    };

    static Builder create(VkDevice device, VkRenderPass renderPass);

private:
    VkDevice device;
    VkPipeline pipeline{VK_NULL_HANDLE};
    VkPipelineLayout pipelineLayout{VK_NULL_HANDLE};
};

class ComputePipeline {
public:
    ComputePipeline(VkDevice device, VkPipeline pipeline, VkPipelineLayout layout);
    ~ComputePipeline();

    ComputePipeline(const ComputePipeline&) = delete;
    ComputePipeline& operator=(const ComputePipeline&) = delete;
    ComputePipeline(ComputePipeline&& other) noexcept;
    ComputePipeline& operator=(ComputePipeline&& other) noexcept;

    VkPipeline handle() const { return pipeline; }
    VkPipelineLayout layout() const { return pipelineLayout; }

    class Builder {
    public:
        Builder(VkDevice device);
        
        Builder& shader(std::shared_ptr<Shader> shader);
        Builder& descriptorSetLayouts(const std::vector<VkDescriptorSetLayout>& layouts);
        Builder& pushConstantRanges(const std::vector<VkPushConstantRange>& ranges);
        
        ComputePipeline build();

    private:
        VkDevice dev;
        std::shared_ptr<Shader> computeShader;
        std::vector<VkDescriptorSetLayout> descLayouts;
        std::vector<VkPushConstantRange> pushRanges;
    };

    static Builder create(VkDevice device);

private:
    VkDevice device;
    VkPipeline pipeline{VK_NULL_HANDLE};
    VkPipelineLayout pipelineLayout{VK_NULL_HANDLE};
};

}
