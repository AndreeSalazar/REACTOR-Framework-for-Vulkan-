#pragma once
#include <vulkan/vulkan.h>
#include <string>
#include <functional>
#include <memory>

struct GLFWwindow;

namespace reactor {

struct WindowConfig {
    std::string title = "REACTOR Application";
    int width = 1280;
    int height = 720;
    bool fullscreen = false;
    bool resizable = true;
    bool vsync = true;
};

class Window {
public:
    Window(const WindowConfig& config);
    ~Window();
    
    Window(const Window&) = delete;
    Window& operator=(const Window&) = delete;
    Window(Window&& other) noexcept;
    Window& operator=(Window&& other) noexcept;
    
    bool shouldClose() const;
    void pollEvents();
    
    VkSurfaceKHR createSurface(VkInstance instance);
    void getFramebufferSize(int* width, int* height) const;
    
    void setTitle(const std::string& title);
    void setSize(int width, int height);
    
    GLFWwindow* handle() const { return window_; }
    
    using KeyCallback = std::function<void(int key, int action)>;
    using MouseButtonCallback = std::function<void(int button, int action)>;
    using MouseMoveCallback = std::function<void(double x, double y)>;
    using ResizeCallback = std::function<void(int width, int height)>;
    
    void setKeyCallback(KeyCallback callback);
    void setMouseButtonCallback(MouseButtonCallback callback);
    void setMouseMoveCallback(MouseMoveCallback callback);
    void setResizeCallback(ResizeCallback callback);
    
    static void init();
    static void terminate();
    
private:
    GLFWwindow* window_ = nullptr;
    WindowConfig config_;
    
    KeyCallback keyCallback_;
    MouseButtonCallback mouseButtonCallback_;
    MouseMoveCallback mouseMoveCallback_;
    ResizeCallback resizeCallback_;
    
    static void keyCallbackStatic(GLFWwindow* window, int key, int scancode, int action, int mods);
    static void mouseButtonCallbackStatic(GLFWwindow* window, int button, int action, int mods);
    static void cursorPosCallbackStatic(GLFWwindow* window, double x, double y);
    static void framebufferSizeCallbackStatic(GLFWwindow* window, int width, int height);
};

} // namespace reactor
