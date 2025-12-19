cmake_minimum_required(VERSION 3.24)

# Killer Triangle Example - Rendering sin triángulos tradicionales
add_executable(killer-triangle
    main.cpp
)

target_link_libraries(killer-triangle PRIVATE reactor)

# Compilar ray marching compute shader
find_program(GLSLC glslc HINTS ${VULKAN_SDK}/Bin)

if(GLSLC)
    set(SHADER_SOURCE ${CMAKE_SOURCE_DIR}/shaders/sdf/raymarch.comp)
    set(SHADER_OUTPUT ${CMAKE_BINARY_DIR}/shaders/sdf/raymarch.comp.spv)
    
    # Crear directorio de salida
    file(MAKE_DIRECTORY ${CMAKE_BINARY_DIR}/shaders/sdf)
    
    add_custom_command(
        OUTPUT ${SHADER_OUTPUT}
        COMMAND ${GLSLC} ${SHADER_SOURCE} -o ${SHADER_OUTPUT}
        DEPENDS ${SHADER_SOURCE}
        COMMENT "Compiling SDF ray marching compute shader: ${SHADER_SOURCE}"
    )
    
    add_custom_target(killer_triangle_shaders ALL DEPENDS ${SHADER_OUTPUT})
    add_dependencies(killer-triangle killer_triangle_shaders)
    
    # Copiar shaders al directorio del ejecutable
    add_custom_command(TARGET killer-triangle POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_directory
        ${CMAKE_BINARY_DIR}/shaders
        $<TARGET_FILE_DIR:killer-triangle>/shaders
        COMMENT "Copying SDF shaders to output directory..."
    )
else()
    message(WARNING "glslc not found - SDF shaders will not be compiled")
endif()
