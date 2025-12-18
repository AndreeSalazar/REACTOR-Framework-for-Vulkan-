add_executable(reactor-cube-render main.cpp)
target_link_libraries(reactor-cube-render PRIVATE reactor)

# Copiar shaders al directorio del ejecutable
add_custom_command(TARGET reactor-cube-render POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_directory
    ${CMAKE_CURRENT_SOURCE_DIR}/shaders
    $<TARGET_FILE_DIR:reactor-cube-render>/shaders
    COMMENT "Copiando shaders..."
)
