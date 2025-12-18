#pragma once
#include <vulkan/vulkan.h>
#include <vector>
#include <optional>
namespace reactor {
struct QueueFamilyIndices {
  std::optional<uint32_t> graphics;
  bool complete() const { return graphics.has_value(); }
};
class VulkanContext {
 public:
  VulkanContext(bool enableValidation);
  void init();
  void shutdown();
  VkInstance instance() const { return inst; }
  VkDevice device() const { return dev; }
  VkPhysicalDevice physical() const { return phys; }
  VkQueue graphicsQueue() const { return gfxQueue; }
 private:
  bool validation;
  VkInstance inst{};
  VkPhysicalDevice phys{};
  VkDevice dev{};
  VkQueue gfxQueue{};
  QueueFamilyIndices indices;
  void createInstance();
  void pickPhysicalDevice();
  void createDevice();
  QueueFamilyIndices findQueueFamilies(VkPhysicalDevice d);
};
}

