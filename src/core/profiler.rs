// =============================================================================
// REACTOR Profiler — UE5-style Hierarchical Profiling
// =============================================================================
// Integrates with `tracing` crate for zero-cost instrumentation.
// Optional: enable `tracy` feature for real-time GPU/CPU profiling
// via Tracy Profiler (https://github.com/wolfpld/tracy).
//
// Usage:
//   profile_scope!("render_forward");
//   profile_scope!("cull_objects", count = scene.objects.len());
//
// =============================================================================

use std::time::{Duration, Instant};

// Thread-local frame counter for profiling frames.
thread_local! {
    static FRAME_COUNTER: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Advance the frame counter. Call once per frame at the start of `update()`.
#[inline]
pub fn begin_frame() {
    FRAME_COUNTER.with(|c| c.set(c.get() + 1));
    tracing::info_span!("frame", id = get_frame_id()).in_scope(|| {});
}

/// Get current frame id (thread-local).
#[inline]
pub fn get_frame_id() -> u64 {
    FRAME_COUNTER.with(|c| c.get())
}

/// Guard that emits a tracing span on drop. Zero-cost when tracing is disabled.
pub struct ProfileGuard {
    _span: tracing::span::EnteredSpan,
}

/// Profile a named scope. Emits a tracing span at `info` level.
///
/// # Examples
/// ```ignore
/// profile_scope!("update_physics");
/// profile_scope!("cull", count = n_objects);
/// ```
#[macro_export]
macro_rules! profile_scope {
    ($name:expr) => {
        let _profile_guard = tracing::info_span!($name).entered();
    };
    ($name:expr, $($fields:tt)*) => {
        let _profile_guard = tracing::info_span!($name, $($fields)*).entered();
    };
}

/// Profile a scope at `debug` level (cheaper, less verbose).
#[macro_export]
macro_rules! profile_scope_debug {
    ($name:expr $(, $($fields:tt)*)?) => {
        let _profile_guard = tracing::debug_span!($name $(, $($fields)*)?).entered();
    };
}

/// Profile a GPU pass. Use this to mark render passes for GPU timing.
/// When the `tracy` feature is enabled, this emits a zone that Tracy
/// can correlate with GPU timestamps.
#[macro_export]
macro_rules! profile_gpu {
    ($name:expr $(, $($fields:tt)*)?) => {
        let _gpu_span = tracing::info_span!("gpu", name = $name $(, $($fields)*)?).entered();
    };
}

// =============================================================================
// CPU Timer — lightweight scope timer without tracing overhead
// =============================================================================

/// A simple CPU timer for benchmarking. Useful in hot paths where you
/// want to measure elapsed time without creating a tracing span.
///
/// # Examples
/// ```
/// use reactor_vulkan::core::profiler::CpuTimer;
/// let mut timer = CpuTimer::new("my_scope");
/// // ... work ...
/// let elapsed = timer.stop();
/// ```
pub struct CpuTimer {
    name: &'static str,
    start: Instant,
}

impl CpuTimer {
    #[inline]
    pub fn new(name: &'static str) -> Self {
        Self { name, start: Instant::now() }
    }

    /// Stop the timer and return elapsed duration. Logs at `debug` level.
    #[inline]
    pub fn stop(self) -> Duration {
        let elapsed = self.start.elapsed();
        tracing::debug!(target: "reactor::timer", name = self.name, ?elapsed);
        elapsed
    }

    /// Stop the timer silently.
    #[inline]
    pub fn stop_silent(self) -> Duration {
        self.start.elapsed()
    }

    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

// =============================================================================
// Statistics accumulator
// =============================================================================

/// Rolling statistics for a named counter (FPS, draw calls, etc.).
/// Uses exponential moving average for smoothing.
#[derive(Debug, Clone)]
pub struct PerfCounter {
    pub name: &'static str,
    pub last: f64,
    pub avg: f64,
    pub min: f64,
    pub max: f64,
    pub alpha: f64,
    first: bool,
}

impl PerfCounter {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            last: 0.0,
            avg: 0.0,
            min: f64::MAX,
            max: f64::MIN,
            alpha: 0.1,
            first: true,
        }
    }

    pub fn with_alpha(mut self, alpha: f64) -> Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }

    pub fn push(&mut self, value: f64) {
        self.last = value;
        if self.first {
            self.avg = value;
            self.min = value;
            self.max = value;
            self.first = false;
        } else {
            self.avg = self.alpha * value + (1.0 - self.alpha) * self.avg;
            if value < self.min {
                self.min = value;
            }
            if value > self.max {
                self.max = value;
            }
        }
    }

    pub fn reset(&mut self) {
        self.last = 0.0;
        self.avg = 0.0;
        self.min = f64::MAX;
        self.max = f64::MIN;
        self.first = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perf_counter() {
        let mut c = PerfCounter::new("fps");
        c.push(60.0);
        c.push(62.0);
        c.push(58.0);
        assert!((c.last - 58.0).abs() < f64::EPSILON);
        assert!(c.avg > 58.0 && c.avg < 62.0);
        assert!((c.min - 58.0).abs() < f64::EPSILON);
        assert!((c.max - 62.0).abs() < f64::EPSILON);
    }
}
