add_executable(stack-gpu-isr 
    main.cpp
)
target_link_libraries(stack-gpu-isr PRIVATE reactor)

# Compilar shaders ISR
find_program(GLSLC glslc HINTS $ENV{VULKAN_SDK}/Bin)

if(GLSLC)
    message(STATUS "Found glslc: ${GLSLC}")
    
    # ISR Compute shaders
    add_custom_command(
        OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/importance.comp.spv
        COMMAND ${GLSLC} ${CMAKE_SOURCE_DIR}/shaders/isr/importance.comp 
                -o ${CMAKE_CURRENT_BINARY_DIR}/importance.comp.spv
        DEPENDS ${CMAKE_SOURCE_DIR}/shaders/isr/importance.comp
        COMMENT "Compiling ISR importance compute shader..."
    )
    
    add_custom_command(
        OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/adaptive.comp.spv
        COMMAND ${GLSLC} ${CMAKE_SOURCE_DIR}/shaders/isr/adaptive.comp 
                -o ${CMAKE_CURRENT_BINARY_DIR}/adaptive.comp.spv
        DEPENDS ${CMAKE_SOURCE_DIR}/shaders/isr/adaptive.comp
        COMMENT "Compiling ISR adaptive compute shader..."
    )
    
    add_custom_command(
        OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/temporal.comp.spv
        COMMAND ${GLSLC} ${CMAKE_SOURCE_DIR}/shaders/isr/temporal.comp 
                -o ${CMAKE_CURRENT_BINARY_DIR}/temporal.comp.spv
        DEPENDS ${CMAKE_SOURCE_DIR}/shaders/isr/temporal.comp
        COMMENT "Compiling ISR temporal compute shader..."
    )
    
    add_custom_target(stack-gpu-isr-shaders ALL
        DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/importance.comp.spv
                ${CMAKE_CURRENT_BINARY_DIR}/adaptive.comp.spv
                ${CMAKE_CURRENT_BINARY_DIR}/temporal.comp.spv
    )
    
    add_dependencies(stack-gpu-isr stack-gpu-isr-shaders)
    
    # Copiar shaders al directorio del ejecutable
    add_custom_command(TARGET stack-gpu-isr POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E make_directory
                $<TARGET_FILE_DIR:stack-gpu-isr>/shaders/isr
        COMMAND ${CMAKE_COMMAND} -E copy
                ${CMAKE_CURRENT_BINARY_DIR}/importance.comp.spv
                $<TARGET_FILE_DIR:stack-gpu-isr>/shaders/isr/importance.comp.spv
        COMMAND ${CMAKE_COMMAND} -E copy
                ${CMAKE_CURRENT_BINARY_DIR}/adaptive.comp.spv
                $<TARGET_FILE_DIR:stack-gpu-isr>/shaders/isr/adaptive.comp.spv
        COMMAND ${CMAKE_COMMAND} -E copy
                ${CMAKE_CURRENT_BINARY_DIR}/temporal.comp.spv
                $<TARGET_FILE_DIR:stack-gpu-isr>/shaders/isr/temporal.comp.spv
        COMMENT "Copying ISR shaders to output directory..."
    )
else()
    message(WARNING "glslc not found - ISR shaders will not be compiled automatically")
endif()
