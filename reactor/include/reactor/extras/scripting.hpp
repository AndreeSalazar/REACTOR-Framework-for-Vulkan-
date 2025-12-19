#pragma once
#include <string>
#include <map>
#include <functional>
#include <any>

namespace reactor {

/**
 * @brief ScriptEngine - Motor de scripting simple
 * 
 * Uso simple:
 * ScriptEngine script;
 * script.registerFunction("print", [](const std::string& msg) {
 *     std::cout << msg << std::endl;
 * });
 * script.execute("print('Hello from script!')");
 */
class ScriptEngine {
public:
    ScriptEngine();
    ~ScriptEngine();
    
    /**
     * @brief Ejecutar script
     */
    bool execute(const std::string& code);
    bool executeFile(const std::string& filepath);
    
    /**
     * @brief Registrar funciones C++ para scripts
     */
    template<typename Func>
    void registerFunction(const std::string& name, Func func);
    
    /**
     * @brief Variables globales
     */
    void setGlobal(const std::string& name, const std::any& value);
    std::any getGlobal(const std::string& name);
    
    /**
     * @brief Llamar función del script desde C++
     */
    template<typename... Args>
    std::any callFunction(const std::string& name, Args&&... args);

private:
    std::map<std::string, std::any> globals;
    std::map<std::string, std::any> functions;
};

} // namespace reactor
