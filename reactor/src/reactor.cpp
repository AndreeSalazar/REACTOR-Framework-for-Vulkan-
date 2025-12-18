#include "reactor/reactor.hpp"
#include "reactor/vulkan_context.hpp"
namespace reactor {
App::App(const AppConfig& cfg) : validation(cfg.enableValidation) {}
int App::run() {
  VulkanContext ctx(validation);
  ctx.init();
  ctx.shutdown();
  return 0;
}
}

