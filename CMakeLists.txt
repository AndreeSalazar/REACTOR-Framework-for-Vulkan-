cmake_minimum_required(VERSION 3.24)
project(reactor VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

option(REACTOR_ENABLE_VALIDATION "Enable Vulkan validation layers" ON)

find_package(Vulkan REQUIRED)

add_library(reactor STATIC
  reactor/src/reactor.cpp
  reactor/src/vulkan_context.cpp
  reactor/src/memory_allocator.cpp
  reactor/src/buffer.cpp
  reactor/src/image.cpp
  reactor/src/sync.cpp
  reactor/src/command_buffer.cpp
  reactor/src/shader.cpp
  reactor/src/pipeline.cpp
  reactor/src/descriptor.cpp
  reactor/src/render_pass.cpp
  reactor/src/swapchain.cpp
)

target_include_directories(reactor PUBLIC
  reactor/include
)

target_compile_definitions(reactor PUBLIC
  $<$<BOOL:${REACTOR_ENABLE_VALIDATION}>:REACTOR_ENABLE_VALIDATION=1>
)

target_link_libraries(reactor PUBLIC Vulkan::Vulkan)

add_subdirectory(examples/sandbox)
add_subdirectory(examples/triangle)

