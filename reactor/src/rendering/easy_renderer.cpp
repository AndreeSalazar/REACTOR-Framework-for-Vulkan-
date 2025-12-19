#include "reactor/rendering/easy_renderer.hpp"
#include "reactor/vulkan_context.hpp"
#include "reactor/window.hpp"
#include <iostream>
#include <cstring>
#include <algorithm>
#include <fstream>
#include <array>

namespace reactor {

EasyRenderer::EasyRenderer(VulkanContext& ctx, Window& window)
    : ctx(ctx), window(window) {
    
    std::cout << "[EasyRenderer] FASE 8 - Rendering simplificado" << std::endl;
    std::cout << "  Inicializando rendering visual real..." << std::endl;
    
    try {
        createSwapchain();
        createRenderPass();
        createFramebuffers();
        createPipeline();
        createCommandPool();
        createCommandBuffers();
        createSyncObjects();
        
        ready = true;
        std::cout << "[EasyRenderer] ✓ Rendering visual listo" << std::endl;
    } catch (const std::exception& e) {
        std::cerr << "[EasyRenderer] Error: " << e.what() << std::endl;
        ready = false;
    }
}

EasyRenderer::~EasyRenderer() {
    cleanup();
}

void EasyRenderer::createSwapchain() {
    std::cout << "[EasyRenderer] Creando swapchain real..." << std::endl;
    
    // Crear surface
    surface = window.createSurface(ctx.instance());
    
    // Query surface capabilities
    VkSurfaceCapabilitiesKHR capabilities;
    vkGetPhysicalDeviceSurfaceCapabilitiesKHR(ctx.physical(), surface, &capabilities);
    
    // Choose surface format
    uint32_t formatCount;
    vkGetPhysicalDeviceSurfaceFormatsKHR(ctx.physical(), surface, &formatCount, nullptr);
    std::vector<VkSurfaceFormatKHR> formats(formatCount);
    vkGetPhysicalDeviceSurfaceFormatsKHR(ctx.physical(), surface, &formatCount, formats.data());
    
    VkSurfaceFormatKHR surfaceFormat = formats[0];
    for (const auto& format : formats) {
        if (format.format == VK_FORMAT_B8G8R8A8_SRGB && format.colorSpace == VK_COLOR_SPACE_SRGB_NONLINEAR_KHR) {
            surfaceFormat = format;
            break;
        }
    }
    swapchainFormat = surfaceFormat.format;
    
    // Choose extent
    if (capabilities.currentExtent.width != UINT32_MAX) {
        swapchainExtent = capabilities.currentExtent;
    } else {
        swapchainExtent = {1280, 720};
        swapchainExtent.width = std::max(capabilities.minImageExtent.width, 
                                         std::min(capabilities.maxImageExtent.width, swapchainExtent.width));
        swapchainExtent.height = std::max(capabilities.minImageExtent.height,
                                          std::min(capabilities.maxImageExtent.height, swapchainExtent.height));
    }
    
    // Choose image count
    uint32_t imageCount = capabilities.minImageCount + 1;
    if (capabilities.maxImageCount > 0 && imageCount > capabilities.maxImageCount) {
        imageCount = capabilities.maxImageCount;
    }
    
    // Create swapchain
    VkSwapchainCreateInfoKHR createInfo{};
    createInfo.sType = VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR;
    createInfo.surface = surface;
    createInfo.minImageCount = imageCount;
    createInfo.imageFormat = surfaceFormat.format;
    createInfo.imageColorSpace = surfaceFormat.colorSpace;
    createInfo.imageExtent = swapchainExtent;
    createInfo.imageArrayLayers = 1;
    createInfo.imageUsage = VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT;
    createInfo.imageSharingMode = VK_SHARING_MODE_EXCLUSIVE;
    createInfo.preTransform = capabilities.currentTransform;
    createInfo.compositeAlpha = VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR;
    createInfo.presentMode = VK_PRESENT_MODE_FIFO_KHR;
    createInfo.clipped = VK_TRUE;
    createInfo.oldSwapchain = VK_NULL_HANDLE;
    
    if (vkCreateSwapchainKHR(ctx.device(), &createInfo, nullptr, &swapchain) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create swapchain");
    }
    
    // Get swapchain images
    vkGetSwapchainImagesKHR(ctx.device(), swapchain, &imageCount, nullptr);
    swapchainImages.resize(imageCount);
    vkGetSwapchainImagesKHR(ctx.device(), swapchain, &imageCount, swapchainImages.data());
    
    // Create image views
    swapchainImageViews.resize(swapchainImages.size());
    for (size_t i = 0; i < swapchainImages.size(); i++) {
        VkImageViewCreateInfo viewInfo{};
        viewInfo.sType = VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO;
        viewInfo.image = swapchainImages[i];
        viewInfo.viewType = VK_IMAGE_VIEW_TYPE_2D;
        viewInfo.format = swapchainFormat;
        viewInfo.components.r = VK_COMPONENT_SWIZZLE_IDENTITY;
        viewInfo.components.g = VK_COMPONENT_SWIZZLE_IDENTITY;
        viewInfo.components.b = VK_COMPONENT_SWIZZLE_IDENTITY;
        viewInfo.components.a = VK_COMPONENT_SWIZZLE_IDENTITY;
        viewInfo.subresourceRange.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
        viewInfo.subresourceRange.baseMipLevel = 0;
        viewInfo.subresourceRange.levelCount = 1;
        viewInfo.subresourceRange.baseArrayLayer = 0;
        viewInfo.subresourceRange.layerCount = 1;
        
        if (vkCreateImageView(ctx.device(), &viewInfo, nullptr, &swapchainImageViews[i]) != VK_SUCCESS) {
            throw std::runtime_error("Failed to create image view");
        }
    }
    
    std::cout << "  ✓ Swapchain: " << swapchainExtent.width << "x" << swapchainExtent.height 
              << " (" << swapchainImages.size() << " images)" << std::endl;
}

void EasyRenderer::createRenderPass() {
    std::cout << "[EasyRenderer] Creando render pass con depth buffer..." << std::endl;
    
    // Color attachment
    VkAttachmentDescription colorAttachment{};
    colorAttachment.format = swapchainFormat;
    colorAttachment.samples = VK_SAMPLE_COUNT_1_BIT;
    colorAttachment.loadOp = VK_ATTACHMENT_LOAD_OP_CLEAR;
    colorAttachment.storeOp = VK_ATTACHMENT_STORE_OP_STORE;
    colorAttachment.stencilLoadOp = VK_ATTACHMENT_LOAD_OP_DONT_CARE;
    colorAttachment.stencilStoreOp = VK_ATTACHMENT_STORE_OP_DONT_CARE;
    colorAttachment.initialLayout = VK_IMAGE_LAYOUT_UNDEFINED;
    colorAttachment.finalLayout = VK_IMAGE_LAYOUT_PRESENT_SRC_KHR;
    
    // Depth attachment
    VkAttachmentDescription depthAttachment{};
    depthAttachment.format = depthFormat;
    depthAttachment.samples = VK_SAMPLE_COUNT_1_BIT;
    depthAttachment.loadOp = VK_ATTACHMENT_LOAD_OP_CLEAR;
    depthAttachment.storeOp = VK_ATTACHMENT_STORE_OP_DONT_CARE;
    depthAttachment.stencilLoadOp = VK_ATTACHMENT_LOAD_OP_DONT_CARE;
    depthAttachment.stencilStoreOp = VK_ATTACHMENT_STORE_OP_DONT_CARE;
    depthAttachment.initialLayout = VK_IMAGE_LAYOUT_UNDEFINED;
    depthAttachment.finalLayout = VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL;
    
    VkAttachmentReference colorAttachmentRef{};
    colorAttachmentRef.attachment = 0;
    colorAttachmentRef.layout = VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL;
    
    VkAttachmentReference depthAttachmentRef{};
    depthAttachmentRef.attachment = 1;
    depthAttachmentRef.layout = VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL;
    
    // Subpass con depth
    VkSubpassDescription subpass{};
    subpass.pipelineBindPoint = VK_PIPELINE_BIND_POINT_GRAPHICS;
    subpass.colorAttachmentCount = 1;
    subpass.pColorAttachments = &colorAttachmentRef;
    subpass.pDepthStencilAttachment = &depthAttachmentRef;
    
    // Dependency
    VkSubpassDependency dependency{};
    dependency.srcSubpass = VK_SUBPASS_EXTERNAL;
    dependency.dstSubpass = 0;
    dependency.srcStageMask = VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT | VK_PIPELINE_STAGE_EARLY_FRAGMENT_TESTS_BIT;
    dependency.srcAccessMask = 0;
    dependency.dstStageMask = VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT | VK_PIPELINE_STAGE_EARLY_FRAGMENT_TESTS_BIT;
    dependency.dstAccessMask = VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT | VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT;
    
    // Crear render pass con ambos attachments
    std::array<VkAttachmentDescription, 2> attachments = {colorAttachment, depthAttachment};
    VkRenderPassCreateInfo renderPassInfo{};
    renderPassInfo.sType = VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO;
    renderPassInfo.attachmentCount = static_cast<uint32_t>(attachments.size());
    renderPassInfo.pAttachments = attachments.data();
    renderPassInfo.subpassCount = 1;
    renderPassInfo.pSubpasses = &subpass;
    renderPassInfo.dependencyCount = 1;
    renderPassInfo.pDependencies = &dependency;
    
    if (vkCreateRenderPass(ctx.device(), &renderPassInfo, nullptr, &renderPass) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create render pass");
    }
    
    std::cout << "  ✓ Render pass creado con depth buffer" << std::endl;
}

void EasyRenderer::createFramebuffers() {
    std::cout << "[EasyRenderer] Creando depth buffer y framebuffers..." << std::endl;
    
    // Crear depth image
    VkImageCreateInfo imageInfo{};
    imageInfo.sType = VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO;
    imageInfo.imageType = VK_IMAGE_TYPE_2D;
    imageInfo.extent.width = swapchainExtent.width;
    imageInfo.extent.height = swapchainExtent.height;
    imageInfo.extent.depth = 1;
    imageInfo.mipLevels = 1;
    imageInfo.arrayLayers = 1;
    imageInfo.format = depthFormat;
    imageInfo.tiling = VK_IMAGE_TILING_OPTIMAL;
    imageInfo.initialLayout = VK_IMAGE_LAYOUT_UNDEFINED;
    imageInfo.usage = VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT;
    imageInfo.samples = VK_SAMPLE_COUNT_1_BIT;
    imageInfo.sharingMode = VK_SHARING_MODE_EXCLUSIVE;
    
    if (vkCreateImage(ctx.device(), &imageInfo, nullptr, &depthImage) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create depth image");
    }
    
    // Allocate memory for depth image
    VkMemoryRequirements memRequirements;
    vkGetImageMemoryRequirements(ctx.device(), depthImage, &memRequirements);
    
    VkMemoryAllocateInfo allocInfo{};
    allocInfo.sType = VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO;
    allocInfo.allocationSize = memRequirements.size;
    allocInfo.memoryTypeIndex = findMemoryType(memRequirements.memoryTypeBits, VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT);
    
    if (vkAllocateMemory(ctx.device(), &allocInfo, nullptr, &depthImageMemory) != VK_SUCCESS) {
        throw std::runtime_error("Failed to allocate depth image memory");
    }
    
    vkBindImageMemory(ctx.device(), depthImage, depthImageMemory, 0);
    
    // Create depth image view
    VkImageViewCreateInfo viewInfo{};
    viewInfo.sType = VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO;
    viewInfo.image = depthImage;
    viewInfo.viewType = VK_IMAGE_VIEW_TYPE_2D;
    viewInfo.format = depthFormat;
    viewInfo.subresourceRange.aspectMask = VK_IMAGE_ASPECT_DEPTH_BIT;
    viewInfo.subresourceRange.baseMipLevel = 0;
    viewInfo.subresourceRange.levelCount = 1;
    viewInfo.subresourceRange.baseArrayLayer = 0;
    viewInfo.subresourceRange.layerCount = 1;
    
    if (vkCreateImageView(ctx.device(), &viewInfo, nullptr, &depthImageView) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create depth image view");
    }
    
    std::cout << "  ✓ Depth buffer creado" << std::endl;
    
    // Crear framebuffers con color + depth
    framebuffers.resize(swapchainImageViews.size());
    
    for (size_t i = 0; i < swapchainImageViews.size(); i++) {
        std::array<VkImageView, 2> attachments = {swapchainImageViews[i], depthImageView};
        
        VkFramebufferCreateInfo framebufferInfo{};
        framebufferInfo.sType = VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO;
        framebufferInfo.renderPass = renderPass;
        framebufferInfo.attachmentCount = static_cast<uint32_t>(attachments.size());
        framebufferInfo.pAttachments = attachments.data();
        framebufferInfo.width = swapchainExtent.width;
        framebufferInfo.height = swapchainExtent.height;
        framebufferInfo.layers = 1;
        
        if (vkCreateFramebuffer(ctx.device(), &framebufferInfo, nullptr, &framebuffers[i]) != VK_SUCCESS) {
            throw std::runtime_error("Failed to create framebuffer");
        }
    }
    
    std::cout << "  ✓ " << framebuffers.size() << " framebuffers creados" << std::endl;
}

void EasyRenderer::createPipeline() {
    std::cout << "[EasyRenderer] Creando pipeline real con shaders..." << std::endl;
    
    // Cargar shaders compilados (desde directorio de ejecución)
    auto vertShaderCode = readFile("cube.vert.spv");
    auto fragShaderCode = readFile("cube.frag.spv");
    
    VkShaderModule vertShaderModule = createShaderModule(vertShaderCode);
    VkShaderModule fragShaderModule = createShaderModule(fragShaderCode);
    
    // Shader stages
    VkPipelineShaderStageCreateInfo vertShaderStageInfo{};
    vertShaderStageInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
    vertShaderStageInfo.stage = VK_SHADER_STAGE_VERTEX_BIT;
    vertShaderStageInfo.module = vertShaderModule;
    vertShaderStageInfo.pName = "main";
    
    VkPipelineShaderStageCreateInfo fragShaderStageInfo{};
    fragShaderStageInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
    fragShaderStageInfo.stage = VK_SHADER_STAGE_FRAGMENT_BIT;
    fragShaderStageInfo.module = fragShaderModule;
    fragShaderStageInfo.pName = "main";
    
    VkPipelineShaderStageCreateInfo shaderStages[] = {vertShaderStageInfo, fragShaderStageInfo};
    
    // Vertex input
    VkVertexInputBindingDescription bindingDescription{};
    bindingDescription.binding = 0;
    bindingDescription.stride = sizeof(float) * 6; // position(3) + color(3)
    bindingDescription.inputRate = VK_VERTEX_INPUT_RATE_VERTEX;
    
    VkVertexInputAttributeDescription attributeDescriptions[2]{};
    attributeDescriptions[0].binding = 0;
    attributeDescriptions[0].location = 0;
    attributeDescriptions[0].format = VK_FORMAT_R32G32B32_SFLOAT;
    attributeDescriptions[0].offset = 0;
    
    attributeDescriptions[1].binding = 0;
    attributeDescriptions[1].location = 1;
    attributeDescriptions[1].format = VK_FORMAT_R32G32B32_SFLOAT;
    attributeDescriptions[1].offset = sizeof(float) * 3;
    
    VkPipelineVertexInputStateCreateInfo vertexInputInfo{};
    vertexInputInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO;
    vertexInputInfo.vertexBindingDescriptionCount = 1;
    vertexInputInfo.pVertexBindingDescriptions = &bindingDescription;
    vertexInputInfo.vertexAttributeDescriptionCount = 2;
    vertexInputInfo.pVertexAttributeDescriptions = attributeDescriptions;
    
    // Input assembly
    VkPipelineInputAssemblyStateCreateInfo inputAssembly{};
    inputAssembly.sType = VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO;
    inputAssembly.topology = VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST;
    inputAssembly.primitiveRestartEnable = VK_FALSE;
    
    // Viewport and scissor
    VkViewport viewport{};
    viewport.x = 0.0f;
    viewport.y = 0.0f;
    viewport.width = static_cast<float>(swapchainExtent.width);
    viewport.height = static_cast<float>(swapchainExtent.height);
    viewport.minDepth = 0.0f;
    viewport.maxDepth = 1.0f;
    
    VkRect2D scissor{};
    scissor.offset = {0, 0};
    scissor.extent = swapchainExtent;
    
    VkPipelineViewportStateCreateInfo viewportState{};
    viewportState.sType = VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO;
    viewportState.viewportCount = 1;
    viewportState.pViewports = &viewport;
    viewportState.scissorCount = 1;
    viewportState.pScissors = &scissor;
    
    // Rasterizer - con back-face culling para cubo sólido
    VkPipelineRasterizationStateCreateInfo rasterizer{};
    rasterizer.sType = VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO;
    rasterizer.depthClampEnable = VK_FALSE;
    rasterizer.rasterizerDiscardEnable = VK_FALSE;
    rasterizer.polygonMode = VK_POLYGON_MODE_FILL;
    rasterizer.lineWidth = 1.0f;
    rasterizer.cullMode = VK_CULL_MODE_BACK_BIT;  // Culling para cubo sólido
    rasterizer.frontFace = VK_FRONT_FACE_COUNTER_CLOCKWISE;
    rasterizer.depthBiasEnable = VK_FALSE;
    
    // Multisampling
    VkPipelineMultisampleStateCreateInfo multisampling{};
    multisampling.sType = VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO;
    multisampling.sampleShadingEnable = VK_FALSE;
    multisampling.rasterizationSamples = VK_SAMPLE_COUNT_1_BIT;
    
    // Depth testing - CRÍTICO para 3D correcto
    VkPipelineDepthStencilStateCreateInfo depthStencil{};
    depthStencil.sType = VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO;
    depthStencil.depthTestEnable = VK_TRUE;
    depthStencil.depthWriteEnable = VK_TRUE;
    depthStencil.depthCompareOp = VK_COMPARE_OP_LESS;
    depthStencil.depthBoundsTestEnable = VK_FALSE;
    depthStencil.stencilTestEnable = VK_FALSE;
    
    // Color blending
    VkPipelineColorBlendAttachmentState colorBlendAttachment{};
    colorBlendAttachment.colorWriteMask = VK_COLOR_COMPONENT_R_BIT | VK_COLOR_COMPONENT_G_BIT | 
                                          VK_COLOR_COMPONENT_B_BIT | VK_COLOR_COMPONENT_A_BIT;
    colorBlendAttachment.blendEnable = VK_FALSE;
    
    VkPipelineColorBlendStateCreateInfo colorBlending{};
    colorBlending.sType = VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO;
    colorBlending.logicOpEnable = VK_FALSE;
    colorBlending.attachmentCount = 1;
    colorBlending.pAttachments = &colorBlendAttachment;
    
    // Pipeline layout con push constants para MVP
    VkPushConstantRange pushConstantRange{};
    pushConstantRange.stageFlags = VK_SHADER_STAGE_VERTEX_BIT;
    pushConstantRange.offset = 0;
    pushConstantRange.size = sizeof(Mat4); // 64 bytes para matriz 4x4
    
    VkPipelineLayoutCreateInfo pipelineLayoutInfo{};
    pipelineLayoutInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO;
    pipelineLayoutInfo.pushConstantRangeCount = 1;
    pipelineLayoutInfo.pPushConstantRanges = &pushConstantRange;
    
    if (vkCreatePipelineLayout(ctx.device(), &pipelineLayoutInfo, nullptr, &pipelineLayout) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create pipeline layout");
    }
    
    // Graphics pipeline
    VkGraphicsPipelineCreateInfo pipelineInfo{};
    pipelineInfo.sType = VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO;
    pipelineInfo.stageCount = 2;
    pipelineInfo.pStages = shaderStages;
    pipelineInfo.pVertexInputState = &vertexInputInfo;
    pipelineInfo.pInputAssemblyState = &inputAssembly;
    pipelineInfo.pViewportState = &viewportState;
    pipelineInfo.pRasterizationState = &rasterizer;
    pipelineInfo.pMultisampleState = &multisampling;
    pipelineInfo.pColorBlendState = &colorBlending;
    pipelineInfo.layout = pipelineLayout;
    pipelineInfo.renderPass = renderPass;
    pipelineInfo.subpass = 0;
    
    if (vkCreateGraphicsPipelines(ctx.device(), VK_NULL_HANDLE, 1, &pipelineInfo, nullptr, &pipeline) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create graphics pipeline");
    }
    
    // Cleanup shader modules
    vkDestroyShaderModule(ctx.device(), fragShaderModule, nullptr);
    vkDestroyShaderModule(ctx.device(), vertShaderModule, nullptr);
    
    std::cout << "  ✓ Pipeline creado con shaders" << std::endl;
}

void EasyRenderer::createCommandPool() {
    std::cout << "[EasyRenderer] Creando command pool real..." << std::endl;
    
    VkCommandPoolCreateInfo poolInfo{};
    poolInfo.sType = VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO;
    poolInfo.flags = VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT;
    poolInfo.queueFamilyIndex = ctx.graphicsQueueFamily();
    
    if (vkCreateCommandPool(ctx.device(), &poolInfo, nullptr, &commandPool) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create command pool");
    }
    
    std::cout << "  ✓ Command pool creado" << std::endl;
}

void EasyRenderer::createCommandBuffers() {
    std::cout << "[EasyRenderer] Creando command buffers reales..." << std::endl;
    
    commandBuffers.resize(framebuffers.size());
    
    VkCommandBufferAllocateInfo allocInfo{};
    allocInfo.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO;
    allocInfo.commandPool = commandPool;
    allocInfo.level = VK_COMMAND_BUFFER_LEVEL_PRIMARY;
    allocInfo.commandBufferCount = static_cast<uint32_t>(commandBuffers.size());
    
    if (vkAllocateCommandBuffers(ctx.device(), &allocInfo, commandBuffers.data()) != VK_SUCCESS) {
        throw std::runtime_error("Failed to allocate command buffers");
    }
    
    std::cout << "  ✓ " << commandBuffers.size() << " command buffers creados" << std::endl;
}

void EasyRenderer::createSyncObjects() {
    std::cout << "[EasyRenderer] Creando sync objects reales..." << std::endl;
    
    VkSemaphoreCreateInfo semaphoreInfo{};
    semaphoreInfo.sType = VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO;
    
    VkFenceCreateInfo fenceInfo{};
    fenceInfo.sType = VK_STRUCTURE_TYPE_FENCE_CREATE_INFO;
    fenceInfo.flags = VK_FENCE_CREATE_SIGNALED_BIT;
    
    if (vkCreateSemaphore(ctx.device(), &semaphoreInfo, nullptr, &imageAvailableSemaphore) != VK_SUCCESS ||
        vkCreateSemaphore(ctx.device(), &semaphoreInfo, nullptr, &renderFinishedSemaphore) != VK_SUCCESS ||
        vkCreateFence(ctx.device(), &fenceInfo, nullptr, &inFlightFence) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create sync objects");
    }
    
    std::cout << "  ✓ Sync objects creados (semaphores + fence)" << std::endl;
}

void EasyRenderer::createBuffers(const void* vertices, size_t vertexSize,
                                 const uint16_t* indices, size_t indexSize) {
    std::cout << "[EasyRenderer] Creando vertex/index buffers..." << std::endl;
    
    // Vertex buffer
    createBuffer(vertexSize, 
                VK_BUFFER_USAGE_VERTEX_BUFFER_BIT,
                VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT,
                vertexBuffer, vertexBufferMemory);
    
    void* data;
    vkMapMemory(ctx.device(), vertexBufferMemory, 0, vertexSize, 0, &data);
    memcpy(data, vertices, vertexSize);
    vkUnmapMemory(ctx.device(), vertexBufferMemory);
    
    // Index buffer
    createBuffer(indexSize,
                VK_BUFFER_USAGE_INDEX_BUFFER_BIT,
                VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT,
                indexBuffer, indexBufferMemory);
    
    vkMapMemory(ctx.device(), indexBufferMemory, 0, indexSize, 0, &data);
    memcpy(data, indices, indexSize);
    vkUnmapMemory(ctx.device(), indexBufferMemory);
    
    std::cout << "  ✓ Buffers creados: " << vertexSize << " bytes (vertex), " 
              << indexSize << " bytes (index)" << std::endl;
}

void EasyRenderer::beginFrame() {
    if (!ready) {
        std::cout << "[EasyRenderer] beginFrame: NOT READY!" << std::endl;
        return;
    }
    
    static int frameLog = 0;
    if (frameLog++ % 60 == 0) {
        std::cout << "[EasyRenderer] Frame " << currentFrame << " - beginFrame()" << std::endl;
    }
    
    // Wait for fence
    vkWaitForFences(ctx.device(), 1, &inFlightFence, VK_TRUE, UINT64_MAX);
    vkResetFences(ctx.device(), 1, &inFlightFence);
    
    // Acquire next image
    VkResult result = vkAcquireNextImageKHR(ctx.device(), swapchain, UINT64_MAX, imageAvailableSemaphore, VK_NULL_HANDLE, &currentImageIndex);
    if (result != VK_SUCCESS && result != VK_SUBOPTIMAL_KHR) {
        std::cout << "[EasyRenderer] ERROR: vkAcquireNextImageKHR failed with code " << result << std::endl;
        return;
    }
    
    // Reset command buffer
    vkResetCommandBuffer(commandBuffers[currentImageIndex], 0);
    
    // Begin command buffer
    VkCommandBufferBeginInfo beginInfo{};
    beginInfo.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;
    vkBeginCommandBuffer(commandBuffers[currentImageIndex], &beginInfo);
    
    // Begin render pass con clear values para color y depth
    VkRenderPassBeginInfo renderPassInfo{};
    renderPassInfo.sType = VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO;
    renderPassInfo.renderPass = renderPass;
    renderPassInfo.framebuffer = framebuffers[currentImageIndex];
    renderPassInfo.renderArea.offset = {0, 0};
    renderPassInfo.renderArea.extent = swapchainExtent;
    
    // Clear values: color + depth
    std::array<VkClearValue, 2> clearValues{};
    clearValues[0].color = {{this->clearColor.r, this->clearColor.g, this->clearColor.b, this->clearColor.a}};
    clearValues[1].depthStencil = {1.0f, 0};  // Depth = 1.0 (far), Stencil = 0
    renderPassInfo.clearValueCount = static_cast<uint32_t>(clearValues.size());
    renderPassInfo.pClearValues = clearValues.data();
    
    vkCmdBeginRenderPass(commandBuffers[currentImageIndex], &renderPassInfo, VK_SUBPASS_CONTENTS_INLINE);
}

void EasyRenderer::endFrame() {
    if (!ready) return;
    
    // End render pass
    vkCmdEndRenderPass(commandBuffers[currentImageIndex]);
    
    // End command buffer
    if (vkEndCommandBuffer(commandBuffers[currentImageIndex]) != VK_SUCCESS) {
        throw std::runtime_error("Failed to record command buffer");
    }
    
    // Submit
    VkSubmitInfo submitInfo{};
    submitInfo.sType = VK_STRUCTURE_TYPE_SUBMIT_INFO;
    
    VkSemaphore waitSemaphores[] = {imageAvailableSemaphore};
    VkPipelineStageFlags waitStages[] = {VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT};
    submitInfo.waitSemaphoreCount = 1;
    submitInfo.pWaitSemaphores = waitSemaphores;
    submitInfo.pWaitDstStageMask = waitStages;
    submitInfo.commandBufferCount = 1;
    submitInfo.pCommandBuffers = &commandBuffers[currentImageIndex];
    
    VkSemaphore signalSemaphores[] = {renderFinishedSemaphore};
    submitInfo.signalSemaphoreCount = 1;
    submitInfo.pSignalSemaphores = signalSemaphores;
    
    if (vkQueueSubmit(ctx.graphicsQueue(), 1, &submitInfo, inFlightFence) != VK_SUCCESS) {
        throw std::runtime_error("Failed to submit draw command buffer");
    }
    
    // Present
    VkPresentInfoKHR presentInfo{};
    presentInfo.sType = VK_STRUCTURE_TYPE_PRESENT_INFO_KHR;
    presentInfo.waitSemaphoreCount = 1;
    presentInfo.pWaitSemaphores = signalSemaphores;
    
    VkSwapchainKHR swapchains[] = {swapchain};
    presentInfo.swapchainCount = 1;
    presentInfo.pSwapchains = swapchains;
    presentInfo.pImageIndices = &currentImageIndex;
    
    vkQueuePresentKHR(ctx.graphicsQueue(), &presentInfo);
    
    currentFrame++;
}

void EasyRenderer::drawMesh(const void* vertices, size_t vertexCount,
                           const uint16_t* indices, size_t indexCount,
                           const Mat4& mvp, const Vec3& color) {
    if (!ready) return;
    
    static int drawLog = 0;
    if (drawLog++ % 60 == 0) {
        std::cout << "[EasyRenderer] drawMesh: " << vertexCount << " verts, " << indexCount << " indices" << std::endl;
    }
    
    // Crear buffers si no existen
    if (vertexBuffer == VK_NULL_HANDLE && vertices && indices) {
        std::cout << "[EasyRenderer] Creating buffers on first draw..." << std::endl;
        size_t vertexSize = vertexCount * sizeof(float);
        size_t indexSize = indexCount * sizeof(uint16_t);
        createBuffers(vertices, vertexSize, indices, indexSize);
    }
    
    // Bind pipeline
    vkCmdBindPipeline(commandBuffers[currentImageIndex], VK_PIPELINE_BIND_POINT_GRAPHICS, pipeline);
    
    // Push MVP matrix como push constant
    vkCmdPushConstants(commandBuffers[currentImageIndex], pipelineLayout, 
                      VK_SHADER_STAGE_VERTEX_BIT, 0, sizeof(Mat4), &mvp);
    
    // Bind vertex buffer
    VkBuffer vertexBuffers[] = {vertexBuffer};
    VkDeviceSize offsets[] = {0};
    vkCmdBindVertexBuffers(commandBuffers[currentImageIndex], 0, 1, vertexBuffers, offsets);
    
    // Bind index buffer
    vkCmdBindIndexBuffer(commandBuffers[currentImageIndex], indexBuffer, 0, VK_INDEX_TYPE_UINT16);
    
    // Draw
    vkCmdDrawIndexed(commandBuffers[currentImageIndex], indexCount, 1, 0, 0, 0);
}

void EasyRenderer::setClearColor(float r, float g, float b, float a) {
    clearColor = Vec4(r, g, b, a);
    std::cout << "[EasyRenderer] Clear color: (" << r << ", " << g << ", " << b << ", " << a << ")\n";
}

void EasyRenderer::setWireframe(bool enabled) {
    wireframeMode = enabled;
    std::cout << "[EasyRenderer] Wireframe: " << (enabled ? "ON" : "OFF") << std::endl;
}

void EasyRenderer::cleanup() {
    std::cout << "[EasyRenderer] Limpiando recursos..." << std::endl;
    
    vkDeviceWaitIdle(ctx.device());
    
    if (vertexBuffer != VK_NULL_HANDLE) {
        vkDestroyBuffer(ctx.device(), vertexBuffer, nullptr);
        vkFreeMemory(ctx.device(), vertexBufferMemory, nullptr);
    }
    
    if (indexBuffer != VK_NULL_HANDLE) {
        vkDestroyBuffer(ctx.device(), indexBuffer, nullptr);
        vkFreeMemory(ctx.device(), indexBufferMemory, nullptr);
    }
    
    if (pipeline != VK_NULL_HANDLE) vkDestroyPipeline(ctx.device(), pipeline, nullptr);
    if (pipelineLayout != VK_NULL_HANDLE) vkDestroyPipelineLayout(ctx.device(), pipelineLayout, nullptr);
    
    if (commandPool != VK_NULL_HANDLE) vkDestroyCommandPool(ctx.device(), commandPool, nullptr);
    
    for (auto framebuffer : framebuffers) {
        vkDestroyFramebuffer(ctx.device(), framebuffer, nullptr);
    }
    
    if (renderPass != VK_NULL_HANDLE) vkDestroyRenderPass(ctx.device(), renderPass, nullptr);
    
    for (auto imageView : swapchainImageViews) {
        vkDestroyImageView(ctx.device(), imageView, nullptr);
    }
    
    if (swapchain != VK_NULL_HANDLE) vkDestroySwapchainKHR(ctx.device(), swapchain, nullptr);
    
    if (surface != VK_NULL_HANDLE) vkDestroySurfaceKHR(ctx.instance(), surface, nullptr);
    
    if (imageAvailableSemaphore != VK_NULL_HANDLE) vkDestroySemaphore(ctx.device(), imageAvailableSemaphore, nullptr);
    if (renderFinishedSemaphore != VK_NULL_HANDLE) vkDestroySemaphore(ctx.device(), renderFinishedSemaphore, nullptr);
    if (inFlightFence != VK_NULL_HANDLE) vkDestroyFence(ctx.device(), inFlightFence, nullptr);
    
    std::cout << "[EasyRenderer] ✓ Limpieza completada" << std::endl;
}

VkShaderModule EasyRenderer::createShaderModule(const std::vector<char>& code) {
    VkShaderModuleCreateInfo createInfo{};
    createInfo.sType = VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO;
    createInfo.codeSize = code.size();
    createInfo.pCode = reinterpret_cast<const uint32_t*>(code.data());
    
    VkShaderModule shaderModule;
    if (vkCreateShaderModule(ctx.device(), &createInfo, nullptr, &shaderModule) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create shader module");
    }
    
    return shaderModule;
}

std::vector<char> EasyRenderer::readFile(const std::string& filename) {
    std::ifstream file(filename, std::ios::ate | std::ios::binary);
    
    if (!file.is_open()) {
        throw std::runtime_error("Failed to open file: " + filename);
    }
    
    size_t fileSize = static_cast<size_t>(file.tellg());
    std::vector<char> buffer(fileSize);
    
    file.seekg(0);
    file.read(buffer.data(), fileSize);
    file.close();
    
    return buffer;
}

uint32_t EasyRenderer::findMemoryType(uint32_t typeFilter, VkMemoryPropertyFlags properties) {
    VkPhysicalDeviceMemoryProperties memProperties;
    vkGetPhysicalDeviceMemoryProperties(ctx.physical(), &memProperties);
    
    for (uint32_t i = 0; i < memProperties.memoryTypeCount; i++) {
        if ((typeFilter & (1 << i)) && (memProperties.memoryTypes[i].propertyFlags & properties) == properties) {
            return i;
        }
    }
    
    throw std::runtime_error("Failed to find suitable memory type");
}

void EasyRenderer::createBuffer(VkDeviceSize size, VkBufferUsageFlags usage,
                               VkMemoryPropertyFlags properties, VkBuffer& buffer,
                               VkDeviceMemory& bufferMemory) {
    VkBufferCreateInfo bufferInfo{};
    bufferInfo.sType = VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO;
    bufferInfo.size = size;
    bufferInfo.usage = usage;
    bufferInfo.sharingMode = VK_SHARING_MODE_EXCLUSIVE;
    
    if (vkCreateBuffer(ctx.device(), &bufferInfo, nullptr, &buffer) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create buffer");
    }
    
    VkMemoryRequirements memRequirements;
    vkGetBufferMemoryRequirements(ctx.device(), buffer, &memRequirements);
    
    VkMemoryAllocateInfo allocInfo{};
    allocInfo.sType = VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO;
    allocInfo.allocationSize = memRequirements.size;
    allocInfo.memoryTypeIndex = findMemoryType(memRequirements.memoryTypeBits, properties);
    
    if (vkAllocateMemory(ctx.device(), &allocInfo, nullptr, &bufferMemory) != VK_SUCCESS) {
        throw std::runtime_error("Failed to allocate buffer memory");
    }
    
    vkBindBufferMemory(ctx.device(), buffer, bufferMemory, 0);
}

// QuickDraw implementation - Cubo completo estilo LunarG
void QuickDraw::cube(std::vector<float>& vertices, std::vector<uint16_t>& indices) {
    // 24 vértices (4 por cara) con colores grises distintos por cara
    // Formato: X, Y, Z, R, G, B
    float g1 = 0.6f, g2 = 0.5f, g3 = 0.4f, g4 = 0.45f, g5 = 0.55f, g6 = 0.35f;
    
    vertices = {
        // Front face (Z+) - gris claro
        -0.5f, -0.5f,  0.5f,  g1, g1, g1,  // 0
         0.5f, -0.5f,  0.5f,  g1, g1, g1,  // 1
         0.5f,  0.5f,  0.5f,  g1, g1, g1,  // 2
        -0.5f,  0.5f,  0.5f,  g1, g1, g1,  // 3
        
        // Back face (Z-) - gris medio
        -0.5f, -0.5f, -0.5f,  g2, g2, g2,  // 4
        -0.5f,  0.5f, -0.5f,  g2, g2, g2,  // 5
         0.5f,  0.5f, -0.5f,  g2, g2, g2,  // 6
         0.5f, -0.5f, -0.5f,  g2, g2, g2,  // 7
        
        // Top face (Y+) - gris más claro
        -0.5f,  0.5f, -0.5f,  g3, g3, g3,  // 8
        -0.5f,  0.5f,  0.5f,  g3, g3, g3,  // 9
         0.5f,  0.5f,  0.5f,  g3, g3, g3,  // 10
         0.5f,  0.5f, -0.5f,  g3, g3, g3,  // 11
        
        // Bottom face (Y-) - gris oscuro
        -0.5f, -0.5f, -0.5f,  g4, g4, g4,  // 12
         0.5f, -0.5f, -0.5f,  g4, g4, g4,  // 13
         0.5f, -0.5f,  0.5f,  g4, g4, g4,  // 14
        -0.5f, -0.5f,  0.5f,  g4, g4, g4,  // 15
        
        // Right face (X+) - gris medio-claro
         0.5f, -0.5f, -0.5f,  g5, g5, g5,  // 16
         0.5f,  0.5f, -0.5f,  g5, g5, g5,  // 17
         0.5f,  0.5f,  0.5f,  g5, g5, g5,  // 18
         0.5f, -0.5f,  0.5f,  g5, g5, g5,  // 19
        
        // Left face (X-) - gris oscuro
        -0.5f, -0.5f, -0.5f,  g6, g6, g6,  // 20
        -0.5f, -0.5f,  0.5f,  g6, g6, g6,  // 21
        -0.5f,  0.5f,  0.5f,  g6, g6, g6,  // 22
        -0.5f,  0.5f, -0.5f,  g6, g6, g6,  // 23
    };
    
    indices = {
        0,  1,  2,   2,  3,  0,   // Front
        4,  5,  6,   6,  7,  4,   // Back
        8,  9,  10,  10, 11, 8,   // Top
        12, 13, 14,  14, 15, 12,  // Bottom
        16, 17, 18,  18, 19, 16,  // Right
        20, 21, 22,  22, 23, 20,  // Left
    };
}

void QuickDraw::sphere(std::vector<float>& vertices, std::vector<uint16_t>& indices, int segments) {
    // TODO: Implementar esfera
    vertices.clear();
    indices.clear();
}

void QuickDraw::plane(std::vector<float>& vertices, std::vector<uint16_t>& indices) {
    // TODO: Implementar plano
    vertices.clear();
    indices.clear();
}

Vec3 QuickDraw::colorFromHSV(float h, float s, float v) {
    // TODO: Implementar conversión HSV a RGB
    return Vec3(1, 1, 1);
}

Vec3 QuickDraw::colorLerp(const Vec3& a, const Vec3& b, float t) {
    return Vec3(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t
    );
}

} // namespace reactor
