#pragma once

#include <vulkan/vulkan.h>

namespace reactor {

/**
 * @brief Event wrapper - Vulkan event synchronization
 * 
 * Abstracción completa de VkEvent para sincronización fine-grained
 */
class Event {
public:
    explicit Event(VkDevice device);
    ~Event();

    // No copyable
    Event(const Event&) = delete;
    Event& operator=(const Event&) = delete;

    // Movable
    Event(Event&& other) noexcept;
    Event& operator=(Event&& other) noexcept;

    VkEvent handle() const { return event_; }

    /**
     * @brief Set event from host
     */
    void set();

    /**
     * @brief Reset event from host
     */
    void reset();

    /**
     * @brief Get event status
     */
    bool isSet() const;

private:
    VkDevice device_;
    VkEvent event_;
};

} // namespace reactor
