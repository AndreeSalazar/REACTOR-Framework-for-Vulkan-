from conan import ConanFile
from conan.tools.cmake import CMakeToolchain, CMake, cmake_layout, CMakeDeps

class ReactorConan(ConanFile):
    name = "reactor"
    version = "0.1.0"
    license = "MIT"
    author = "REACTOR Team"
    url = "https://github.com/tu-usuario/reactor"
    description = "Modern React-style framework for Vulkan"
    topics = ("vulkan", "graphics", "game-engine", "framework")
    
    settings = "os", "compiler", "build_type", "arch"
    
    options = {
        "shared": [True, False],
        "fPIC": [True, False],
        "with_window": [True, False],
        "with_imgui": [True, False],
        "with_physics": [True, False]
    }
    
    default_options = {
        "shared": False,
        "fPIC": True,
        "with_window": True,
        "with_imgui": True,
        "with_physics": False
    }
    
    exports_sources = "CMakeLists.txt", "reactor/*", "examples/*"
    
    def requirements(self):
        # Core dependencies
        self.requires("vulkan-headers/1.3.268")
        self.requires("glm/0.9.9.8")
        
        # Optional dependencies
        if self.options.with_window:
            self.requires("glfw/3.3.8")
        
        if self.options.with_imgui:
            self.requires("imgui/1.89.9")
        
        if self.options.with_physics:
            self.requires("bullet3/3.25")
    
    def build_requirements(self):
        self.tool_requires("cmake/3.27.7")
    
    def config_options(self):
        if self.settings.os == "Windows":
            del self.options.fPIC
    
    def layout(self):
        cmake_layout(self)
    
    def generate(self):
        deps = CMakeDeps(self)
        deps.generate()
        tc = CMakeToolchain(self)
        tc.variables["REACTOR_ENABLE_VALIDATION"] = self.settings.build_type == "Debug"
        tc.generate()
    
    def build(self):
        cmake = CMake(self)
        cmake.configure()
        cmake.build()
    
    def package(self):
        cmake = CMake(self)
        cmake.install()
    
    def package_info(self):
        self.cpp_info.libs = ["reactor"]
        self.cpp_info.includedirs = ["include"]
        
        if self.settings.build_type == "Debug":
            self.cpp_info.defines.append("REACTOR_ENABLE_VALIDATION=1")
