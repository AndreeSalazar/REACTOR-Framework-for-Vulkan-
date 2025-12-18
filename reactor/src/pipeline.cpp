#include "reactor/pipeline.hpp"
#include <stdexcept>
#include <array>

namespace reactor {

GraphicsPipeline::GraphicsPipeline(VkDevice device, VkPipeline pipeline, VkPipelineLayout layout)
    : device(device), pipeline(pipeline), pipelineLayout(layout) {}

GraphicsPipeline::~GraphicsPipeline() {
    if (pipeline != VK_NULL_HANDLE) {
        vkDestroyPipeline(device, pipeline, nullptr);
    }
    if (pipelineLayout != VK_NULL_HANDLE) {
        vkDestroyPipelineLayout(device, pipelineLayout, nullptr);
    }
}

GraphicsPipeline::GraphicsPipeline(GraphicsPipeline&& other) noexcept
    : device(other.device), pipeline(other.pipeline), pipelineLayout(other.pipelineLayout) {
    other.pipeline = VK_NULL_HANDLE;
    other.pipelineLayout = VK_NULL_HANDLE;
}

GraphicsPipeline& GraphicsPipeline::operator=(GraphicsPipeline&& other) noexcept {
    if (this != &other) {
        if (pipeline != VK_NULL_HANDLE) {
            vkDestroyPipeline(device, pipeline, nullptr);
        }
        if (pipelineLayout != VK_NULL_HANDLE) {
            vkDestroyPipelineLayout(device, pipelineLayout, nullptr);
        }
        device = other.device;
        pipeline = other.pipeline;
        pipelineLayout = other.pipelineLayout;
        other.pipeline = VK_NULL_HANDLE;
        other.pipelineLayout = VK_NULL_HANDLE;
    }
    return *this;
}

GraphicsPipeline::Builder::Builder(VkDevice device, VkRenderPass renderPass)
    : dev(device), renderPass(renderPass) {}

GraphicsPipeline::Builder& GraphicsPipeline::Builder::shader(std::shared_ptr<Shader> shader) {
    shaders.push_back(shader);
    return *this;
}

GraphicsPipeline::Builder& GraphicsPipeline::Builder::vertexInput(
    const std::vector<VertexInputBinding>& bindings,
    const std::vector<VertexInputAttribute>& attributes) {
    vertexBindings = bindings;
    vertexAttributes = attributes;
    return *this;
}

GraphicsPipeline::Builder& GraphicsPipeline::Builder::topology(Topology topo) {
    primTopology = topo;
    return *this;
}

GraphicsPipeline::Builder& GraphicsPipeline::Builder::polygonMode(PolygonMode mode) {
    polyMode = mode;
    return *this;
}

GraphicsPipeline::Builder& GraphicsPipeline::Builder::cullMode(CullMode mode) {
    cullMd = mode;
    return *this;
}

GraphicsPipeline::Builder& GraphicsPipeline::Builder::depthTest(bool enable) {
    enableDepthTest = enable;
    return *this;
}

GraphicsPipeline::Builder& GraphicsPipeline::Builder::depthWrite(bool enable) {
    enableDepthWrite = enable;
    return *this;
}

GraphicsPipeline::Builder& GraphicsPipeline::Builder::blending(BlendMode mode) {
    blendMode = mode;
    return *this;
}

GraphicsPipeline::Builder& GraphicsPipeline::Builder::viewport(float width, float height) {
    viewportWidth = width;
    viewportHeight = height;
    return *this;
}

GraphicsPipeline::Builder& GraphicsPipeline::Builder::descriptorSetLayouts(
    const std::vector<VkDescriptorSetLayout>& layouts) {
    descLayouts = layouts;
    return *this;
}

GraphicsPipeline::Builder& GraphicsPipeline::Builder::pushConstantRanges(
    const std::vector<VkPushConstantRange>& ranges) {
    pushRanges = ranges;
    return *this;
}

GraphicsPipeline GraphicsPipeline::Builder::build() {
    if (shaders.empty()) {
        throw std::runtime_error("at least one shader is required");
    }
    
    std::vector<VkPipelineShaderStageCreateInfo> shaderStages;
    for (const auto& shader : shaders) {
        shaderStages.push_back(shader->getStageInfo());
    }
    
    std::vector<VkVertexInputBindingDescription> bindingDescs;
    for (const auto& binding : vertexBindings) {
        VkVertexInputBindingDescription desc{};
        desc.binding = binding.binding;
        desc.stride = binding.stride;
        desc.inputRate = binding.inputRate;
        bindingDescs.push_back(desc);
    }
    
    std::vector<VkVertexInputAttributeDescription> attributeDescs;
    for (const auto& attr : vertexAttributes) {
        VkVertexInputAttributeDescription desc{};
        desc.location = attr.location;
        desc.binding = attr.binding;
        desc.format = attr.format;
        desc.offset = attr.offset;
        attributeDescs.push_back(desc);
    }
    
    VkPipelineVertexInputStateCreateInfo vertexInputInfo{};
    vertexInputInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO;
    vertexInputInfo.vertexBindingDescriptionCount = static_cast<uint32_t>(bindingDescs.size());
    vertexInputInfo.pVertexBindingDescriptions = bindingDescs.data();
    vertexInputInfo.vertexAttributeDescriptionCount = static_cast<uint32_t>(attributeDescs.size());
    vertexInputInfo.pVertexAttributeDescriptions = attributeDescs.data();
    
    VkPipelineInputAssemblyStateCreateInfo inputAssembly{};
    inputAssembly.sType = VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO;
    inputAssembly.topology = static_cast<VkPrimitiveTopology>(primTopology);
    inputAssembly.primitiveRestartEnable = VK_FALSE;
    
    VkViewport viewport{};
    viewport.x = 0.0f;
    viewport.y = 0.0f;
    viewport.width = viewportWidth;
    viewport.height = viewportHeight;
    viewport.minDepth = 0.0f;
    viewport.maxDepth = 1.0f;
    
    VkRect2D scissor{};
    scissor.offset = {0, 0};
    scissor.extent = {static_cast<uint32_t>(viewportWidth), static_cast<uint32_t>(viewportHeight)};
    
    VkPipelineViewportStateCreateInfo viewportState{};
    viewportState.sType = VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO;
    viewportState.viewportCount = 1;
    viewportState.pViewports = nullptr;  // Usar estados dinámicos
    viewportState.scissorCount = 1;
    viewportState.pScissors = nullptr;   // Usar estados dinámicos
    
    // Habilitar estados dinámicos para viewport y scissor
    std::vector<VkDynamicState> dynamicStates = {
        VK_DYNAMIC_STATE_VIEWPORT,
        VK_DYNAMIC_STATE_SCISSOR
    };
    
    VkPipelineDynamicStateCreateInfo dynamicState{};
    dynamicState.sType = VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO;
    dynamicState.dynamicStateCount = static_cast<uint32_t>(dynamicStates.size());
    dynamicState.pDynamicStates = dynamicStates.data();
    
    VkPipelineRasterizationStateCreateInfo rasterizer{};
    rasterizer.sType = VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO;
    rasterizer.depthClampEnable = VK_FALSE;
    rasterizer.rasterizerDiscardEnable = VK_FALSE;
    rasterizer.polygonMode = static_cast<VkPolygonMode>(polyMode);
    rasterizer.lineWidth = 1.0f;
    rasterizer.cullMode = static_cast<VkCullModeFlags>(cullMd);
    rasterizer.frontFace = VK_FRONT_FACE_COUNTER_CLOCKWISE;
    rasterizer.depthBiasEnable = VK_FALSE;
    
    VkPipelineMultisampleStateCreateInfo multisampling{};
    multisampling.sType = VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO;
    multisampling.sampleShadingEnable = VK_FALSE;
    multisampling.rasterizationSamples = VK_SAMPLE_COUNT_1_BIT;
    
    VkPipelineDepthStencilStateCreateInfo depthStencil{};
    depthStencil.sType = VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO;
    depthStencil.depthTestEnable = enableDepthTest ? VK_TRUE : VK_FALSE;
    depthStencil.depthWriteEnable = enableDepthWrite ? VK_TRUE : VK_FALSE;
    depthStencil.depthCompareOp = VK_COMPARE_OP_LESS;
    depthStencil.depthBoundsTestEnable = VK_FALSE;
    depthStencil.stencilTestEnable = VK_FALSE;
    
    VkPipelineColorBlendAttachmentState colorBlendAttachment{};
    colorBlendAttachment.colorWriteMask = VK_COLOR_COMPONENT_R_BIT | VK_COLOR_COMPONENT_G_BIT |
                                         VK_COLOR_COMPONENT_B_BIT | VK_COLOR_COMPONENT_A_BIT;
    
    switch (blendMode) {
        case BlendMode::None:
            colorBlendAttachment.blendEnable = VK_FALSE;
            break;
        case BlendMode::Alpha:
            colorBlendAttachment.blendEnable = VK_TRUE;
            colorBlendAttachment.srcColorBlendFactor = VK_BLEND_FACTOR_SRC_ALPHA;
            colorBlendAttachment.dstColorBlendFactor = VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA;
            colorBlendAttachment.colorBlendOp = VK_BLEND_OP_ADD;
            colorBlendAttachment.srcAlphaBlendFactor = VK_BLEND_FACTOR_ONE;
            colorBlendAttachment.dstAlphaBlendFactor = VK_BLEND_FACTOR_ZERO;
            colorBlendAttachment.alphaBlendOp = VK_BLEND_OP_ADD;
            break;
        case BlendMode::Additive:
            colorBlendAttachment.blendEnable = VK_TRUE;
            colorBlendAttachment.srcColorBlendFactor = VK_BLEND_FACTOR_ONE;
            colorBlendAttachment.dstColorBlendFactor = VK_BLEND_FACTOR_ONE;
            colorBlendAttachment.colorBlendOp = VK_BLEND_OP_ADD;
            break;
        case BlendMode::Multiply:
            colorBlendAttachment.blendEnable = VK_TRUE;
            colorBlendAttachment.srcColorBlendFactor = VK_BLEND_FACTOR_DST_COLOR;
            colorBlendAttachment.dstColorBlendFactor = VK_BLEND_FACTOR_ZERO;
            colorBlendAttachment.colorBlendOp = VK_BLEND_OP_ADD;
            break;
    }
    
    VkPipelineColorBlendStateCreateInfo colorBlending{};
    colorBlending.sType = VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO;
    colorBlending.logicOpEnable = VK_FALSE;
    colorBlending.attachmentCount = 1;
    colorBlending.pAttachments = &colorBlendAttachment;
    
    VkPipelineLayoutCreateInfo pipelineLayoutInfo{};
    pipelineLayoutInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO;
    pipelineLayoutInfo.setLayoutCount = static_cast<uint32_t>(descLayouts.size());
    pipelineLayoutInfo.pSetLayouts = descLayouts.empty() ? nullptr : descLayouts.data();
    pipelineLayoutInfo.pushConstantRangeCount = static_cast<uint32_t>(pushRanges.size());
    pipelineLayoutInfo.pPushConstantRanges = pushRanges.empty() ? nullptr : pushRanges.data();
    
    VkPipelineLayout layout;
    if (vkCreatePipelineLayout(dev, &pipelineLayoutInfo, nullptr, &layout) != VK_SUCCESS) {
        throw std::runtime_error("failed to create pipeline layout");
    }
    
    VkGraphicsPipelineCreateInfo pipelineInfo{};
    pipelineInfo.sType = VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO;
    pipelineInfo.stageCount = static_cast<uint32_t>(shaderStages.size());
    pipelineInfo.pStages = shaderStages.data();
    pipelineInfo.pVertexInputState = &vertexInputInfo;
    pipelineInfo.pInputAssemblyState = &inputAssembly;
    pipelineInfo.pViewportState = &viewportState;
    pipelineInfo.pRasterizationState = &rasterizer;
    pipelineInfo.pMultisampleState = &multisampling;
    pipelineInfo.pDepthStencilState = &depthStencil;
    pipelineInfo.pColorBlendState = &colorBlending;
    pipelineInfo.pDynamicState = &dynamicState;
    pipelineInfo.layout = layout;
    pipelineInfo.renderPass = renderPass;
    pipelineInfo.subpass = 0;
    
    VkPipeline pipeline;
    if (vkCreateGraphicsPipelines(dev, VK_NULL_HANDLE, 1, &pipelineInfo, nullptr, &pipeline) != VK_SUCCESS) {
        vkDestroyPipelineLayout(dev, layout, nullptr);
        throw std::runtime_error("failed to create graphics pipeline");
    }
    
    return GraphicsPipeline(dev, pipeline, layout);
}

GraphicsPipeline::Builder GraphicsPipeline::create(VkDevice device, VkRenderPass renderPass) {
    return Builder(device, renderPass);
}

ComputePipeline::ComputePipeline(VkDevice device, VkPipeline pipeline, VkPipelineLayout layout)
    : device(device), pipeline(pipeline), pipelineLayout(layout) {}

ComputePipeline::~ComputePipeline() {
    if (pipeline != VK_NULL_HANDLE) {
        vkDestroyPipeline(device, pipeline, nullptr);
    }
    if (pipelineLayout != VK_NULL_HANDLE) {
        vkDestroyPipelineLayout(device, pipelineLayout, nullptr);
    }
}

ComputePipeline::ComputePipeline(ComputePipeline&& other) noexcept
    : device(other.device), pipeline(other.pipeline), pipelineLayout(other.pipelineLayout) {
    other.pipeline = VK_NULL_HANDLE;
    other.pipelineLayout = VK_NULL_HANDLE;
}

ComputePipeline& ComputePipeline::operator=(ComputePipeline&& other) noexcept {
    if (this != &other) {
        if (pipeline != VK_NULL_HANDLE) {
            vkDestroyPipeline(device, pipeline, nullptr);
        }
        if (pipelineLayout != VK_NULL_HANDLE) {
            vkDestroyPipelineLayout(device, pipelineLayout, nullptr);
        }
        device = other.device;
        pipeline = other.pipeline;
        pipelineLayout = other.pipelineLayout;
        other.pipeline = VK_NULL_HANDLE;
        other.pipelineLayout = VK_NULL_HANDLE;
    }
    return *this;
}

ComputePipeline::Builder::Builder(VkDevice device) : dev(device) {}

ComputePipeline::Builder& ComputePipeline::Builder::shader(std::shared_ptr<Shader> shader) {
    computeShader = shader;
    return *this;
}

ComputePipeline::Builder& ComputePipeline::Builder::descriptorSetLayouts(
    const std::vector<VkDescriptorSetLayout>& layouts) {
    descLayouts = layouts;
    return *this;
}

ComputePipeline::Builder& ComputePipeline::Builder::pushConstantRanges(
    const std::vector<VkPushConstantRange>& ranges) {
    pushRanges = ranges;
    return *this;
}

ComputePipeline ComputePipeline::Builder::build() {
    if (!computeShader) {
        throw std::runtime_error("compute shader is required");
    }
    
    VkPipelineLayoutCreateInfo pipelineLayoutInfo{};
    pipelineLayoutInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO;
    pipelineLayoutInfo.setLayoutCount = static_cast<uint32_t>(descLayouts.size());
    pipelineLayoutInfo.pSetLayouts = descLayouts.empty() ? nullptr : descLayouts.data();
    pipelineLayoutInfo.pushConstantRangeCount = static_cast<uint32_t>(pushRanges.size());
    pipelineLayoutInfo.pPushConstantRanges = pushRanges.empty() ? nullptr : pushRanges.data();
    
    VkPipelineLayout layout;
    if (vkCreatePipelineLayout(dev, &pipelineLayoutInfo, nullptr, &layout) != VK_SUCCESS) {
        throw std::runtime_error("failed to create pipeline layout");
    }
    
    VkComputePipelineCreateInfo pipelineInfo{};
    pipelineInfo.sType = VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO;
    pipelineInfo.stage = computeShader->getStageInfo();
    pipelineInfo.layout = layout;
    
    VkPipeline pipeline;
    if (vkCreateComputePipelines(dev, VK_NULL_HANDLE, 1, &pipelineInfo, nullptr, &pipeline) != VK_SUCCESS) {
        vkDestroyPipelineLayout(dev, layout, nullptr);
        throw std::runtime_error("failed to create compute pipeline");
    }
    
    return ComputePipeline(dev, pipeline, layout);
}

ComputePipeline::Builder ComputePipeline::create(VkDevice device) {
    return Builder(device);
}

}
