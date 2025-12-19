#pragma once
#include "../math.hpp"
#include <string>
#include <map>
#include <functional>

namespace reactor {

/**
 * @brief Input - Sistema de input simplificado
 * 
 * Uso ultra simple:
 * if (Input::getKey(Key::W)) { moveForward(); }
 * if (Input::getKeyDown(Key::Space)) { jump(); }
 * Vec2 mouse = Input::getMousePosition();
 */
class Input {
public:
    enum class Key {
        // Letters
        A = 65, B, C, D, E, F, G, H, I, J, K, L, M,
        N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
        
        // Numbers
        Num0 = 48, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
        
        // Special
        Space = 32,
        Enter = 257,
        Escape = 256,
        Tab = 258,
        Backspace = 259,
        
        // Arrows
        Right = 262,
        Left = 263,
        Down = 264,
        Up = 265,
        
        // Modifiers
        LeftShift = 340,
        LeftControl = 341,
        LeftAlt = 342
    };
    
    enum class MouseButton {
        Left = 0,
        Right = 1,
        Middle = 2
    };
    
    /**
     * @brief Keyboard
     */
    static bool getKey(Key key);
    static bool getKeyDown(Key key);
    static bool getKeyUp(Key key);
    
    /**
     * @brief Mouse
     */
    static bool getMouseButton(MouseButton button);
    static bool getMouseButtonDown(MouseButton button);
    static bool getMouseButtonUp(MouseButton button);
    
    static Vec2 getMousePosition();
    static Vec2 getMouseDelta();
    static float getMouseScroll();
    
    /**
     * @brief Axes (virtual)
     */
    static float getAxis(const std::string& axisName);
    static Vec2 getAxis2D(const std::string& axisName);
    
    /**
     * @brief Update (called by engine)
     */
    static void update();
    static void setKeyState(int key, bool pressed);
    static void setMouseButtonState(int button, bool pressed);
    static void setMousePosition(float x, float y);
    static void setMouseScroll(float scroll);

private:
    static std::map<int, bool> keyStates;
    static std::map<int, bool> keyDownStates;
    static std::map<int, bool> keyUpStates;
    
    static std::map<int, bool> mouseButtonStates;
    static std::map<int, bool> mouseButtonDownStates;
    static std::map<int, bool> mouseButtonUpStates;
    
    static Vec2 mousePosition;
    static Vec2 lastMousePosition;
    static float mouseScroll;
};

} // namespace reactor
