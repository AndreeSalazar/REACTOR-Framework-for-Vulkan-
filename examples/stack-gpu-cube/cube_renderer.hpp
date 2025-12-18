#pragma once
#include "reactor/vulkan_context.hpp"
#include "reactor/buffer.hpp"
#include "reactor/pipeline.hpp"
#include "reactor/command_buffer.hpp"
#include "reactor/shader.hpp"
#include <glm/glm.hpp>
#include <vector>

namespace cube {

struct Vertex {
    glm::vec3 pos;
    glm::vec3 normal;
    glm::vec3 color;
};

class CubeRenderer {
public:
    CubeRenderer(reactor::VulkanContext& ctx, VkRenderPass renderPass, uint32_t width, uint32_t height);
    ~CubeRenderer() = default;

    void render(reactor::CommandBuffer& cmd, const glm::mat4& mvp, const glm::mat4& model);

private:
    reactor::VulkanContext& context;
    std::unique_ptr<reactor::Buffer> vertexBuffer;
    std::unique_ptr<reactor::Buffer> indexBuffer;
    std::unique_ptr<reactor::GraphicsPipeline> pipeline;
    
    void createBuffers();
    void createPipeline(VkRenderPass renderPass, uint32_t width, uint32_t height);
    
    static std::vector<Vertex> getCubeVertices();
    static std::vector<uint16_t> getCubeIndices();
};

} // namespace cube
