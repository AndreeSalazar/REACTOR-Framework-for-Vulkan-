#include "reactor/reactor.hpp"
#include "reactor/vulkan_context.hpp"
#include "reactor/buffer.hpp"
#include "reactor/shader.hpp"
#include "reactor/pipeline.hpp"
#include "reactor/command_buffer.hpp"
#include "reactor/sync.hpp"
#include "reactor/render_pass.hpp"
#include <iostream>
#include <array>

struct Vertex {
    float pos[2];
    float color[3];
};

int main() {
    try {
        reactor::VulkanContext ctx(true);
        ctx.init();
        
        std::cout << "REACTOR Triangle Example - Framework initialized successfully!" << std::endl;
        
        std::array<Vertex, 3> vertices = {{
            {{0.0f, -0.5f}, {1.0f, 0.0f, 0.0f}},
            {{0.5f, 0.5f}, {0.0f, 1.0f, 0.0f}},
            {{-0.5f, 0.5f}, {0.0f, 0.0f, 1.0f}}
        }};
        
        {
            auto vertexBuffer = reactor::Buffer::create(ctx.allocator())
                .size(sizeof(vertices))
                .usage(reactor::BufferUsage::Vertex)
                .memoryType(reactor::MemoryType::HostVisible)
                .build();
            
            vertexBuffer.upload(vertices.data(), sizeof(vertices));
            
            std::cout << "Created vertex buffer with " << vertices.size() << " vertices" << std::endl;
        }
        
        std::cout << "REACTOR Framework demonstration complete!" << std::endl;
        std::cout << "\nFramework Features Demonstrated:" << std::endl;
        std::cout << "  - Vulkan context initialization" << std::endl;
        std::cout << "  - Memory allocator integration" << std::endl;
        std::cout << "  - Buffer creation with builder pattern" << std::endl;
        std::cout << "  - Automatic resource management (RAII)" << std::endl;
        
        ctx.shutdown();
        return 0;
        
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
        return 1;
    }
}
