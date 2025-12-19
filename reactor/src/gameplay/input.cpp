#include "reactor/gameplay/input.hpp"
#include <iostream>

namespace reactor {

// Static member initialization
std::map<int, bool> Input::keyStates;
std::map<int, bool> Input::keyDownStates;
std::map<int, bool> Input::keyUpStates;

std::map<int, bool> Input::mouseButtonStates;
std::map<int, bool> Input::mouseButtonDownStates;
std::map<int, bool> Input::mouseButtonUpStates;

Vec2 Input::mousePosition{0, 0};
Vec2 Input::lastMousePosition{0, 0};
float Input::mouseScroll{0};

bool Input::getKey(Key key) {
    return keyStates[static_cast<int>(key)];
}

bool Input::getKeyDown(Key key) {
    return keyDownStates[static_cast<int>(key)];
}

bool Input::getKeyUp(Key key) {
    return keyUpStates[static_cast<int>(key)];
}

bool Input::getMouseButton(MouseButton button) {
    return mouseButtonStates[static_cast<int>(button)];
}

bool Input::getMouseButtonDown(MouseButton button) {
    return mouseButtonDownStates[static_cast<int>(button)];
}

bool Input::getMouseButtonUp(MouseButton button) {
    return mouseButtonUpStates[static_cast<int>(button)];
}

Vec2 Input::getMousePosition() {
    return mousePosition;
}

Vec2 Input::getMouseDelta() {
    return mousePosition - lastMousePosition;
}

float Input::getMouseScroll() {
    return mouseScroll;
}

float Input::getAxis(const std::string& axisName) {
    if (axisName == "Horizontal") {
        float value = 0.0f;
        if (getKey(Key::D) || getKey(Key::Right)) value += 1.0f;
        if (getKey(Key::A) || getKey(Key::Left)) value -= 1.0f;
        return value;
    }
    else if (axisName == "Vertical") {
        float value = 0.0f;
        if (getKey(Key::W) || getKey(Key::Up)) value += 1.0f;
        if (getKey(Key::S) || getKey(Key::Down)) value -= 1.0f;
        return value;
    }
    return 0.0f;
}

Vec2 Input::getAxis2D(const std::string& axisName) {
    if (axisName == "Movement") {
        return Vec2(getAxis("Horizontal"), getAxis("Vertical"));
    }
    return Vec2(0, 0);
}

void Input::update() {
    // Clear down/up states
    keyDownStates.clear();
    keyUpStates.clear();
    mouseButtonDownStates.clear();
    mouseButtonUpStates.clear();
    
    // Update mouse delta
    lastMousePosition = mousePosition;
    
    // Reset scroll
    mouseScroll = 0.0f;
}

void Input::setKeyState(int key, bool pressed) {
    bool wasPressed = keyStates[key];
    keyStates[key] = pressed;
    
    if (pressed && !wasPressed) {
        keyDownStates[key] = true;
    }
    else if (!pressed && wasPressed) {
        keyUpStates[key] = true;
    }
}

void Input::setMouseButtonState(int button, bool pressed) {
    bool wasPressed = mouseButtonStates[button];
    mouseButtonStates[button] = pressed;
    
    if (pressed && !wasPressed) {
        mouseButtonDownStates[button] = true;
    }
    else if (!pressed && wasPressed) {
        mouseButtonUpStates[button] = true;
    }
}

void Input::setMousePosition(float x, float y) {
    mousePosition = Vec2(x, y);
}

void Input::setMouseScroll(float scroll) {
    mouseScroll = scroll;
}

} // namespace reactor
