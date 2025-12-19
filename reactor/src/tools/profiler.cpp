#include "reactor/tools/profiler.hpp"
#include <iostream>
#include <iomanip>
#include <algorithm>

namespace reactor {

std::map<std::string, Profiler::TimerData> Profiler::timers;
std::chrono::high_resolution_clock::time_point Profiler::frameStartTime;
double Profiler::lastFrameTime = 0.0;

void Profiler::begin(const std::string& name) {
    timers[name].startTime = std::chrono::high_resolution_clock::now();
}

void Profiler::end(const std::string& name) {
    auto endTime = std::chrono::high_resolution_clock::now();
    auto& timer = timers[name];
    
    double duration = std::chrono::duration<double, std::milli>(endTime - timer.startTime).count();
    timer.samples.push_back(duration);
}

Profiler::ScopedTimer::ScopedTimer(const std::string& name) : timerName(name) {
    Profiler::begin(timerName);
}

Profiler::ScopedTimer::~ScopedTimer() {
    Profiler::end(timerName);
}

Profiler::Stats Profiler::getStats(const std::string& name) {
    Stats stats;
    
    auto it = timers.find(name);
    if (it == timers.end() || it->second.samples.empty()) {
        return stats;
    }
    
    const auto& samples = it->second.samples;
    stats.callCount = samples.size();
    
    for (double sample : samples) {
        stats.totalTime += sample;
        stats.minTime = std::min(stats.minTime, sample);
        stats.maxTime = std::max(stats.maxTime, sample);
    }
    
    stats.avgTime = stats.totalTime / stats.callCount;
    
    return stats;
}

void Profiler::printStats() {
    std::cout << "\n========== PROFILER STATS ==========" << std::endl;
    std::cout << std::fixed << std::setprecision(3);
    
    for (const auto& [name, timer] : timers) {
        if (timer.samples.empty()) continue;
        
        Stats stats = getStats(name);
        
        std::cout << name << ":" << std::endl;
        std::cout << "  Calls: " << stats.callCount << std::endl;
        std::cout << "  Total: " << stats.totalTime << " ms" << std::endl;
        std::cout << "  Avg:   " << stats.avgTime << " ms" << std::endl;
        std::cout << "  Min:   " << stats.minTime << " ms" << std::endl;
        std::cout << "  Max:   " << stats.maxTime << " ms" << std::endl;
    }
    
    std::cout << "\nFrame Time: " << lastFrameTime << " ms (" << getFPS() << " FPS)" << std::endl;
    std::cout << "===================================\n" << std::endl;
}

void Profiler::reset() {
    timers.clear();
}

void Profiler::beginFrame() {
    frameStartTime = std::chrono::high_resolution_clock::now();
}

void Profiler::endFrame() {
    auto endTime = std::chrono::high_resolution_clock::now();
    lastFrameTime = std::chrono::duration<double, std::milli>(endTime - frameStartTime).count();
}

double Profiler::getFrameTime() {
    return lastFrameTime;
}

double Profiler::getFPS() {
    return lastFrameTime > 0.0 ? 1000.0 / lastFrameTime : 0.0;
}

} // namespace reactor
