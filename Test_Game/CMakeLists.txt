cmake_minimum_required(VERSION 3.15)
project(TestGame)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Usar REACTOR library
add_executable(test-game
    main.cpp
    simple_renderer.cpp
)
target_link_libraries(test-game PRIVATE reactor)
