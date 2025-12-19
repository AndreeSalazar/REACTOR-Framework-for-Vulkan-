#include "reactor/tools/ui_system.hpp"
#include <iostream>

#ifdef REACTOR_HAS_WINDOW
#include <imgui.h>
#include <imgui_impl_glfw.h>
#include <imgui_impl_vulkan.h>
#endif

namespace reactor {

UISystem::UISystem() {
    std::cout << "[UISystem] Created" << std::endl;
}

UISystem::~UISystem() {
    if (initialized) {
        shutdown();
    }
}

void UISystem::init(void* window, void* instance, void* device, void* physicalDevice, uint32_t queueFamily) {
    IMGUI_CHECKVERSION();
    ImGui::CreateContext();
    ImGuiIO& io = ImGui::GetIO();
    io.ConfigFlags |= ImGuiConfigFlags_NavEnableKeyboard;
    
    ImGui::StyleColorsDark();
    
    // Note: Full Vulkan initialization would go here
    // For now, just mark as initialized
    initialized = true;
    
    std::cout << "[UISystem] Initialized with ImGui " << IMGUI_VERSION << std::endl;
}

void UISystem::shutdown() {
    if (!initialized) return;
    
#ifdef REACTOR_HAS_WINDOW
    ImGui_ImplVulkan_Shutdown();
    ImGui_ImplGlfw_Shutdown();
    ImGui::DestroyContext();
#endif
    
    initialized = false;
    std::cout << "[UISystem] Shutdown" << std::endl;
}

void UISystem::newFrame() {
    if (!initialized) return;
#ifdef REACTOR_HAS_WINDOW
    ImGui_ImplVulkan_NewFrame();
    ImGui_ImplGlfw_NewFrame();
    ImGui::NewFrame();
#endif
}

void UISystem::render() {
    if (!initialized) return;
#ifdef REACTOR_HAS_WINDOW
    ImGui::Render();
#endif
}

void UISystem::beginWindow(const std::string& title, bool* open) {
#ifdef REACTOR_HAS_WINDOW
    ImGui::Begin(title.c_str(), open);
#endif
}

void UISystem::endWindow() {
#ifdef REACTOR_HAS_WINDOW
    ImGui::End();
#endif
}

void UISystem::text(const std::string& text) {
#ifdef REACTOR_HAS_WINDOW
    ImGui::Text("%s", text.c_str());
#endif
}

bool UISystem::button(const std::string& label) {
#ifdef REACTOR_HAS_WINDOW
    return ImGui::Button(label.c_str());
#else
    return false;
#endif
}

bool UISystem::checkbox(const std::string& label, bool* value) {
#ifdef REACTOR_HAS_WINDOW
    return ImGui::Checkbox(label.c_str(), value);
#else
    return false;
#endif
}

bool UISystem::slider(const std::string& label, float* value, float min, float max) {
#ifdef REACTOR_HAS_WINDOW
    return ImGui::SliderFloat(label.c_str(), value, min, max);
#else
    return false;
#endif
}

bool UISystem::colorPicker(const std::string& label, float* color) {
#ifdef REACTOR_HAS_WINDOW
    return ImGui::ColorEdit3(label.c_str(), color);
#else
    return false;
#endif
}

bool UISystem::inputText(const std::string& label, char* buffer, size_t bufferSize) {
#ifdef REACTOR_HAS_WINDOW
    return ImGui::InputText(label.c_str(), buffer, bufferSize);
#else
    return false;
#endif
}

bool UISystem::inputFloat(const std::string& label, float* value) {
#ifdef REACTOR_HAS_WINDOW
    return ImGui::InputFloat(label.c_str(), value);
#else
    return false;
#endif
}

bool UISystem::inputInt(const std::string& label, int* value) {
#ifdef REACTOR_HAS_WINDOW
    return ImGui::InputInt(label.c_str(), value);
#else
    return false;
#endif
}

void UISystem::separator() {
#ifdef REACTOR_HAS_WINDOW
    ImGui::Separator();
#endif
}

void UISystem::sameLine() {
#ifdef REACTOR_HAS_WINDOW
    ImGui::SameLine();
#endif
}

void UISystem::spacing() {
#ifdef REACTOR_HAS_WINDOW
    ImGui::Spacing();
#endif
}

void UISystem::showDemoWindow() {
#ifdef REACTOR_HAS_WINDOW
    bool show = true;
    ImGui::ShowDemoWindow(&show);
#endif
}

void UISystem::showMetrics() {
#ifdef REACTOR_HAS_WINDOW
    ImGui::ShowMetricsWindow();
#endif
}

} // namespace reactor
