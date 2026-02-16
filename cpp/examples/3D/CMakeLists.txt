# =============================================================================
# REACTOR 3D Example — CMake Build Configuration
# =============================================================================
# Build:
#   cmake -B build
#   cmake --build build
#   ./build/reactor_3d
# =============================================================================

cmake_minimum_required(VERSION 3.16)
project(reactor_3d VERSION 1.0.5 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Find REACTOR C++ SDK
set(REACTOR_CPP_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../reactor_cpp")
set(REACTOR_C_API_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../reactor_c_api")

# Include directories
include_directories(${REACTOR_CPP_DIR}/include)

# Find Vulkan (required by REACTOR)
find_package(Vulkan QUIET)

# Executable
add_executable(reactor_3d main_basic.cpp)

# Link against REACTOR C API library
set(REACTOR_C_API_RELEASE "${CMAKE_CURRENT_SOURCE_DIR}/../../reactor_c_api/target/release")
set(REACTOR_C_API_DEBUG "${CMAKE_CURRENT_SOURCE_DIR}/../../reactor_c_api/target/debug")

if(WIN32)
    # Windows: Link against .dll.lib
    target_link_directories(reactor_3d PRIVATE 
        "${REACTOR_C_API_RELEASE}"
        "${REACTOR_C_API_DEBUG}"
    )
    target_link_libraries(reactor_3d PRIVATE reactor_c_api.dll.lib)
else()
    # Linux/macOS: Link against .so or .dylib
    target_link_directories(reactor_3d PRIVATE 
        "${REACTOR_C_API_RELEASE}"
        "${REACTOR_C_API_DEBUG}"
    )
    target_link_libraries(reactor_3d PRIVATE reactor_c_api)
endif()

# Copy DLL to output directory on Windows
if(WIN32)
    add_custom_command(TARGET reactor_3d POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
            "${REACTOR_C_API_RELEASE}/reactor_c_api.dll"
            "$<TARGET_FILE_DIR:reactor_3d>"
        COMMENT "Copying REACTOR C API DLL..."
    )
endif()

# Compiler warnings
if(MSVC)
    target_compile_options(reactor_3d PRIVATE /W4)
else()
    target_compile_options(reactor_3d PRIVATE -Wall -Wextra -Wpedantic)
endif()

# Print configuration
message(STATUS "REACTOR 3D Example")
message(STATUS "  C++ Standard: ${CMAKE_CXX_STANDARD}")
message(STATUS "  REACTOR SDK: ${REACTOR_CPP_DIR}")
message(STATUS "  Build Type: ${CMAKE_BUILD_TYPE}")
