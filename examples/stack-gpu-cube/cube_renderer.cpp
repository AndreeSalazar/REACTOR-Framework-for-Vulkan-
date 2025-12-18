#include "cube_renderer.hpp"
#include <stdexcept>
#include <iostream>
#include <fstream>
#include <windows.h>

namespace cube {

CubeRenderer::CubeRenderer(reactor::VulkanContext& ctx, VkRenderPass renderPass, uint32_t width, uint32_t height)
    : context(ctx) {
    createBuffers();
    createPipeline(renderPass, width, height);
}

void CubeRenderer::createBuffers() {
    auto vertices = getCubeVertices();
    auto indices = getCubeIndices();
    
    // Crear vertex buffer
    vertexBuffer = std::make_unique<reactor::Buffer>(
        reactor::Buffer::create(context.allocator())
            .size(sizeof(Vertex) * vertices.size())
            .usage(reactor::BufferUsage::Vertex)
            .memoryType(reactor::MemoryType::HostVisible)
            .build()
    );
    vertexBuffer->upload(vertices.data(), sizeof(Vertex) * vertices.size());
    
    // Crear index buffer
    indexBuffer = std::make_unique<reactor::Buffer>(
        reactor::Buffer::create(context.allocator())
            .size(sizeof(uint16_t) * indices.size())
            .usage(reactor::BufferUsage::Index)
            .memoryType(reactor::MemoryType::HostVisible)
            .build()
    );
    indexBuffer->upload(indices.data(), sizeof(uint16_t) * indices.size());
    
    std::cout << "      ✓ Buffers creados (24 vértices, 36 índices)" << std::endl;
}

void CubeRenderer::createPipeline(VkRenderPass renderPass, uint32_t width, uint32_t height) {
    // Cargar shaders - intentar rutas relativas y absolutas
    std::string vertPath = "shaders/cube.vert.spv";
    std::string fragPath = "shaders/cube.frag.spv";
    
    // Si no existen en ruta relativa, intentar ruta absoluta basada en el ejecutable
    std::ifstream test(vertPath);
    if (!test.is_open()) {
        // Obtener ruta del ejecutable
        char exePath[MAX_PATH];
        GetModuleFileNameA(nullptr, exePath, MAX_PATH);
        std::string exeDir = std::string(exePath);
        size_t lastSlash = exeDir.find_last_of("\\/");
        if (lastSlash != std::string::npos) {
            exeDir = exeDir.substr(0, lastSlash + 1);
            vertPath = exeDir + "shaders/cube.vert.spv";
            fragPath = exeDir + "shaders/cube.frag.spv";
        }
    } else {
        test.close();
    }
    
    reactor::Shader vertShader(context.device(), vertPath, reactor::ShaderStage::Vertex);
    reactor::Shader fragShader(context.device(), fragPath, reactor::ShaderStage::Fragment);
    
    std::cout << "      ✓ Shaders cargados" << std::endl;
    
    // Vertex input
    std::vector<reactor::VertexInputBinding> bindings = {{
        .binding = 0,
        .stride = sizeof(Vertex),
        .inputRate = VK_VERTEX_INPUT_RATE_VERTEX
    }};
    
    std::vector<reactor::VertexInputAttribute> attributes = {
        {.location = 0, .binding = 0, .format = VK_FORMAT_R32G32B32_SFLOAT, .offset = offsetof(Vertex, pos)},
        {.location = 1, .binding = 0, .format = VK_FORMAT_R32G32B32_SFLOAT, .offset = offsetof(Vertex, normal)},
        {.location = 2, .binding = 0, .format = VK_FORMAT_R32G32B32_SFLOAT, .offset = offsetof(Vertex, color)}
    };
    
    // Push constant range (MVP + Model matrices + debug mode)
    VkPushConstantRange pushConstant{};
    pushConstant.stageFlags = VK_SHADER_STAGE_VERTEX_BIT | VK_SHADER_STAGE_FRAGMENT_BIT;
    pushConstant.offset = 0;
    pushConstant.size = sizeof(glm::mat4) * 2 + sizeof(int) + sizeof(float) * 3;  // MVP + Model + debugMode + padding
    
    // Crear pipeline
    pipeline = std::make_unique<reactor::GraphicsPipeline>(
        reactor::GraphicsPipeline::create(context.device(), renderPass)
            .shader(std::make_shared<reactor::Shader>(std::move(vertShader)))
            .shader(std::make_shared<reactor::Shader>(std::move(fragShader)))
            .vertexInput(bindings, attributes)
            .topology(reactor::Topology::TriangleList)
            .viewport(static_cast<float>(width), static_cast<float>(height))
            .cullMode(reactor::CullMode::Back)
            .depthTest(true)
            .pushConstantRanges({pushConstant})
            .build()
    );
    
    std::cout << "      ✓ Pipeline creado" << std::endl;
}

void CubeRenderer::render(reactor::CommandBuffer& cmd, const glm::mat4& mvp, const glm::mat4& model, int debugMode) {
    // Bind pipeline
    vkCmdBindPipeline(cmd.handle(), VK_PIPELINE_BIND_POINT_GRAPHICS, pipeline->handle());
    
    // Push constants (MVP + Model matrices + debug mode)
    struct PushConstants {
        glm::mat4 mvp;
        glm::mat4 model;
        int debugMode;
        float padding[3];
    } pushConstants;
    
    pushConstants.mvp = mvp;
    pushConstants.model = model;
    pushConstants.debugMode = debugMode;
    pushConstants.padding[0] = 0.0f;
    pushConstants.padding[1] = 0.0f;
    pushConstants.padding[2] = 0.0f;
    
    cmd.pushConstants(pipeline->layout(), VK_SHADER_STAGE_VERTEX_BIT | VK_SHADER_STAGE_FRAGMENT_BIT, 
                     0, sizeof(PushConstants), &pushConstants);
    
    // Bind vertex buffer
    std::vector<VkBuffer> buffers = {vertexBuffer->handle()};
    std::vector<VkDeviceSize> offsets = {0};
    cmd.bindVertexBuffers(0, buffers, offsets);
    
    // Bind index buffer
    cmd.bindIndexBuffer(indexBuffer->handle(), 0, VK_INDEX_TYPE_UINT16);
    
    // Draw
    cmd.drawIndexed(36, 1, 0, 0, 0);
}

std::vector<Vertex> CubeRenderer::getCubeVertices() {
    // Cada cara tiene 4 vértices con normales correctas para Phong shading
    return {
        // Front face (Z+) - Cyan/Teal como LunarG
        {{-0.5f, -0.5f,  0.5f}, {0.0f, 0.0f, 1.0f}, {0.0f, 0.8f, 0.8f}},
        {{ 0.5f, -0.5f,  0.5f}, {0.0f, 0.0f, 1.0f}, {0.0f, 0.8f, 0.8f}},
        {{ 0.5f,  0.5f,  0.5f}, {0.0f, 0.0f, 1.0f}, {0.0f, 0.9f, 0.9f}},
        {{-0.5f,  0.5f,  0.5f}, {0.0f, 0.0f, 1.0f}, {0.0f, 0.9f, 0.9f}},
        
        // Back face (Z-) - Dark gray
        {{ 0.5f, -0.5f, -0.5f}, {0.0f, 0.0f, -1.0f}, {0.3f, 0.3f, 0.3f}},
        {{-0.5f, -0.5f, -0.5f}, {0.0f, 0.0f, -1.0f}, {0.3f, 0.3f, 0.3f}},
        {{-0.5f,  0.5f, -0.5f}, {0.0f, 0.0f, -1.0f}, {0.4f, 0.4f, 0.4f}},
        {{ 0.5f,  0.5f, -0.5f}, {0.0f, 0.0f, -1.0f}, {0.4f, 0.4f, 0.4f}},
        
        // Left face (X-) - Medium gray
        {{-0.5f, -0.5f, -0.5f}, {-1.0f, 0.0f, 0.0f}, {0.5f, 0.5f, 0.5f}},
        {{-0.5f, -0.5f,  0.5f}, {-1.0f, 0.0f, 0.0f}, {0.5f, 0.5f, 0.5f}},
        {{-0.5f,  0.5f,  0.5f}, {-1.0f, 0.0f, 0.0f}, {0.6f, 0.6f, 0.6f}},
        {{-0.5f,  0.5f, -0.5f}, {-1.0f, 0.0f, 0.0f}, {0.6f, 0.6f, 0.6f}},
        
        // Right face (X+) - Light gray
        {{ 0.5f, -0.5f,  0.5f}, {1.0f, 0.0f, 0.0f}, {0.6f, 0.6f, 0.6f}},
        {{ 0.5f, -0.5f, -0.5f}, {1.0f, 0.0f, 0.0f}, {0.6f, 0.6f, 0.6f}},
        {{ 0.5f,  0.5f, -0.5f}, {1.0f, 0.0f, 0.0f}, {0.7f, 0.7f, 0.7f}},
        {{ 0.5f,  0.5f,  0.5f}, {1.0f, 0.0f, 0.0f}, {0.7f, 0.7f, 0.7f}},
        
        // Top face (Y+) - Light cyan
        {{-0.5f,  0.5f,  0.5f}, {0.0f, 1.0f, 0.0f}, {0.0f, 0.7f, 0.7f}},
        {{ 0.5f,  0.5f,  0.5f}, {0.0f, 1.0f, 0.0f}, {0.0f, 0.7f, 0.7f}},
        {{ 0.5f,  0.5f, -0.5f}, {0.0f, 1.0f, 0.0f}, {0.0f, 0.8f, 0.8f}},
        {{-0.5f,  0.5f, -0.5f}, {0.0f, 1.0f, 0.0f}, {0.0f, 0.8f, 0.8f}},
        
        // Bottom face (Y-) - Dark cyan
        {{-0.5f, -0.5f, -0.5f}, {0.0f, -1.0f, 0.0f}, {0.0f, 0.5f, 0.5f}},
        {{ 0.5f, -0.5f, -0.5f}, {0.0f, -1.0f, 0.0f}, {0.0f, 0.5f, 0.5f}},
        {{ 0.5f, -0.5f,  0.5f}, {0.0f, -1.0f, 0.0f}, {0.0f, 0.6f, 0.6f}},
        {{-0.5f, -0.5f,  0.5f}, {0.0f, -1.0f, 0.0f}, {0.0f, 0.6f, 0.6f}}
    };
}

std::vector<uint16_t> CubeRenderer::getCubeIndices() {
    return {
        // Front face (0-3)
        0, 1, 2, 2, 3, 0,
        // Back face (4-7)
        4, 5, 6, 6, 7, 4,
        // Left face (8-11)
        8, 9, 10, 10, 11, 8,
        // Right face (12-15)
        12, 13, 14, 14, 15, 12,
        // Top face (16-19)
        16, 17, 18, 18, 19, 16,
        // Bottom face (20-23)
        20, 21, 22, 22, 23, 20
    };
}

} // namespace cube
