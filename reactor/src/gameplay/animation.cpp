#include "reactor/gameplay/animation.hpp"
#include <iostream>
#include <algorithm>

namespace reactor {

AnimationClip::Keyframe AnimationClip::sample(float time) const {
    if (keyframes.empty()) {
        return Keyframe{0, Vec3(0), Vec3(0), Vec3(1)};
    }
    
    // Wrap time if looping
    if (loop && duration > 0.0f) {
        time = fmod(time, duration);
    }
    
    // Find keyframes to interpolate between
    for (size_t i = 0; i < keyframes.size() - 1; i++) {
        if (time >= keyframes[i].time && time <= keyframes[i + 1].time) {
            float t = (time - keyframes[i].time) / (keyframes[i + 1].time - keyframes[i].time);
            
            Keyframe result;
            result.time = time;
            result.position = glm::mix(keyframes[i].position, keyframes[i + 1].position, t);
            result.rotation = glm::mix(keyframes[i].rotation, keyframes[i + 1].rotation, t);
            result.scale = glm::mix(keyframes[i].scale, keyframes[i + 1].scale, t);
            
            return result;
        }
    }
    
    return keyframes.back();
}

AnimationClip AnimationClip::idle() {
    AnimationClip clip;
    clip.name = "idle";
    clip.duration = 2.0f;
    clip.loop = true;
    
    clip.keyframes.push_back({0.0f, Vec3(0, 0, 0), Vec3(0, 0, 0), Vec3(1, 1, 1)});
    clip.keyframes.push_back({1.0f, Vec3(0, 0.1f, 0), Vec3(0, 0, 0), Vec3(1, 1, 1)});
    clip.keyframes.push_back({2.0f, Vec3(0, 0, 0), Vec3(0, 0, 0), Vec3(1, 1, 1)});
    
    std::cout << "[AnimationClip] Created idle preset" << std::endl;
    return clip;
}

AnimationClip AnimationClip::walk() {
    AnimationClip clip;
    clip.name = "walk";
    clip.duration = 1.0f;
    clip.loop = true;
    
    clip.keyframes.push_back({0.0f, Vec3(0, 0, 0), Vec3(0, 0, 0), Vec3(1, 1, 1)});
    clip.keyframes.push_back({0.5f, Vec3(0.5f, 0, 0), Vec3(0, 0.1f, 0), Vec3(1, 1, 1)});
    clip.keyframes.push_back({1.0f, Vec3(1.0f, 0, 0), Vec3(0, 0, 0), Vec3(1, 1, 1)});
    
    std::cout << "[AnimationClip] Created walk preset" << std::endl;
    return clip;
}

AnimationClip AnimationClip::run() {
    AnimationClip clip;
    clip.name = "run";
    clip.duration = 0.6f;
    clip.loop = true;
    
    clip.keyframes.push_back({0.0f, Vec3(0, 0, 0), Vec3(0, 0, 0), Vec3(1, 1, 1)});
    clip.keyframes.push_back({0.3f, Vec3(1.0f, 0, 0), Vec3(0, 0.2f, 0), Vec3(1, 1, 1)});
    clip.keyframes.push_back({0.6f, Vec3(2.0f, 0, 0), Vec3(0, 0, 0), Vec3(1, 1, 1)});
    
    std::cout << "[AnimationClip] Created run preset" << std::endl;
    return clip;
}

AnimationClip AnimationClip::jump() {
    AnimationClip clip;
    clip.name = "jump";
    clip.duration = 1.0f;
    clip.loop = false;
    
    clip.keyframes.push_back({0.0f, Vec3(0, 0, 0), Vec3(0, 0, 0), Vec3(1, 1, 1)});
    clip.keyframes.push_back({0.3f, Vec3(0, 2.0f, 0), Vec3(0, 0, 0), Vec3(1, 1, 1)});
    clip.keyframes.push_back({1.0f, Vec3(0, 0, 0), Vec3(0, 0, 0), Vec3(1, 1, 1)});
    
    std::cout << "[AnimationClip] Created jump preset" << std::endl;
    return clip;
}

void Animator::addClip(const std::string& name, const AnimationClip& clip) {
    clips[name] = clip;
}

void Animator::play(const std::string& name) {
    if (clips.find(name) != clips.end()) {
        currentClipName = name;
        currentTime = 0.0f;
        playing = true;
        paused = false;
        std::cout << "[Animator] Playing: " << name << std::endl;
    }
}

void Animator::stop() {
    playing = false;
    paused = false;
    currentTime = 0.0f;
}

void Animator::pause() {
    paused = true;
}

void Animator::resume() {
    paused = false;
}

void Animator::update(float deltaTime) {
    if (!playing || paused) return;
    
    auto* clip = getCurrentClip();
    if (!clip) return;
    
    currentTime += deltaTime * playbackSpeed;
    
    if (!clip->loop && currentTime >= clip->duration) {
        stop();
    }
}

const AnimationClip* Animator::getCurrentClip() const {
    auto it = clips.find(currentClipName);
    if (it != clips.end()) {
        return &it->second;
    }
    return nullptr;
}

} // namespace reactor
