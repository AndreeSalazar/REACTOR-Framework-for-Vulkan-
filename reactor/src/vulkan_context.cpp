#include "reactor/vulkan_context.hpp"
#include "reactor/memory_allocator.hpp"
#include <stdexcept>
#include <cstring>
#ifdef REACTOR_HAS_WINDOW
#include <GLFW/glfw3.h>
#endif
namespace reactor {
static const char* kValidationLayer = "VK_LAYER_KHRONOS_validation";
VulkanContext::VulkanContext(bool enableValidation) : validation(enableValidation) {}
void VulkanContext::init() {
  createInstance();
  pickPhysicalDevice();
  createDevice();
  alloc = std::make_shared<MemoryAllocator>(dev, phys);
}
void VulkanContext::shutdown() {
  if (dev) vkDeviceWaitIdle(dev);
  if (dev) vkDestroyDevice(dev, nullptr);
  if (inst) vkDestroyInstance(inst, nullptr);
}
void VulkanContext::createInstance() {
  VkApplicationInfo appInfo{};
  appInfo.sType = VK_STRUCTURE_TYPE_APPLICATION_INFO;
  appInfo.pApplicationName = "reactor";
  appInfo.applicationVersion = VK_MAKE_VERSION(0,1,0);
  appInfo.pEngineName = "reactor";
  appInfo.engineVersion = VK_MAKE_VERSION(0,1,0);
  appInfo.apiVersion = VK_API_VERSION_1_3;
  
  std::vector<const char*> layers;
  if (validation) layers.push_back(kValidationLayer);
  
  // Obtener extensiones requeridas por GLFW (para window surface)
  std::vector<const char*> extensions;
#ifdef REACTOR_HAS_WINDOW
  uint32_t glfwExtensionCount = 0;
  const char** glfwExtensions = glfwGetRequiredInstanceExtensions(&glfwExtensionCount);
  for (uint32_t i = 0; i < glfwExtensionCount; i++) {
    extensions.push_back(glfwExtensions[i]);
  }
#endif
  
  VkInstanceCreateInfo ci{};
  ci.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO;
  ci.pApplicationInfo = &appInfo;
  ci.enabledLayerCount = static_cast<uint32_t>(layers.size());
  ci.ppEnabledLayerNames = layers.empty() ? nullptr : layers.data();
  ci.enabledExtensionCount = static_cast<uint32_t>(extensions.size());
  ci.ppEnabledExtensionNames = extensions.empty() ? nullptr : extensions.data();
  
  if (vkCreateInstance(&ci, nullptr, &inst) != VK_SUCCESS) throw std::runtime_error("vkCreateInstance failed");
}
QueueFamilyIndices VulkanContext::findQueueFamilies(VkPhysicalDevice d) {
  QueueFamilyIndices r;
  uint32_t count = 0;
  vkGetPhysicalDeviceQueueFamilyProperties(d, &count, nullptr);
  std::vector<VkQueueFamilyProperties> props(count);
  vkGetPhysicalDeviceQueueFamilyProperties(d, &count, props.data());
  for (uint32_t i=0;i<count;i++) {
    if (props[i].queueFlags & VK_QUEUE_GRAPHICS_BIT) { r.graphics = i; break; }
  }
  return r;
}
void VulkanContext::pickPhysicalDevice() {
  uint32_t count = 0;
  vkEnumeratePhysicalDevices(inst, &count, nullptr);
  if (count == 0) throw std::runtime_error("no physical devices");
  std::vector<VkPhysicalDevice> devs(count);
  vkEnumeratePhysicalDevices(inst, &count, devs.data());
  for (auto d : devs) {
    auto q = findQueueFamilies(d);
    if (q.complete()) { phys = d; indices = q; break; }
  }
  if (!phys) throw std::runtime_error("no suitable device");
}
void VulkanContext::createDevice() {
  float priority = 1.0f;
  VkDeviceQueueCreateInfo qci{};
  qci.sType = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO;
  qci.queueFamilyIndex = indices.graphics.value();
  qci.queueCount = 1;
  qci.pQueuePriorities = &priority;
  
  // Extensiones del device (swapchain para windows)
  std::vector<const char*> deviceExtensions;
#ifdef REACTOR_HAS_WINDOW
  deviceExtensions.push_back(VK_KHR_SWAPCHAIN_EXTENSION_NAME);
#endif
  
  VkDeviceCreateInfo dci{};
  dci.sType = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO;
  dci.queueCreateInfoCount = 1;
  dci.pQueueCreateInfos = &qci;
  dci.enabledExtensionCount = static_cast<uint32_t>(deviceExtensions.size());
  dci.ppEnabledExtensionNames = deviceExtensions.empty() ? nullptr : deviceExtensions.data();
  
  if (vkCreateDevice(phys, &dci, nullptr, &dev) != VK_SUCCESS) throw std::runtime_error("vkCreateDevice failed");
  vkGetDeviceQueue(dev, indices.graphics.value(), 0, &gfxQueue);
}
}

