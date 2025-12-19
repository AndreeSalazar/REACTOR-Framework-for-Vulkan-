#include "reactor/gameplay/audio.hpp"
#include <iostream>

namespace reactor {

AudioClip AudioClip::load(const std::string& path) {
    AudioClip clip;
    clip.path = path;
    clip.duration = 5.0f;  // Placeholder
    std::cout << "[AudioClip] Loaded: " << path << std::endl;
    return clip;
}

void AudioSource::play() {
    if (clip) {
        playing = true;
        paused = false;
        std::cout << "[AudioSource] Playing: " << clip->path << std::endl;
    }
}

void AudioSource::stop() {
    playing = false;
    paused = false;
}

void AudioSource::pause() {
    paused = true;
}

void AudioSource::resume() {
    paused = false;
}

AudioSource AudioSource::music() {
    AudioSource source;
    source.loop = true;
    source.spatialize = false;
    source.volume = 0.7f;
    std::cout << "[AudioSource] Created music preset" << std::endl;
    return source;
}

AudioSource AudioSource::sfx() {
    AudioSource source;
    source.loop = false;
    source.spatialize = true;
    source.volume = 1.0f;
    std::cout << "[AudioSource] Created SFX preset" << std::endl;
    return source;
}

AudioSource AudioSource::ambient() {
    AudioSource source;
    source.loop = true;
    source.spatialize = true;
    source.volume = 0.5f;
    std::cout << "[AudioSource] Created ambient preset" << std::endl;
    return source;
}

AudioSystem::AudioSystem() {
    std::cout << "[AudioSystem] Initialized" << std::endl;
}

AudioSystem::~AudioSystem() = default;

void AudioSystem::setMasterVolume(float volume) {
    masterVolume = glm::clamp(volume, 0.0f, 1.0f);
    std::cout << "[AudioSystem] Master volume: " << masterVolume << std::endl;
}

void AudioSystem::setMusicVolume(float volume) {
    musicVolume = glm::clamp(volume, 0.0f, 1.0f);
    std::cout << "[AudioSystem] Music volume: " << musicVolume << std::endl;
}

void AudioSystem::setSFXVolume(float volume) {
    sfxVolume = glm::clamp(volume, 0.0f, 1.0f);
    std::cout << "[AudioSystem] SFX volume: " << sfxVolume << std::endl;
}

void AudioSystem::setListener(const AudioListener& l) {
    listener = l;
}

void AudioSystem::update(float deltaTime) {
    // Update audio sources based on listener position
}

} // namespace reactor
