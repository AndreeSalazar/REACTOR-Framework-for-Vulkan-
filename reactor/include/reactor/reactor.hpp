#pragma once

/**
 * @file reactor.hpp
 * @brief REACTOR Framework - Header principal para incluir TODO
 * 
 * Este header incluye TODAS las funcionalidades de REACTOR para que
 * proyectos B, C, etc. puedan heredar fácilmente de la biblioteca base (A).
 * 
 * Uso:
 * #include <reactor/reactor.hpp>
 * 
 * Esto da acceso a:
 * - Vulkan context y device management
 * - Memory allocation
 * - Buffers e images
 * - Shaders y pipelines (graphics + compute)
 * - Command buffers
 * - Synchronization (fences, semaphores)
 * - Render passes
 * - Swapchain
 * - Descriptor management
 * - Window management (si GLFW disponible)
 * - SDF primitives (Killer Triangle)
 * - ISR system
 * - Math utilities
 */

// Core Vulkan
#include "reactor/vulkan_context.hpp"
#include "reactor/memory_allocator.hpp"
#include "reactor/buffer.hpp"
#include "reactor/image.hpp"
#include "reactor/shader.hpp"
#include "reactor/pipeline.hpp"
// Note: compute_pipeline.hpp has ComputePipelineBuilder for advanced users
// #include "reactor/compute_pipeline.hpp"
#include "reactor/descriptor.hpp"
#include "reactor/descriptor_manager.hpp"
#include "reactor/command_buffer.hpp"
#include "reactor/sync.hpp"
#include "reactor/render_pass.hpp"
#include "reactor/swapchain.hpp"
#include "reactor/framebuffer.hpp"
#include "reactor/sampler.hpp"

// FASE 2 - ASSETS & RESOURCES
#include "reactor/texture.hpp"
#include "reactor/mesh.hpp"
#include "reactor/material.hpp"
#include "reactor/resource_manager.hpp"

// FASE 3 - SCENE & COMPONENTS
#include "reactor/scene/component.hpp"
#include "reactor/scene/transform.hpp"
#include "reactor/scene/camera.hpp"
#include "reactor/scene/entity.hpp"
#include "reactor/scene/entity_impl.hpp"
#include "reactor/scene/scene.hpp"

// FASE 4 - ADVANCED RENDERING
#include "reactor/rendering/light.hpp"
#include "reactor/rendering/shadow_map.hpp"
#include "reactor/rendering/post_process.hpp"
#include "reactor/rendering/post_process_impl.hpp"
#include "reactor/rendering/particle_system.hpp"

// FASE 5 - GAMEPLAY
#include "reactor/gameplay/physics.hpp"
#include "reactor/gameplay/animation.hpp"
#include "reactor/gameplay/audio.hpp"
#include "reactor/gameplay/input.hpp"

// FASE 6 - TOOLS & DEBUG
#include "reactor/tools/ui_system.hpp"
#include "reactor/tools/debug_renderer.hpp"
#include "reactor/tools/profiler.hpp"
#include "reactor/tools/serialization.hpp"

// FASE 7 - EXTRAS
#include "reactor/extras/networking.hpp"
#include "reactor/extras/scripting.hpp"
#include "reactor/extras/compute.hpp"
#include "reactor/extras/advanced_effects.hpp"

// FASE 8 - RENDERING HELPERS
#include "reactor/rendering/easy_renderer.hpp"

// GAME LAYER - Final abstraction (A->B->C)
#include "reactor/game/game.hpp"

// EDITOR LAYER - Visual Editor (Blender + UE5 style)
#include "reactor/editor/editor.hpp"

// Window (si disponible)
#ifdef REACTOR_HAS_WINDOW
#include "reactor/window.hpp"
#endif

// Math utilities
#include "reactor/math.hpp"

// SDF System (Killer Triangle)
#include "reactor/sdf/sdf_primitives.hpp"
// Note: primitives.hpp and raymarcher.hpp for advanced SDF usage
// #include "reactor/sdf/primitives.hpp"
// #include "reactor/sdf/raymarcher.hpp"

// ISR System
// #include "reactor/isr/importance.hpp"

/**
 * @namespace reactor
 * @brief Namespace principal de REACTOR Framework
 * 
 * REACTOR es la biblioteca base (A) que proporciona TODAS las
 * funcionalidades fundamentales para proyectos Vulkan.
 * 
 * Proyectos B, C, etc. heredan de REACTOR y extienden según necesiten.
 */
namespace reactor {

/**
 * @brief Versión de REACTOR
 */
constexpr int VERSION_MAJOR = 0;
constexpr int VERSION_MINOR = 5;
constexpr int VERSION_PATCH = 0;

/**
 * @brief Obtiene la versión de REACTOR como string
 */
inline const char* getVersion() {
    return "REACTOR v0.5.0 - Vulkan Framework Base Library";
}

/**
 * @brief Características disponibles en REACTOR
 */
struct Features {
    static constexpr bool HAS_WINDOW = 
#ifdef REACTOR_HAS_WINDOW
        true;
#else
        false;
#endif
    
    static constexpr bool HAS_SDF = true;
    static constexpr bool HAS_ISR = true;
    static constexpr bool HAS_COMPUTE = true;
    static constexpr bool HAS_GRAPHICS = true;
};

struct AppConfig {
  bool enableValidation;
};

class App {
 public:
  explicit App(const AppConfig& cfg);
  int run();
 private:
  bool validation;
};

} // namespace reactor
