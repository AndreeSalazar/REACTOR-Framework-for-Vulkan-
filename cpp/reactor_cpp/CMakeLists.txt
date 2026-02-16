# =============================================================================
# REACTOR C++ SDK — CMake Configuration
# =============================================================================
# This builds the REACTOR C++ SDK as a header-only library that links
# against the Rust-built reactor_c_api dynamic library.
# =============================================================================

cmake_minimum_required(VERSION 3.16)
project(reactor_cpp VERSION 0.4.1 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# =============================================================================
# Header-only library
# =============================================================================

add_library(reactor_cpp INTERFACE)

target_include_directories(reactor_cpp INTERFACE
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
    $<INSTALL_INTERFACE:include>
)

# =============================================================================
# Find the Rust-built C API library
# =============================================================================

# Default path to the Rust target directory
set(REACTOR_RUST_TARGET_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../target" CACHE PATH
    "Path to Rust target directory containing reactor_c_api library")

# Determine library name based on platform
if(WIN32)
    set(REACTOR_C_API_LIB_NAME "reactor_c_api.dll")
    set(REACTOR_C_API_IMPORT_LIB "reactor_c_api.dll.lib")
elseif(APPLE)
    set(REACTOR_C_API_LIB_NAME "libreactor_c_api.dylib")
else()
    set(REACTOR_C_API_LIB_NAME "libreactor_c_api.so")
endif()

# Look for the library in release and debug directories
find_library(REACTOR_C_API_LIBRARY
    NAMES reactor_c_api reactor-c-api
    PATHS
        "${REACTOR_RUST_TARGET_DIR}/release"
        "${REACTOR_RUST_TARGET_DIR}/debug"
        "${CMAKE_CURRENT_SOURCE_DIR}/../reactor_c_api/target/release"
        "${CMAKE_CURRENT_SOURCE_DIR}/../reactor_c_api/target/debug"
    NO_DEFAULT_PATH
)

if(REACTOR_C_API_LIBRARY)
    message(STATUS "Found REACTOR C API: ${REACTOR_C_API_LIBRARY}")
    
    # Create imported library target
    add_library(reactor_c_api SHARED IMPORTED GLOBAL)
    set_target_properties(reactor_c_api PROPERTIES
        IMPORTED_LOCATION "${REACTOR_C_API_LIBRARY}"
    )
    
    # On Windows, we need the import library
    if(WIN32)
        get_filename_component(REACTOR_C_API_DIR "${REACTOR_C_API_LIBRARY}" DIRECTORY)
        set_target_properties(reactor_c_api PROPERTIES
            IMPORTED_IMPLIB "${REACTOR_C_API_DIR}/${REACTOR_C_API_IMPORT_LIB}"
        )
    endif()
    
    # Link the C API to our SDK
    target_link_libraries(reactor_cpp INTERFACE reactor_c_api)
else()
    message(WARNING "REACTOR C API library not found. Build it first with: cargo build --release -p reactor-c-api")
endif()

# =============================================================================
# Installation
# =============================================================================

include(GNUInstallDirs)

install(TARGETS reactor_cpp
    EXPORT reactor_cpp-targets
)

install(DIRECTORY include/
    DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
)

install(EXPORT reactor_cpp-targets
    FILE reactor_cpp-config.cmake
    NAMESPACE reactor::
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/reactor_cpp
)
