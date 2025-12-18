add_executable(stack-gpu-cube main.cpp)
target_link_libraries(stack-gpu-cube PRIVATE reactor)

# Compilar shaders SDF
find_program(GLSLC glslc HINTS $ENV{VULKAN_SDK}/Bin)

if(GLSLC)
    message(STATUS "Found glslc: ${GLSLC}")
    
    # Vertex shader
    add_custom_command(
        OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/raymarching.vert.spv
        COMMAND ${GLSLC} ${CMAKE_SOURCE_DIR}/shaders/sdf/raymarching.vert 
                -o ${CMAKE_CURRENT_BINARY_DIR}/raymarching.vert.spv
        DEPENDS ${CMAKE_SOURCE_DIR}/shaders/sdf/raymarching.vert
        COMMENT "Compiling raymarching vertex shader..."
    )
    
    # Fragment shader
    add_custom_command(
        OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/raymarching.frag.spv
        COMMAND ${GLSLC} ${CMAKE_SOURCE_DIR}/shaders/sdf/raymarching.frag 
                -o ${CMAKE_CURRENT_BINARY_DIR}/raymarching.frag.spv
                -I${CMAKE_SOURCE_DIR}/shaders/sdf
        DEPENDS ${CMAKE_SOURCE_DIR}/shaders/sdf/raymarching.frag
                ${CMAKE_SOURCE_DIR}/shaders/sdf/primitives.glsl
        COMMENT "Compiling raymarching fragment shader..."
    )
    
    add_custom_target(stack-gpu-cube-shaders ALL
        DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/raymarching.vert.spv
                ${CMAKE_CURRENT_BINARY_DIR}/raymarching.frag.spv
    )
    
    add_dependencies(stack-gpu-cube stack-gpu-cube-shaders)
    
    # Copiar shaders al directorio del ejecutable
    add_custom_command(TARGET stack-gpu-cube POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy
                ${CMAKE_CURRENT_BINARY_DIR}/raymarching.vert.spv
                $<TARGET_FILE_DIR:stack-gpu-cube>/shaders/raymarching.vert.spv
        COMMAND ${CMAKE_COMMAND} -E copy
                ${CMAKE_CURRENT_BINARY_DIR}/raymarching.frag.spv
                $<TARGET_FILE_DIR:stack-gpu-cube>/shaders/raymarching.frag.spv
        COMMENT "Copying shaders to output directory..."
    )
else()
    message(WARNING "glslc not found - shaders will not be compiled automatically")
endif()
