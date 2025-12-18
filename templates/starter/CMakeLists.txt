cmake_minimum_required(VERSION 3.24)
project(reactor-starter VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Buscar REACTOR Framework
# Opción 1: REACTOR como subdirectorio
if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/../../reactor")
    add_subdirectory(${CMAKE_CURRENT_SOURCE_DIR}/../.. reactor_framework)
    set(REACTOR_FOUND TRUE)
endif()

# Opción 2: REACTOR instalado en el sistema
if(NOT REACTOR_FOUND)
    find_package(reactor QUIET)
    if(reactor_FOUND)
        set(REACTOR_FOUND TRUE)
    endif()
endif()

if(NOT REACTOR_FOUND)
    message(FATAL_ERROR 
        "REACTOR Framework not found!\n"
        "Please ensure REACTOR is either:\n"
        "  1. In the parent directory (../../reactor)\n"
        "  2. Installed system-wide\n"
        "  3. Set CMAKE_PREFIX_PATH to REACTOR location"
    )
endif()

# Crear ejecutable
add_executable(reactor-starter
    src/main.cpp
)

# Enlazar con REACTOR
target_link_libraries(reactor-starter PRIVATE reactor)

# Copiar assets al directorio de build (si existen)
if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/assets")
    add_custom_command(TARGET reactor-starter POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_directory
        ${CMAKE_CURRENT_SOURCE_DIR}/assets
        $<TARGET_FILE_DIR:reactor-starter>/assets
        COMMENT "Copying assets to build directory"
    )
endif()

# Mensaje de éxito
message(STATUS "REACTOR Starter configured successfully!")
message(STATUS "  Build type: ${CMAKE_BUILD_TYPE}")
message(STATUS "  Output: ${CMAKE_BINARY_DIR}")
