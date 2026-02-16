cmake_minimum_required(VERSION 3.16)
project(simple_cube)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_executable(simple_cube main.cpp)

target_link_libraries(simple_cube PRIVATE reactor_cpp)

# Copy DLL on Windows
if(WIN32)
    add_custom_command(TARGET simple_cube POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
        $<TARGET_FILE:reactor_c_api>
        $<TARGET_FILE_DIR:simple_cube>
    )
endif()
