#include "reactor/tools/serialization.hpp"
#include "reactor/scene/scene.hpp"
#include <iostream>
#include <sstream>

namespace reactor {

void Serializer::write(const std::string& key, bool value) {
    data[key] = value ? "true" : "false";
}

void Serializer::write(const std::string& key, int value) {
    data[key] = std::to_string(value);
}

void Serializer::write(const std::string& key, float value) {
    data[key] = std::to_string(value);
}

void Serializer::write(const std::string& key, const std::string& value) {
    data[key] = value;
}

void Serializer::write(const std::string& key, const Vec2& value) {
    data[key] = std::to_string(value.x) + "," + std::to_string(value.y);
}

void Serializer::write(const std::string& key, const Vec3& value) {
    data[key] = std::to_string(value.x) + "," + std::to_string(value.y) + "," + std::to_string(value.z);
}

void Serializer::write(const std::string& key, const Vec4& value) {
    data[key] = std::to_string(value.x) + "," + std::to_string(value.y) + "," + std::to_string(value.z) + "," + std::to_string(value.w);
}

bool Serializer::readBool(const std::string& key, bool defaultValue) {
    auto it = data.find(key);
    if (it == data.end()) return defaultValue;
    return it->second == "true";
}

int Serializer::readInt(const std::string& key, int defaultValue) {
    auto it = data.find(key);
    if (it == data.end()) return defaultValue;
    return std::stoi(it->second);
}

float Serializer::readFloat(const std::string& key, float defaultValue) {
    auto it = data.find(key);
    if (it == data.end()) return defaultValue;
    return std::stof(it->second);
}

std::string Serializer::readString(const std::string& key, const std::string& defaultValue) {
    auto it = data.find(key);
    if (it == data.end()) return defaultValue;
    return it->second;
}

Vec2 Serializer::readVec2(const std::string& key, const Vec2& defaultValue) {
    auto it = data.find(key);
    if (it == data.end()) return defaultValue;
    
    std::stringstream ss(it->second);
    Vec2 result;
    char comma;
    ss >> result.x >> comma >> result.y;
    return result;
}

Vec3 Serializer::readVec3(const std::string& key, const Vec3& defaultValue) {
    auto it = data.find(key);
    if (it == data.end()) return defaultValue;
    
    std::stringstream ss(it->second);
    Vec3 result;
    char comma;
    ss >> result.x >> comma >> result.y >> comma >> result.z;
    return result;
}

Vec4 Serializer::readVec4(const std::string& key, const Vec4& defaultValue) {
    auto it = data.find(key);
    if (it == data.end()) return defaultValue;
    
    std::stringstream ss(it->second);
    Vec4 result;
    char comma;
    ss >> result.x >> comma >> result.y >> comma >> result.z >> comma >> result.w;
    return result;
}

bool Serializer::saveToFile(const std::string& path) {
    std::ofstream file(path);
    if (!file.is_open()) {
        std::cerr << "[Serializer] Failed to open file for writing: " << path << std::endl;
        return false;
    }
    
    for (const auto& [key, value] : data) {
        file << key << "=" << value << "\n";
    }
    
    file.close();
    std::cout << "[Serializer] Saved to: " << path << " (" << data.size() << " entries)" << std::endl;
    return true;
}

bool Serializer::loadFromFile(const std::string& path) {
    std::ifstream file(path);
    if (!file.is_open()) {
        std::cerr << "[Serializer] Failed to open file for reading: " << path << std::endl;
        return false;
    }
    
    data.clear();
    std::string line;
    while (std::getline(file, line)) {
        size_t pos = line.find('=');
        if (pos != std::string::npos) {
            std::string key = line.substr(0, pos);
            std::string value = line.substr(pos + 1);
            data[key] = value;
        }
    }
    
    file.close();
    std::cout << "[Serializer] Loaded from: " << path << " (" << data.size() << " entries)" << std::endl;
    return true;
}

void Serializer::clear() {
    data.clear();
}

bool SceneSerializer::saveScene(const std::string& path, Scene* scene) {
    if (!scene) return false;
    
    Serializer s;
    s.write("scene_name", scene->name());
    s.write("entity_count", static_cast<int>(scene->entityCount()));
    
    return s.saveToFile(path);
}

bool SceneSerializer::loadScene(const std::string& path, Scene* scene) {
    if (!scene) return false;
    
    Serializer s;
    if (!s.loadFromFile(path)) return false;
    
    std::string sceneName = s.readString("scene_name");
    scene->setName(sceneName);
    
    std::cout << "[SceneSerializer] Loaded scene: " << sceneName << std::endl;
    return true;
}

} // namespace reactor
