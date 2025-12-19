cmake_minimum_required(VERSION 3.15)
project(TestGame)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Find Vulkan
find_package(Vulkan REQUIRED)

# Find or download GLFW
find_package(glfw3 QUIET)
if(NOT glfw3_FOUND)
    message(STATUS "GLFW3 not found, downloading with FetchContent...")
    include(FetchContent)
    FetchContent_Declare(
        glfw
        GIT_REPOSITORY https://github.com/glfw/glfw.git
        GIT_TAG        3.3.8
    )
    FetchContent_MakeAvailable(glfw)
endif()

add_executable(test-game main.cpp)

target_link_libraries(test-game PRIVATE 
    Vulkan::Vulkan
    glfw
)
