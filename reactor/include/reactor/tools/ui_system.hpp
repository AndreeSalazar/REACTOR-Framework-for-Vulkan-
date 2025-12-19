#pragma once
#include "../math.hpp"
#include <string>
#include <functional>

namespace reactor {

/**
 * @brief UISystem - Sistema de UI con ImGui
 * 
 * Uso ultra simple:
 * UISystem ui;
 * ui.begin();
 * ui.text("Hello World");
 * ui.button("Click Me", []() { std::cout << "Clicked!" << std::endl; });
 * ui.end();
 */
class UISystem {
public:
    UISystem();
    ~UISystem();
    
    /**
     * @brief Initialize (call once)
     */
    void init(void* window, void* instance, void* device, void* physicalDevice, uint32_t queueFamily);
    void shutdown();
    
    /**
     * @brief Frame lifecycle
     */
    void newFrame();
    void render();
    
    /**
     * @brief Windows
     */
    void beginWindow(const std::string& title, bool* open = nullptr);
    void endWindow();
    
    /**
     * @brief Widgets
     */
    void text(const std::string& text);
    bool button(const std::string& label);
    bool checkbox(const std::string& label, bool* value);
    bool slider(const std::string& label, float* value, float min, float max);
    bool colorPicker(const std::string& label, float* color);
    
    /**
     * @brief Input
     */
    bool inputText(const std::string& label, char* buffer, size_t bufferSize);
    bool inputFloat(const std::string& label, float* value);
    bool inputInt(const std::string& label, int* value);
    
    /**
     * @brief Layout
     */
    void separator();
    void sameLine();
    void spacing();
    
    /**
     * @brief Stats
     */
    void showDemoWindow();
    void showMetrics();

private:
    bool initialized{false};
};

} // namespace reactor
