#pragma once
#include <vulkan/vulkan.h>
#include <vector>
#include <optional>
#include <memory>
namespace reactor {
class MemoryAllocator;
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
  uint32_t graphicsQueueFamily() const { return indices.graphics.value(); }
  QueueFamilyIndices queueFamilyIndices() const { return indices; }
  std::shared_ptr<MemoryAllocator> allocator() const { return alloc; }
 private:
  bool validation;
  VkInstance inst{};
  VkPhysicalDevice phys{};
  VkDevice dev{};
  VkQueue gfxQueue{};
  QueueFamilyIndices indices;
  std::shared_ptr<MemoryAllocator> alloc;
  void createInstance();
  void pickPhysicalDevice();
  void createDevice();
  QueueFamilyIndices findQueueFamilies(VkPhysicalDevice d);
};
}

