// =============================================================================
// REACTOR GPU Memory Budget — VRAM Monitoring
// =============================================================================
// Queries the GPU's memory budget at runtime using VK_EXT_memory_budget.
// This allows the engine to:
// - Display VRAM usage in the profiler/console.
// - Implement intelligent LOD scaling (reduce textures when VRAM is low).
// - Warn before critical OutOfMemory (OOM) situations.
//
// When VK_EXT_memory_budget is not available, falls back to static heap sizes.
// =============================================================================

use ash::vk;
use std::fmt;

/// Memory budget information for a single GPU memory heap.
#[derive(Debug, Clone, Copy)]
pub struct HeapBudget {
    /// Index of this heap.
    pub heap_index: u32,
    /// Total budget (in bytes) — the amount the OS allows this process to use.
    pub budget: u64,
    /// Current usage (in bytes) — how much this process has allocated.
    pub usage: u64,
    /// Total physical heap size (from VkPhysicalDeviceMemoryProperties).
    pub heap_size: u64,
    /// Whether this heap is device-local (VRAM).
    pub is_device_local: bool,
}

impl HeapBudget {
    /// Free memory available (budget - usage), clamped to 0.
    #[inline]
    pub fn free(&self) -> u64 {
        self.budget.saturating_sub(self.usage)
    }

    /// Usage ratio (0.0 = empty, 1.0 = full).
    #[inline]
    pub fn usage_ratio(&self) -> f32 {
        if self.budget == 0 {
            return 0.0;
        }
        self.usage as f32 / self.budget as f32
    }

    /// Convert bytes to megabytes (rounded).
    #[inline]
    pub fn usage_mb(&self) -> u64 {
        self.usage / (1024 * 1024)
    }

    /// Convert bytes to megabytes (rounded).
    #[inline]
    pub fn budget_mb(&self) -> u64 {
        self.budget / (1024 * 1024)
    }

    /// Convert bytes to megabytes (rounded).
    #[inline]
    pub fn free_mb(&self) -> u64 {
        self.free() / (1024 * 1024)
    }
}

/// Aggregate GPU memory budget across all heaps.
#[derive(Debug, Clone)]
pub struct GpuMemoryBudget {
    /// Per-heap budget data.
    pub heaps: Vec<HeapBudget>,
    /// Whether the data was queried from VK_EXT_memory_budget (true)
    /// or estimated from static heap sizes (false).
    pub has_dynamic_budget: bool,
}

impl GpuMemoryBudget {
    /// Total VRAM budget (sum of device-local heaps).
    pub fn total_vram_budget(&self) -> u64 {
        self.heaps
            .iter()
            .filter(|h| h.is_device_local)
            .map(|h| h.budget)
            .sum()
    }

    /// Total VRAM usage (sum of device-local heaps).
    pub fn total_vram_usage(&self) -> u64 {
        self.heaps
            .iter()
            .filter(|h| h.is_device_local)
            .map(|h| h.usage)
            .sum()
    }

    /// Total free VRAM.
    pub fn total_vram_free(&self) -> u64 {
        self.total_vram_budget()
            .saturating_sub(self.total_vram_usage())
    }

    /// Total VRAM budget in MB.
    pub fn total_vram_budget_mb(&self) -> u64 {
        self.total_vram_budget() / (1024 * 1024)
    }

    /// Total VRAM usage in MB.
    pub fn total_vram_usage_mb(&self) -> u64 {
        self.total_vram_usage() / (1024 * 1024)
    }

    /// Total free VRAM in MB.
    pub fn total_vram_free_mb(&self) -> u64 {
        self.total_vram_free() / (1024 * 1024)
    }

    /// Overall usage ratio of device-local memory.
    pub fn vram_usage_ratio(&self) -> f32 {
        let budget = self.total_vram_budget();
        if budget == 0 {
            return 0.0;
        }
        self.total_vram_usage() as f32 / budget as f32
    }

    /// Returns true if VRAM pressure is high (usage > 85% of budget).
    pub fn is_vram_pressure_high(&self) -> bool {
        self.vram_usage_ratio() > 0.85
    }

    /// Returns true if VRAM is critically low (usage > 95% of budget).
    pub fn is_vram_critical(&self) -> bool {
        self.vram_usage_ratio() > 0.95
    }
}

impl fmt::Display for GpuMemoryBudget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "GPU Memory Budget ({})",
            if self.has_dynamic_budget {
                "dynamic"
            } else {
                "static"
            }
        )?;
        for h in &self.heaps {
            let kind = if h.is_device_local { "VRAM" } else { " RAM" };
            writeln!(
                f,
                "  Heap {}: [{}] {}/{} MB used ({:.1}% — {} MB free)",
                h.heap_index,
                kind,
                h.usage_mb(),
                h.budget_mb(),
                h.usage_ratio() * 100.0,
                h.free_mb(),
            )?;
        }
        Ok(())
    }
}

/// Query the memory budget from the physical device.
///
/// If `VK_EXT_memory_budget` is available, returns real-time budget/usage data.
/// Otherwise, falls back to static heap sizes with usage = 0.
pub fn query_memory_budget(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    has_memory_budget_ext: bool,
) -> GpuMemoryBudget {
    let mem_props = unsafe { instance.get_physical_device_memory_properties(physical_device) };

    if has_memory_budget_ext {
        // Use VK_EXT_memory_budget for real-time budget data
        let mut budget_props = vk::PhysicalDeviceMemoryBudgetPropertiesEXT::default();
        let mut mem_props2 =
            vk::PhysicalDeviceMemoryProperties2::default().push_next(&mut budget_props);

        unsafe {
            instance.get_physical_device_memory_properties2(physical_device, &mut mem_props2);
        }

        let heap_count = mem_props2.memory_properties.memory_heap_count as usize;
        let heaps: Vec<HeapBudget> = (0..heap_count)
            .map(|i| {
                let heap = mem_props2.memory_properties.memory_heaps[i];
                HeapBudget {
                    heap_index: i as u32,
                    budget: budget_props.heap_budget[i],
                    usage: budget_props.heap_usage[i],
                    heap_size: heap.size,
                    is_device_local: heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL),
                }
            })
            .collect();

        GpuMemoryBudget { heaps, has_dynamic_budget: true }
    } else {
        // Fallback: static heap sizes, no usage data
        let heap_count = mem_props.memory_heap_count as usize;
        let heaps: Vec<HeapBudget> = (0..heap_count)
            .map(|i| {
                let heap = mem_props.memory_heaps[i];
                HeapBudget {
                    heap_index: i as u32,
                    budget: heap.size,
                    usage: 0,
                    heap_size: heap.size,
                    is_device_local: heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL),
                }
            })
            .collect();

        GpuMemoryBudget { heaps, has_dynamic_budget: false }
    }
}
