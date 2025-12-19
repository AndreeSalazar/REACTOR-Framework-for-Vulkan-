#include "reactor/extras/scripting.hpp"
#include <iostream>
#include <fstream>
#include <sstream>

namespace reactor {

ScriptEngine::ScriptEngine() {
    std::cout << "[ScriptEngine] Initialized" << std::endl;
}

ScriptEngine::~ScriptEngine() = default;

bool ScriptEngine::execute(const std::string& code) {
    std::cout << "[ScriptEngine] Executing: " << code << std::endl;
    // TODO: Implement actual scripting (Lua, Python, etc.)
    return true;
}

bool ScriptEngine::executeFile(const std::string& filepath) {
    std::ifstream file(filepath);
    if (!file.is_open()) {
        std::cerr << "[ScriptEngine] Failed to open: " << filepath << std::endl;
        return false;
    }
    
    std::stringstream buffer;
    buffer << file.rdbuf();
    std::string code = buffer.str();
    
    std::cout << "[ScriptEngine] Executing file: " << filepath << std::endl;
    return execute(code);
}

void ScriptEngine::setGlobal(const std::string& name, const std::any& value) {
    globals[name] = value;
    std::cout << "[ScriptEngine] Set global: " << name << std::endl;
}

std::any ScriptEngine::getGlobal(const std::string& name) {
    auto it = globals.find(name);
    if (it != globals.end()) {
        return it->second;
    }
    return std::any();
}

} // namespace reactor
