#include "reactor/event.hpp"
#include <stdexcept>

namespace reactor {

Event::Event(VkDevice device)
    : device_(device), event_(VK_NULL_HANDLE) {
    
    VkEventCreateInfo eventInfo{};
    eventInfo.sType = VK_STRUCTURE_TYPE_EVENT_CREATE_INFO;

    if (vkCreateEvent(device_, &eventInfo, nullptr, &event_) != VK_SUCCESS) {
        throw std::runtime_error("Failed to create event");
    }
}

Event::~Event() {
    if (event_ != VK_NULL_HANDLE) {
        vkDestroyEvent(device_, event_, nullptr);
    }
}

Event::Event(Event&& other) noexcept
    : device_(other.device_), event_(other.event_) {
    other.event_ = VK_NULL_HANDLE;
}

Event& Event::operator=(Event&& other) noexcept {
    if (this != &other) {
        if (event_ != VK_NULL_HANDLE) {
            vkDestroyEvent(device_, event_, nullptr);
        }
        device_ = other.device_;
        event_ = other.event_;
        other.event_ = VK_NULL_HANDLE;
    }
    return *this;
}

void Event::set() {
    vkSetEvent(device_, event_);
}

void Event::reset() {
    vkResetEvent(device_, event_);
}

bool Event::isSet() const {
    VkResult result = vkGetEventStatus(device_, event_);
    return result == VK_EVENT_SET;
}

} // namespace reactor
