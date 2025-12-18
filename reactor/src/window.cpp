#include "reactor/window.hpp"
#define GLFW_INCLUDE_VULKAN
#include <GLFW/glfw3.h>
#include <stdexcept>

namespace reactor {

void Window::init() {
    if (!glfwInit()) {
        throw std::runtime_error("Failed to initialize GLFW");
    }
}

void Window::terminate() {
    glfwTerminate();
}

Window::Window(const WindowConfig& config) : config_(config) {
    glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);
    glfwWindowHint(GLFW_RESIZABLE, config.resizable ? GLFW_TRUE : GLFW_FALSE);
    
    GLFWmonitor* monitor = config.fullscreen ? glfwGetPrimaryMonitor() : nullptr;
    
    window_ = glfwCreateWindow(
        config.width,
        config.height,
        config.title.c_str(),
        monitor,
        nullptr
    );
    
    if (!window_) {
        throw std::runtime_error("Failed to create GLFW window");
    }
    
    glfwSetWindowUserPointer(window_, this);
    
    glfwSetKeyCallback(window_, keyCallbackStatic);
    glfwSetMouseButtonCallback(window_, mouseButtonCallbackStatic);
    glfwSetCursorPosCallback(window_, cursorPosCallbackStatic);
    glfwSetFramebufferSizeCallback(window_, framebufferSizeCallbackStatic);
}

Window::~Window() {
    if (window_) {
        glfwDestroyWindow(window_);
    }
}

Window::Window(Window&& other) noexcept
    : window_(other.window_)
    , config_(std::move(other.config_))
    , keyCallback_(std::move(other.keyCallback_))
    , mouseButtonCallback_(std::move(other.mouseButtonCallback_))
    , mouseMoveCallback_(std::move(other.mouseMoveCallback_))
    , resizeCallback_(std::move(other.resizeCallback_))
{
    other.window_ = nullptr;
    if (window_) {
        glfwSetWindowUserPointer(window_, this);
    }
}

Window& Window::operator=(Window&& other) noexcept {
    if (this != &other) {
        if (window_) {
            glfwDestroyWindow(window_);
        }
        
        window_ = other.window_;
        config_ = std::move(other.config_);
        keyCallback_ = std::move(other.keyCallback_);
        mouseButtonCallback_ = std::move(other.mouseButtonCallback_);
        mouseMoveCallback_ = std::move(other.mouseMoveCallback_);
        resizeCallback_ = std::move(other.resizeCallback_);
        
        other.window_ = nullptr;
        
        if (window_) {
            glfwSetWindowUserPointer(window_, this);
        }
    }
    return *this;
}

bool Window::shouldClose() const {
    return glfwWindowShouldClose(window_);
}

void Window::pollEvents() {
    glfwPollEvents();
}

VkSurfaceKHR Window::createSurface(VkInstance instance) {
    VkSurfaceKHR surface;
    if (glfwCreateWindowSurface(instance, window_, nullptr, &surface) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create window surface");
    }
    return surface;
}

void Window::getFramebufferSize(int* width, int* height) const {
    glfwGetFramebufferSize(window_, width, height);
}

void Window::setTitle(const std::string& title) {
    config_.title = title;
    glfwSetWindowTitle(window_, title.c_str());
}

void Window::setSize(int width, int height) {
    config_.width = width;
    config_.height = height;
    glfwSetWindowSize(window_, width, height);
}

void Window::setKeyCallback(KeyCallback callback) {
    keyCallback_ = std::move(callback);
}

void Window::setMouseButtonCallback(MouseButtonCallback callback) {
    mouseButtonCallback_ = std::move(callback);
}

void Window::setMouseMoveCallback(MouseMoveCallback callback) {
    mouseMoveCallback_ = std::move(callback);
}

void Window::setResizeCallback(ResizeCallback callback) {
    resizeCallback_ = std::move(callback);
}

void Window::keyCallbackStatic(GLFWwindow* window, int key, int scancode, int action, int mods) {
    auto* self = static_cast<Window*>(glfwGetWindowUserPointer(window));
    if (self && self->keyCallback_) {
        self->keyCallback_(key, action);
    }
}

void Window::mouseButtonCallbackStatic(GLFWwindow* window, int button, int action, int mods) {
    auto* self = static_cast<Window*>(glfwGetWindowUserPointer(window));
    if (self && self->mouseButtonCallback_) {
        self->mouseButtonCallback_(button, action);
    }
}

void Window::cursorPosCallbackStatic(GLFWwindow* window, double x, double y) {
    auto* self = static_cast<Window*>(glfwGetWindowUserPointer(window));
    if (self && self->mouseMoveCallback_) {
        self->mouseMoveCallback_(x, y);
    }
}

void Window::framebufferSizeCallbackStatic(GLFWwindow* window, int width, int height) {
    auto* self = static_cast<Window*>(glfwGetWindowUserPointer(window));
    if (self && self->resizeCallback_) {
        self->resizeCallback_(width, height);
    }
}

} // namespace reactor
