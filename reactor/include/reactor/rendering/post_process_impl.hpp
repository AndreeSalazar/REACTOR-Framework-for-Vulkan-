#pragma once
#include "post_process.hpp"

namespace reactor {

template<typename T, typename... Args>
T* PostProcessStack::addEffect(Args&&... args) {
    static_assert(std::is_base_of<PostProcessEffect, T>::value, "T must inherit from PostProcessEffect");
    
    auto effect = std::make_unique<T>(std::forward<Args>(args)...);
    auto ptr = effect.get();
    effects.push_back(std::move(effect));
    
    return ptr;
}

} // namespace reactor
