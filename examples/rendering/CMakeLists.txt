add_executable(reactor-rendering main.cpp)
target_link_libraries(reactor-rendering PRIVATE reactor)

# Copiar assets si existen
if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/assets")
    add_custom_command(TARGET reactor-rendering POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_directory
        ${CMAKE_CURRENT_SOURCE_DIR}/assets
        $<TARGET_FILE_DIR:reactor-rendering>/assets
    )
endif()
