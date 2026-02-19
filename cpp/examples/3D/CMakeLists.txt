# =============================================================================
# REACTOR 3D Examples — CMake Build Configuration
# =============================================================================
# Build:
#   cmake -B build
#   cmake --build build
#
# Examples:
#   ./build/reactor_3d              — Basic cube (original)
#   ./build/reactor_ecs_scene       — ECS entity/component CRUD
#   ./build/reactor_pbr_materials   — PBR material system
#   ./build/reactor_frame_graph     — FrameGraph render passes
#   ./build/reactor_fps_controller  — FPS character controller
#   ./build/reactor_lighting        — Multi-light showcase
#   ./build/reactor_telemetry       — GPU stats & telemetry
#   ./build/reactor_play_mode       — Play-in-editor bridge
#   ./build/reactor_multi_object    — Large scene management
# =============================================================================

cmake_minimum_required(VERSION 3.16)
project(reactor_3d_examples VERSION 1.0.6 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Find REACTOR C++ SDK
set(REACTOR_CPP_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../reactor_cpp")
set(REACTOR_C_API_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../reactor_c_api")

# Include directories
include_directories(${REACTOR_CPP_DIR}/include)

# Find Vulkan (required by REACTOR)
find_package(Vulkan QUIET)

# =============================================================================
# Helper function to add a REACTOR example
# =============================================================================
function(add_reactor_example TARGET_NAME SOURCE_FILE)
    add_executable(${TARGET_NAME} ${SOURCE_FILE})

    # Link against REACTOR C API library
    set(REACTOR_C_API_RELEASE "${CMAKE_CURRENT_SOURCE_DIR}/../../reactor_c_api/target/release")
    set(REACTOR_C_API_DEBUG "${CMAKE_CURRENT_SOURCE_DIR}/../../reactor_c_api/target/debug")

    if(WIN32)
        target_link_directories(${TARGET_NAME} PRIVATE
            "${REACTOR_C_API_RELEASE}"
            "${REACTOR_C_API_DEBUG}"
        )
        target_link_libraries(${TARGET_NAME} PRIVATE reactor_c_api.dll.lib)
    else()
        target_link_directories(${TARGET_NAME} PRIVATE
            "${REACTOR_C_API_RELEASE}"
            "${REACTOR_C_API_DEBUG}"
        )
        target_link_libraries(${TARGET_NAME} PRIVATE reactor_c_api)
    endif()

    # Copy DLL to output directory on Windows
    if(WIN32)
        if(EXISTS "${REACTOR_C_API_RELEASE}/reactor_c_api.dll")
            add_custom_command(TARGET ${TARGET_NAME} POST_BUILD
                COMMAND ${CMAKE_COMMAND} -E copy_if_different
                    "${REACTOR_C_API_RELEASE}/reactor_c_api.dll"
                    "$<TARGET_FILE_DIR:${TARGET_NAME}>"
                COMMENT "Copying REACTOR C API DLL (release) for ${TARGET_NAME}..."
            )
        elseif(EXISTS "${REACTOR_C_API_DEBUG}/reactor_c_api.dll")
            add_custom_command(TARGET ${TARGET_NAME} POST_BUILD
                COMMAND ${CMAKE_COMMAND} -E copy_if_different
                    "${REACTOR_C_API_DEBUG}/reactor_c_api.dll"
                    "$<TARGET_FILE_DIR:${TARGET_NAME}>"
                COMMENT "Copying REACTOR C API DLL (debug) for ${TARGET_NAME}..."
            )
        else()
            message(WARNING "reactor_c_api.dll not found. Build it with: cargo build --release -p reactor-c-api")
        endif()
    endif()

    # Compiler warnings
    if(MSVC)
        target_compile_options(${TARGET_NAME} PRIVATE /W3)
    else()
        target_compile_options(${TARGET_NAME} PRIVATE -Wall -Wextra -Wpedantic)
    endif()

    message(STATUS "  + ${TARGET_NAME}")
endfunction()

# =============================================================================
# All Examples
# =============================================================================
message(STATUS "REACTOR 3D Examples:")

add_reactor_example(reactor_3d              main_basic.cpp)
add_reactor_example(reactor_ecs_scene       ecs_scene/main.cpp)
add_reactor_example(reactor_pbr_materials   pbr_materials/main.cpp)
add_reactor_example(reactor_frame_graph     frame_graph/main.cpp)
add_reactor_example(reactor_fps_controller  fps_controller/main.cpp)
add_reactor_example(reactor_lighting        lighting_showcase/main.cpp)
add_reactor_example(reactor_telemetry       telemetry_stats/main.cpp)
add_reactor_example(reactor_play_mode       play_mode/main.cpp)
add_reactor_example(reactor_multi_object    multi_object/main.cpp)

message(STATUS "")
message(STATUS "  C++ Standard: ${CMAKE_CXX_STANDARD}")
message(STATUS "  REACTOR SDK: ${REACTOR_CPP_DIR}")
message(STATUS "  Build Type: ${CMAKE_BUILD_TYPE}")
