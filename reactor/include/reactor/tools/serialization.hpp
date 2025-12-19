#pragma once
#include "../math.hpp"
#include <string>
#include <fstream>
#include <map>
#include <vector>

namespace reactor {

/**
 * @brief Serializer - Sistema de serialización JSON-like
 * 
 * Uso simple:
 * Serializer s;
 * s.write("name", "Player");
 * s.write("position", Vec3(1, 2, 3));
 * s.saveToFile("save.dat");
 */
class Serializer {
public:
    Serializer() = default;
    
    /**
     * @brief Write
     */
    void write(const std::string& key, bool value);
    void write(const std::string& key, int value);
    void write(const std::string& key, float value);
    void write(const std::string& key, const std::string& value);
    void write(const std::string& key, const Vec2& value);
    void write(const std::string& key, const Vec3& value);
    void write(const std::string& key, const Vec4& value);
    
    /**
     * @brief Read
     */
    bool readBool(const std::string& key, bool defaultValue = false);
    int readInt(const std::string& key, int defaultValue = 0);
    float readFloat(const std::string& key, float defaultValue = 0.0f);
    std::string readString(const std::string& key, const std::string& defaultValue = "");
    Vec2 readVec2(const std::string& key, const Vec2& defaultValue = Vec2(0));
    Vec3 readVec3(const std::string& key, const Vec3& defaultValue = Vec3(0));
    Vec4 readVec4(const std::string& key, const Vec4& defaultValue = Vec4(0));
    
    /**
     * @brief File I/O
     */
    bool saveToFile(const std::string& path);
    bool loadFromFile(const std::string& path);
    
    /**
     * @brief Clear
     */
    void clear();

private:
    std::map<std::string, std::string> data;
};

/**
 * @brief SceneSerializer - Serializar escenas completas
 */
class SceneSerializer {
public:
    /**
     * @brief Save/Load scene
     */
    static bool saveScene(const std::string& path, class Scene* scene);
    static bool loadScene(const std::string& path, class Scene* scene);
};

} // namespace reactor
