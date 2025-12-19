#pragma once
#include "../math.hpp"
#include <string>
#include <vector>
#include <map>

namespace reactor {

/**
 * @brief AnimationClip - Clip de animación
 */
class AnimationClip {
public:
    std::string name;
    float duration{1.0f};
    bool loop{true};
    
    /**
     * @brief Keyframes
     */
    struct Keyframe {
        float time;
        Vec3 position;
        Vec3 rotation;  // Euler angles
        Vec3 scale;
    };
    
    std::vector<Keyframe> keyframes;
    
    /**
     * @brief Sample animation at time
     */
    Keyframe sample(float time) const;
    
    /**
     * @brief Helpers
     */
    static AnimationClip idle();
    static AnimationClip walk();
    static AnimationClip run();
    static AnimationClip jump();
};

/**
 * @brief Animator - Componente de animación
 * 
 * Uso simple:
 * auto& animator = entity->addComponent<Animator>();
 * animator.play("walk");
 * animator.setSpeed(1.5f);
 */
class Animator {
public:
    Animator() = default;
    
    /**
     * @brief Clips
     */
    void addClip(const std::string& name, const AnimationClip& clip);
    void play(const std::string& name);
    void stop();
    void pause();
    void resume();
    
    /**
     * @brief Control
     */
    void setSpeed(float speed) { playbackSpeed = speed; }
    float getSpeed() const { return playbackSpeed; }
    
    void setTime(float time) { currentTime = time; }
    float getTime() const { return currentTime; }
    
    bool isPlaying() const { return playing && !paused; }
    
    /**
     * @brief Update
     */
    void update(float deltaTime);
    
    /**
     * @brief Current state
     */
    const AnimationClip* getCurrentClip() const;

private:
    std::map<std::string, AnimationClip> clips;
    std::string currentClipName;
    float currentTime{0.0f};
    float playbackSpeed{1.0f};
    bool playing{false};
    bool paused{false};
};

} // namespace reactor
