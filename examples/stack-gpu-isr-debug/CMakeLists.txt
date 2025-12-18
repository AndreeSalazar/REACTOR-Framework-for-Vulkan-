cmake_minimum_required(VERSION 3.24)

project(stack-gpu-isr-debug)

# Executable
add_executable(stack-gpu-isr-debug
    main.cpp
    ../stack-gpu-cube/cube_renderer.cpp
)

target_link_libraries(stack-gpu-isr-debug PRIVATE
    reactor
    glfw
    glm::glm
    Vulkan::Vulkan
)

target_include_directories(stack-gpu-isr-debug PRIVATE
    ${CMAKE_SOURCE_DIR}/reactor/include
    ${CMAKE_SOURCE_DIR}/examples/stack-gpu-cube
)

# Copy shaders to output directory
add_custom_command(TARGET stack-gpu-isr-debug POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E echo "Copying shaders to output directory..."
    COMMAND ${CMAKE_COMMAND} -E make_directory $<TARGET_FILE_DIR:stack-gpu-isr-debug>/shaders/cube
    COMMAND ${CMAKE_COMMAND} -E make_directory $<TARGET_FILE_DIR:stack-gpu-isr-debug>/shaders/isr
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${CMAKE_SOURCE_DIR}/shaders/cube/cube.vert.spv
        ${CMAKE_SOURCE_DIR}/shaders/cube/cube.frag.spv
        $<TARGET_FILE_DIR:stack-gpu-isr-debug>/shaders/cube/
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${CMAKE_SOURCE_DIR}/shaders/isr/importance.comp.spv
        ${CMAKE_SOURCE_DIR}/shaders/isr/adaptive.comp.spv
        ${CMAKE_SOURCE_DIR}/shaders/isr/temporal.comp.spv
        $<TARGET_FILE_DIR:stack-gpu-isr-debug>/shaders/isr/
)
