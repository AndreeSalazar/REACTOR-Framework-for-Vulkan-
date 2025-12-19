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

# Buscar GLM (requerido para matemáticas)
find_package(glm QUIET)
if(NOT glm_FOUND)
    # Intentar encontrar GLM en ubicaciones comunes
    find_path(GLM_INCLUDE_DIR "glm/glm.hpp"
        PATHS
        "C:/vcpkg/installed/x64-windows/include"
        "C:/vcpkg/installed/x64-windows-static/include"
        "C:/Program Files/glm"
        "${CMAKE_SOURCE_DIR}/third_party/glm"
        "${CMAKE_SOURCE_DIR}/external/glm"
    )
    if(GLM_INCLUDE_DIR)
        set(glm_FOUND TRUE)
        message(STATUS "GLM found manually at: ${GLM_INCLUDE_DIR}")
    else()
        # Usar FetchContent para descargar GLM automáticamente
        message(STATUS "GLM not found, downloading with FetchContent...")
        include(FetchContent)
        FetchContent_Declare(
            glm
            GIT_REPOSITORY https://github.com/g-truc/glm.git
            GIT_TAG        master
        )
        FetchContent_GetProperties(glm)
        if(NOT glm_POPULATED)
            FetchContent_Populate(glm)
            set(GLM_INCLUDE_DIR ${glm_SOURCE_DIR})
            set(glm_FOUND TRUE)
            message(STATUS "GLM downloaded to: ${GLM_INCLUDE_DIR}")
        endif()
    endif()
endif()

# Descargar ImGui para UI System (FASE 6) - Última versión estable
message(STATUS "Downloading ImGui latest stable version...")
include(FetchContent)
FetchContent_Declare(
    imgui
    GIT_REPOSITORY https://github.com/ocornut/imgui.git
    GIT_TAG v1.91.5
    GIT_SHALLOW TRUE
)
FetchContent_MakeAvailable(imgui)
set(IMGUI_DIR "${imgui_SOURCE_DIR}")
message(STATUS "ImGui v1.91.5 downloaded to: ${IMGUI_DIR}")

# Buscar GLFW (opcional)
find_package(glfw3 QUIET)
if(NOT glfw3_FOUND)
    # Intentar encontrar GLFW manualmente
    find_path(GLFW3_INCLUDE_DIR "GLFW/glfw3.h"
        PATHS
        "C:/vcpkg/installed/x64-windows/include"
        "C:/vcpkg/installed/x64-windows-static/include"
    )
    find_library(GLFW3_LIBRARY glfw3
        PATHS
        "C:/vcpkg/installed/x64-windows/lib"
        "C:/vcpkg/installed/x64-windows-static/lib"
    )
    if(GLFW3_INCLUDE_DIR AND GLFW3_LIBRARY)
        set(glfw3_FOUND TRUE)
        message(STATUS "GLFW3 found manually")
    else()
        # Usar FetchContent para descargar GLFW automáticamente
        message(STATUS "GLFW3 not found, downloading with FetchContent...")
        include(FetchContent)
        FetchContent_Declare(
            glfw
            GIT_REPOSITORY https://github.com/glfw/glfw.git
            GIT_TAG        3.3.8
        )
        FetchContent_GetProperties(glfw)
        if(NOT glfw_POPULATED)
            FetchContent_Populate(glfw)
            add_subdirectory(${glfw_SOURCE_DIR} ${glfw_BINARY_DIR})
            set(glfw3_FOUND TRUE)
            set(GLFW3_INCLUDE_DIR ${glfw_SOURCE_DIR}/include)
            set(GLFW3_LIBRARY glfw)
            message(STATUS "GLFW3 downloaded to: ${GLFW3_INCLUDE_DIR}")
        endif()
    endif()
endif()

if(glfw3_FOUND)
    set(REACTOR_HAS_WINDOW ON)
    message(STATUS "GLFW3 found - Window support enabled")
else()
    set(REACTOR_HAS_WINDOW OFF)
    message(STATUS "GLFW3 not found - Window support disabled (install with: install-dependencies.bat)")
endif()

