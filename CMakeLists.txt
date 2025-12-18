cmake_minimum_required(VERSION 3.24)
project(reactor VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

option(REACTOR_ENABLE_VALIDATION "Enable Vulkan validation layers" ON)

# Buscar Vulkan SDK
if(NOT DEFINED ENV{VULKAN_SDK})
    # Auto-detectar Vulkan SDK en Windows
    if(WIN32)
        if(EXISTS "C:/VulkanSDK/1.4.328.1")
            set(ENV{VULKAN_SDK} "C:/VulkanSDK/1.4.328.1")
            message(STATUS "Auto-detected Vulkan SDK: C:/VulkanSDK/1.4.328.1")
        elseif(EXISTS "C:/VulkanSDK/1.3.290.0")
            set(ENV{VULKAN_SDK} "C:/VulkanSDK/1.3.290.0")
            message(STATUS "Auto-detected Vulkan SDK: C:/VulkanSDK/1.3.290.0")
        endif()
    endif()
endif()

# Configurar rutas de Vulkan manualmente si es necesario
if(WIN32 AND EXISTS "C:/VulkanSDK/1.4.328.1")
    set(Vulkan_INCLUDE_DIR "C:/VulkanSDK/1.4.328.1/Include")
    set(Vulkan_LIBRARY "C:/VulkanSDK/1.4.328.1/Lib/vulkan-1.lib")
    set(VULKAN_SDK "C:/VulkanSDK/1.4.328.1")
endif()

find_package(Vulkan REQUIRED)

# Buscar GLFW (opcional)
find_package(glfw3 QUIET)
if(glfw3_FOUND)
    set(REACTOR_HAS_WINDOW ON)
    message(STATUS "GLFW3 found - Window support enabled")
else()
    set(REACTOR_HAS_WINDOW OFF)
    message(STATUS "GLFW3 not found - Window support disabled (install with: install-dependencies.bat)")
endif()

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

# Agregar window.cpp si GLFW está disponible
if(REACTOR_HAS_WINDOW)
    target_sources(reactor PRIVATE reactor/src/window.cpp)
    target_link_libraries(reactor PUBLIC glfw)
    target_compile_definitions(reactor PUBLIC REACTOR_HAS_WINDOW=1)
endif()

target_include_directories(reactor PUBLIC
  reactor/include
)

target_compile_definitions(reactor PUBLIC
  $<$<BOOL:${REACTOR_ENABLE_VALIDATION}>:REACTOR_ENABLE_VALIDATION=1>
)

target_link_libraries(reactor PUBLIC Vulkan::Vulkan)

add_subdirectory(examples/sandbox)
add_subdirectory(examples/triangle)

# Agregar ejemplos con ventanas solo si GLFW está disponible
if(REACTOR_HAS_WINDOW)
    add_subdirectory(examples/cube-render)   # Cubo 3D COMPLETO con renderizado
    add_subdirectory(examples/cube-simple)   # Demo simple sin renderizado
    # add_subdirectory(examples/rendering)  # Requiere más APIs
    # add_subdirectory(examples/cube)       # Versión alternativa
endif()

