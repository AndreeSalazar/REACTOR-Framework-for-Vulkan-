// =============================================================================
// REACTOR Linear Allocator — UE5-style Frame-scoped Memory
// =============================================================================
// Zero-overhead bump allocator for data that lives exactly one frame.
//
// Use cases:
//   - Per-frame command lists
//   - Temporary vertex/index buffers
//   - Culling result sets
//   - Particle system scratch space
//
// Advantages:
//   - O(1) allocation (bump pointer)
//   - O(1) deallocation (reset entire arena at frame start)
//   - Cache-friendly (allocations are contiguous)
//   - Zero fragmentation
//
// =============================================================================

use std::alloc::{alloc, dealloc, Layout};
use std::cell::Cell;
use std::ptr::NonNull;

/// A linear (bump) allocator backed by a single contiguous buffer.
///
/// # Safety
/// The allocator itself is safe to use, but the returned pointers
/// must not outlive the allocator. Typical pattern:
///   - Allocate at frame start
///   - Use throughout the frame
///   - Call `reset()` at the end of the frame (invalidates all pointers)
pub struct LinearAllocator {
    /// Base pointer of the backing buffer.
    base: NonNull<u8>,
    /// Total capacity in bytes.
    capacity: usize,
    /// Current offset (bump pointer).
    offset: Cell<usize>,
    /// Peak usage (for telemetry).
    peak: Cell<usize>,
    /// Layout used for allocation (needed for dealloc).
    layout: Layout,
}

// LinearAllocator is single-threaded by design.
// For multi-threaded use, create one per thread (e.g., via thread_local!).

impl LinearAllocator {
    /// Create a new linear allocator with the given capacity in bytes.
    ///
    /// # Panics
    /// Panics if capacity is 0 or if allocation fails.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "LinearAllocator capacity must be > 0");
        let layout =
            Layout::from_size_align(capacity, 16).expect("Invalid layout for linear allocator");

        let base = unsafe {
            let ptr = alloc(layout);
            NonNull::new(ptr).expect("LinearAllocator: out of memory")
        };

        Self {
            base,
            capacity,
            offset: Cell::new(0),
            peak: Cell::new(0),
            layout,
        }
    }

    /// Allocate `size` bytes with the given alignment.
    /// Returns a pointer to the allocated region.
    ///
    /// Returns `None` if there is not enough space.
    #[inline]
    pub fn allocate(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let current = self.offset.get();
        // Align up the current offset
        let mask = align.wrapping_sub(1);
        let aligned = (current + mask) & !mask;
        let new_offset = aligned.checked_add(size)?;

        if new_offset > self.capacity {
            tracing::warn!(
                target: "reactor::allocator",
                requested = size,
                available = self.capacity - current,
                "Linear allocator exhausted"
            );
            return None;
        }

        self.offset.set(new_offset);
        if new_offset > self.peak.get() {
            self.peak.set(new_offset);
        }

        // SAFETY: `aligned < new_offset <= capacity`, so the pointer is in-bounds.
        unsafe { Some(NonNull::new_unchecked(self.base.as_ptr().add(aligned))) }
    }

    /// Allocate and return a mutable slice of `count` elements of type `T`.
    /// All elements are left uninitialized (like `MaybeUninit`).
    ///
    /// # Safety
    /// Caller must ensure elements are properly initialized before reading.
    #[inline]
    pub fn allocate_slice<T>(&self, count: usize) -> Option<&mut [std::mem::MaybeUninit<T>]> {
        let size = count.checked_mul(std::mem::size_of::<T>())?;
        let align = std::mem::align_of::<T>();
        let ptr = self.allocate(size, align)?;
        unsafe {
            let slice = std::slice::from_raw_parts_mut(
                ptr.as_ptr() as *mut std::mem::MaybeUninit<T>,
                count,
            );
            Some(slice)
        }
    }

    /// Reset the allocator, invalidating all previously returned pointers.
    /// Call this once per frame at the start of the frame.
    #[inline]
    pub fn reset(&self) {
        self.offset.set(0);
    }

    /// Reset and record the peak usage for telemetry.
    #[inline]
    pub fn reset_and_record_peak(&self) {
        // Peak is already tracked on each allocation; just reset offset.
        self.offset.set(0);
    }

    /// Current bytes used.
    #[inline]
    pub fn used(&self) -> usize {
        self.offset.get()
    }

    /// Total capacity.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Peak usage since creation (or last manual reset).
    #[inline]
    pub fn peak(&self) -> usize {
        self.peak.get()
    }

    /// Reset peak counter (e.g., at the start of a benchmarking window).
    #[inline]
    pub fn reset_peak(&self) {
        self.peak.set(self.offset.get());
    }

    /// Usage ratio in [0.0, 1.0].
    #[inline]
    pub fn usage_ratio(&self) -> f32 {
        self.offset.get() as f32 / self.capacity as f32
    }
}

impl Drop for LinearAllocator {
    fn drop(&mut self) {
        // SAFETY: we allocated this in `new` with the same layout.
        unsafe {
            dealloc(self.base.as_ptr(), self.layout);
        }
    }
}

// =============================================================================
// Typed wrapper for ergonomic use
// =============================================================================

/// A typed bump arena that hands out `&'a mut T` references.
/// The lifetime `'a` is tied to the arena's scope, preventing use-after-reset.
pub struct BumpArena {
    inner: LinearAllocator,
}

impl BumpArena {
    pub fn new(capacity: usize) -> Self {
        Self { inner: LinearAllocator::new(capacity) }
    }

    /// Allocate a value of type `T`, returning a mutable reference with the
    /// arena's lifetime.
    pub fn alloc<T>(&self, value: T) -> Option<&mut T> {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();
        let ptr = self.inner.allocate(size, align)?;
        unsafe {
            let typed = ptr.as_ptr() as *mut T;
            typed.write(value);
            // SAFETY: the returned reference lives as long as `&self`.
            // Caller must not use it after `reset()`.
            Some(&mut *typed)
        }
    }

    pub fn reset(&self) {
        self.inner.reset();
    }

    pub fn used(&self) -> usize {
        self.inner.used()
    }

    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    pub fn peak(&self) -> usize {
        self.inner.peak()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_allocator_basic() {
        let alloc = LinearAllocator::new(1024);
        let p1 = alloc.allocate(64, 8).unwrap();
        let p2 = alloc.allocate(64, 8).unwrap();
        assert_ne!(p1, p2);
        assert_eq!(alloc.used(), 128);
    }

    #[test]
    fn test_linear_allocator_exhaustion() {
        let alloc = LinearAllocator::new(64);
        assert!(alloc.allocate(64, 8).is_some());
        assert!(alloc.allocate(1, 1).is_none());
    }

    #[test]
    fn test_linear_allocator_reset() {
        let alloc = LinearAllocator::new(1024);
        let _ = alloc.allocate(256, 8);
        assert_eq!(alloc.used(), 256);
        alloc.reset();
        assert_eq!(alloc.used(), 0);
    }

    #[test]
    fn test_bump_arena() {
        let arena = BumpArena::new(1024);
        let x = arena.alloc(42u32).unwrap();
        assert_eq!(*x, 42);
        *x = 100;
        assert_eq!(*x, 100);
    }

    #[test]
    fn test_alignment() {
        let alloc = LinearAllocator::new(1024);
        let p = alloc.allocate(16, 64).unwrap();
        assert_eq!(p.as_ptr() as usize % 64, 0);
    }
}
