#pragma once
#include "../math.hpp"
#include <string>
#include <memory>
#include <vector>

namespace reactor {

/**
 * @brief AudioClip - Clip de audio
 */
class AudioClip {
public:
    std::string path;
    float duration{0.0f};
    
    /**
     * @brief Load from file
     */
    static AudioClip load(const std::string& path);
};

/**
 * @brief AudioSource - Fuente de audio 3D
 * 
 * Uso simple:
 * auto& audio = entity->addComponent<AudioSource>();
 * audio.clip = &myClip;
 * audio.play();
 */
class AudioSource {
public:
    AudioSource() = default;
    
    AudioClip* clip{nullptr};
    Vec3 position{0, 0, 0};
    
    float volume{1.0f};
    float pitch{1.0f};
    bool loop{false};
    bool spatialize{true};
    
    float minDistance{1.0f};
    float maxDistance{100.0f};
    
    /**
     * @brief Control
     */
    void play();
    void stop();
    void pause();
    void resume();
    
    bool isPlaying() const { return playing; }
    
    /**
     * @brief Presets
     */
    static AudioSource music();
    static AudioSource sfx();
    static AudioSource ambient();

private:
    bool playing{false};
    bool paused{false};
};

/**
 * @brief AudioListener - Oyente (cámara)
 */
class AudioListener {
public:
    Vec3 position{0, 0, 0};
    Vec3 forward{0, 0, -1};
    Vec3 up{0, 1, 0};
};

/**
 * @brief AudioSystem - Sistema de audio
 * 
 * Uso simple:
 * AudioSystem audio;
 * audio.setMasterVolume(0.8f);
 */
class AudioSystem {
public:
    AudioSystem();
    ~AudioSystem();
    
    /**
     * @brief Volume control
     */
    void setMasterVolume(float volume);
    void setMusicVolume(float volume);
    void setSFXVolume(float volume);
    
    float getMasterVolume() const { return masterVolume; }
    float getMusicVolume() const { return musicVolume; }
    float getSFXVolume() const { return sfxVolume; }
    
    /**
     * @brief Listener
     */
    void setListener(const AudioListener& listener);
    
    /**
     * @brief Update
     */
    void update(float deltaTime);

private:
    float masterVolume{1.0f};
    float musicVolume{1.0f};
    float sfxVolume{1.0f};
    AudioListener listener;
};

} // namespace reactor
