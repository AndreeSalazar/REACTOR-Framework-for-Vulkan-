# =============================================================================
# REACTOR C++ Example — Hello REACTOR
# =============================================================================
# Simple example demonstrating the ONE CALL ReactorApp() pattern.
#
# Build:
#   1. cargo build --release -p reactor-c-api
#   2. cmake -B build
#   3. cmake --build build
# =============================================================================

cmake_minimum_required(VERSION 3.16)
project(hello_cpp VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# =============================================================================
# Add REACTOR C++ SDK
# =============================================================================

add_subdirectory(${CMAKE_CURRENT_SOURCE_DIR}/../../reactor_cpp reactor_cpp_build)

# =============================================================================
# Hello REACTOR executable
# =============================================================================

add_executable(hello_cpp main.cpp)

target_link_libraries(hello_cpp PRIVATE reactor_cpp)

# =============================================================================
# Copy DLL to output directory (Windows)
# =============================================================================

if(WIN32 AND REACTOR_C_API_LIBRARY)
    get_filename_component(REACTOR_DLL_DIR "${REACTOR_C_API_LIBRARY}" DIRECTORY)
    add_custom_command(TARGET hello_cpp POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
            "${REACTOR_C_API_LIBRARY}"
            "$<TARGET_FILE_DIR:hello_cpp>"
        COMMENT "Copying reactor_c_api.dll to output directory"
    )
endif()