add_library(reactor STATIC
  reactor/src/vulkan_context.cpp
  reactor/src/memory_allocator.cpp
  reactor/src/buffer.cpp
  reactor/src/image.cpp
  reactor/src/shader.cpp
  reactor/src/pipeline.cpp
  reactor/src/descriptor.cpp
  reactor/src/command_buffer.cpp
  reactor/src/sync.cpp
  reactor/src/render_pass.cpp
  reactor/src/swapchain.cpp
  # REACTOR Base Library - Complete Vulkan Features
  reactor/src/compute_pipeline.cpp
  reactor/src/descriptor_manager.cpp
  reactor/src/sampler.cpp
  reactor/src/framebuffer.cpp
  reactor/src/query_pool.cpp
  reactor/src/event.cpp
  reactor/src/pipeline_cache.cpp
  # FASE 2 - ASSETS & RESOURCES
  reactor/src/texture.cpp
  reactor/src/mesh.cpp
  reactor/src/material.cpp
  reactor/src/resource_manager.cpp
  # FASE 3 - SCENE & COMPONENTS
  reactor/src/scene/entity.cpp
  reactor/src/scene/transform.cpp
  reactor/src/scene/camera.cpp
  reactor/src/scene/scene.cpp
  # FASE 4 - ADVANCED RENDERING
  reactor/src/rendering/light.cpp
  reactor/src/rendering/shadow_map.cpp
  reactor/src/rendering/post_process.cpp
  reactor/src/rendering/particle_system.cpp
  # FASE 5 - GAMEPLAY
  reactor/src/gameplay/physics.cpp
  reactor/src/gameplay/animation.cpp
  reactor/src/gameplay/audio.cpp
  reactor/src/gameplay/input.cpp
  # FASE 6 - TOOLS & DEBUG
  reactor/src/tools/debug_renderer.cpp
  reactor/src/tools/profiler.cpp
  reactor/src/tools/serialization.cpp
  reactor/src/tools/ui_system.cpp
  # FASE 7 - EXTRAS
  reactor/src/extras/networking.cpp
  reactor/src/extras/scripting.cpp
  reactor/src/extras/compute.cpp
  reactor/src/extras/advanced_effects.cpp
  # FASE 8 - RENDERING HELPERS
  reactor/src/rendering/easy_renderer.cpp
  # GAME LAYER - Final abstraction (A->B->C)
  reactor/src/game/game.cpp
  # Stack-GPU-OP: SDF (Killer Triangle System)
  reactor/src/sdf/primitives.cpp
  reactor/src/sdf/raymarcher.cpp
  reactor/src/sdf/sdf_primitives.cpp
  # Stack-GPU-OP: ISR (Simplified Integration)
  reactor/src/isr/importance_simple.cpp
)

# Agregar window.cpp si GLFW está disponible
if(REACTOR_HAS_WINDOW)
    target_sources(reactor PRIVATE reactor/src/window.cpp)
    target_link_libraries(reactor PUBLIC glfw)
    target_compile_definitions(reactor PUBLIC REACTOR_HAS_WINDOW=1)
    
    # Agregar ImGui sources cuando GLFW está disponible
    target_sources(reactor PRIVATE
        ${IMGUI_DIR}/imgui.cpp
        ${IMGUI_DIR}/imgui_draw.cpp
        ${IMGUI_DIR}/imgui_tables.cpp
        ${IMGUI_DIR}/imgui_widgets.cpp
        ${IMGUI_DIR}/imgui_demo.cpp
        ${IMGUI_DIR}/backends/imgui_impl_glfw.cpp
        ${IMGUI_DIR}/backends/imgui_impl_vulkan.cpp
    )
endif()

target_include_directories(reactor PUBLIC
  reactor/include
  ${IMGUI_DIR}
  ${IMGUI_DIR}/backends
)

# Agregar GLM includes si está disponible
if(glm_FOUND)
    if(TARGET glm::glm)
        target_link_libraries(reactor PUBLIC glm::glm)
    elseif(TARGET glm)
        target_link_libraries(reactor PUBLIC glm)
    elseif(GLM_INCLUDE_DIR)
        target_include_directories(reactor PUBLIC ${GLM_INCLUDE_DIR})
    endif()
endif()

target_compile_definitions(reactor PUBLIC
  $<$<BOOL:${REACTOR_ENABLE_VALIDATION}>:REACTOR_ENABLE_VALIDATION=1>
)

target_link_libraries(reactor PUBLIC Vulkan::Vulkan)

# Examples (optional - only add if they exist)
if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/examples/triangle")
    add_subdirectory(examples/triangle)
endif()
if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/examples/cube-simple")
    add_subdirectory(examples/cube-simple)
endif()
if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/examples/sandbox")
    add_subdirectory(examples/sandbox)
endif()
if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/examples/stack-gpu-cube")
    add_subdirectory(examples/stack-gpu-cube)
endif()
if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/examples/stack-gpu-isr")
    add_subdirectory(examples/stack-gpu-isr)
endif()
if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/examples/killer-triangle")
    add_subdirectory(examples/killer-triangle)
endif()

# Test Game - Isolated testing environment
add_subdirectory(Test_Game)
