add_executable(reactor-cube main.cpp)
target_link_libraries(reactor-cube PRIVATE reactor)

# Compilar shaders automáticamente
find_program(GLSLC glslc HINTS "$ENV{VULKAN_SDK}/Bin")

if(GLSLC)
    set(SHADER_DIR "${CMAKE_CURRENT_SOURCE_DIR}/shaders")
    set(SHADER_OUTPUT_DIR "${CMAKE_CURRENT_BINARY_DIR}/shaders")
    
    file(MAKE_DIRECTORY ${SHADER_OUTPUT_DIR})
    
    # Compilar vertex shader
    add_custom_command(
        OUTPUT ${SHADER_OUTPUT_DIR}/cube.vert.spv
        COMMAND ${GLSLC} ${SHADER_DIR}/cube.vert -o ${SHADER_OUTPUT_DIR}/cube.vert.spv
        DEPENDS ${SHADER_DIR}/cube.vert
        COMMENT "Compiling vertex shader..."
    )
    
    # Compilar fragment shader
    add_custom_command(
        OUTPUT ${SHADER_OUTPUT_DIR}/cube.frag.spv
        COMMAND ${GLSLC} ${SHADER_DIR}/cube.frag -o ${SHADER_OUTPUT_DIR}/cube.frag.spv
        DEPENDS ${SHADER_DIR}/cube.frag
        COMMENT "Compiling fragment shader..."
    )
    
    # Target personalizado para shaders
    add_custom_target(cube-shaders ALL
        DEPENDS 
            ${SHADER_OUTPUT_DIR}/cube.vert.spv
            ${SHADER_OUTPUT_DIR}/cube.frag.spv
    )
    
    add_dependencies(reactor-cube cube-shaders)
    
    # Copiar shaders al directorio del ejecutable
    add_custom_command(TARGET reactor-cube POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_directory
        ${SHADER_OUTPUT_DIR}
        $<TARGET_FILE_DIR:reactor-cube>/shaders
    )
    
    message(STATUS "Shader compilation enabled for cube example")
else()
    message(WARNING "glslc not found - shaders will not be compiled automatically")
endif()
