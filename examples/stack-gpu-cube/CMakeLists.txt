add_executable(stack-gpu-cube 
    main.cpp
    cube_renderer.cpp
)
target_link_libraries(stack-gpu-cube PRIVATE reactor)

# Compilar shaders SDF
find_program(GLSLC glslc HINTS $ENV{VULKAN_SDK}/Bin)

if(GLSLC)
    message(STATUS "Found glslc: ${GLSLC}")
    
    # Cube shaders
    add_custom_command(
        OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/cube.vert.spv
        COMMAND ${GLSLC} ${CMAKE_SOURCE_DIR}/shaders/cube/cube.vert 
                -o ${CMAKE_CURRENT_BINARY_DIR}/cube.vert.spv
        DEPENDS ${CMAKE_SOURCE_DIR}/shaders/cube/cube.vert
        COMMENT "Compiling cube vertex shader..."
    )
    
    add_custom_command(
        OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/cube.frag.spv
        COMMAND ${GLSLC} ${CMAKE_SOURCE_DIR}/shaders/cube/cube.frag 
                -o ${CMAKE_CURRENT_BINARY_DIR}/cube.frag.spv
        DEPENDS ${CMAKE_SOURCE_DIR}/shaders/cube/cube.frag
        COMMENT "Compiling cube fragment shader..."
    )
    
    add_custom_target(stack-gpu-cube-shaders ALL
        DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/cube.vert.spv
                ${CMAKE_CURRENT_BINARY_DIR}/cube.frag.spv
    )
    
    add_dependencies(stack-gpu-cube stack-gpu-cube-shaders)
    
    # Copiar shaders al directorio del ejecutable
    add_custom_command(TARGET stack-gpu-cube POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy
                ${CMAKE_CURRENT_BINARY_DIR}/cube.vert.spv
                $<TARGET_FILE_DIR:stack-gpu-cube>/shaders/cube.vert.spv
        COMMAND ${CMAKE_COMMAND} -E copy
                ${CMAKE_CURRENT_BINARY_DIR}/cube.frag.spv
                $<TARGET_FILE_DIR:stack-gpu-cube>/shaders/cube.frag.spv
        COMMENT "Copying shaders to output directory..."
    )
else()
    message(WARNING "glslc not found - shaders will not be compiled automatically")
endif()
