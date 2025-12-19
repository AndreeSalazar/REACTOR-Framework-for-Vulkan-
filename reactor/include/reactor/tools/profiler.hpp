#pragma once
#include <string>
#include <chrono>
#include <map>
#include <vector>

namespace reactor {

/**
 * @brief Profiler - Sistema de profiling simple
 * 
 * Uso simple:
 * Profiler::begin("MyFunction");
 * // ... código ...
 * Profiler::end("MyFunction");
 * 
 * Profiler::printStats();
 */
class Profiler {
public:
    /**
     * @brief Timing
     */
    static void begin(const std::string& name);
    static void end(const std::string& name);
    
    /**
     * @brief Scoped timer (RAII)
     */
    class ScopedTimer {
    public:
        ScopedTimer(const std::string& name);
        ~ScopedTimer();
    private:
        std::string timerName;
    };
    
    /**
     * @brief Stats
     */
    struct Stats {
        double totalTime{0.0};
        double avgTime{0.0};
        double minTime{999999.0};
        double maxTime{0.0};
        size_t callCount{0};
    };
    
    static Stats getStats(const std::string& name);
    static void printStats();
    static void reset();
    
    /**
     * @brief Frame timing
     */
    static void beginFrame();
    static void endFrame();
    static double getFrameTime();
    static double getFPS();

private:
    struct TimerData {
        std::chrono::high_resolution_clock::time_point startTime;
        std::vector<double> samples;
    };
    
    static std::map<std::string, TimerData> timers;
    static std::chrono::high_resolution_clock::time_point frameStartTime;
    static double lastFrameTime;
};

// Macro for easy scoped profiling
#define PROFILE_SCOPE(name) reactor::Profiler::ScopedTimer _profiler_##__LINE__(name)
#define PROFILE_FUNCTION() PROFILE_SCOPE(__FUNCTION__)

} // namespace reactor
