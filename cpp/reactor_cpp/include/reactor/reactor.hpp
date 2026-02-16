// =============================================================================
// REACTOR C++ SDK — Main Header
// =============================================================================
// Include this single header to get everything.
// ONE CALL: ReactorApp() initializes everything ultra-intelligently.
//
// Usage:
//   #include <reactor/reactor.hpp>
//
// Example 1 — Functional (THE SIMPLEST):
//   int main() {
//       return reactor::ReactorApp("My Game", 1280, 720,
//           []() { /* init */ },
//           [](float dt) { /* update */ },
//           []() { /* render */ }
//       );
//   }
//
// Example 2 — Class-based:
//   class MyGame : public reactor::Application {
//       void on_init() override { /* setup */ }
//       void on_update(float dt) override { /* logic */ }
//       void on_render() override { /* draw */ }
//   };
//   int main() { return MyGame().run("My Game", 1280, 720); }
//
// Example 3 — Minimal:
//   int main() { return reactor::ReactorApp("My Game"); }
// =============================================================================

#pragma once

#include "core.hpp"
#include "types.hpp"
#include "application.hpp"

// =============================================================================
// Convenience using declarations
// =============================================================================

namespace reactor {

// Version info
inline const char* version() { return reactor_version(); }
inline const char* engine_name() { return reactor_engine_name(); }

} // namespace reactor
