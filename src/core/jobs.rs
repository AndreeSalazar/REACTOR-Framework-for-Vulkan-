// =============================================================================
// REACTOR Job System — UE5 TaskGraph-style parallel execution
// =============================================================================
// A thin wrapper over `rayon` that provides:
//   - JobSystem::spawn(task) — fire-and-forget parallel task
//   - JobSystem::parallel_for(range, body) — data-parallel loops
//   - JobSystem::join(a, b) — fork-join parallelism
//   - Scoped tasks that are guaranteed to complete before returning
//
// Thread pool is initialized lazily with N-1 workers (one per logical core,
// leaving one for the main/render thread).
//
// =============================================================================

use rayon::prelude::*;
use std::ops::Range;
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize the global thread pool with optimal settings.
/// Called automatically on first use, but can be called explicitly
/// at startup to control initialization timing.
pub fn init_job_system() {
    INIT.call_once(|| {
        let num_threads = std::thread::available_parallelism()
            .map(|n| n.get().saturating_sub(1).max(1))
            .unwrap_or(4);

        rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .thread_name(|i| format!("reactor-worker-{}", i))
            .stack_size(4 * 1024 * 1024) // 4MB stack for deep recursion
            .build_global()
            .expect("Failed to initialize rayon thread pool");

        tracing::info!(
            target: "reactor::jobs",
            threads = num_threads,
            "Job system initialized"
        );
    });
}

// =============================================================================
// Core primitives
// =============================================================================

/// Execute two closures in parallel, returning both results.
/// UE5 calls this "Fork/Join". Zero-cost when there's nothing to parallelize.
///
/// # Examples
/// ```ignore
/// let (left, right) = jobs::join(
///     || process_left_half(),
///     || process_right_half(),
/// );
/// ```
#[inline]
pub fn join<A, B, RA, RB>(a: A, b: B) -> (RA, RB)
where
    A: FnOnce() -> RA + Send,
    B: FnOnce() -> RB + Send,
    RA: Send,
    RB: Send,
{
    init_job_system();
    rayon::join(a, b)
}

/// Execute a closure over a range of indices in parallel.
/// The closure receives the index and can mutate captured state via
/// thread-local storage or atomics.
///
/// # Examples
/// ```ignore
/// jobs::parallel_for(0..entities.len(), |i| {
///     update_entity(&mut entities[i]);
/// });
/// ```
#[inline]
pub fn parallel_for<F>(range: Range<usize>, body: F)
where
    F: Fn(usize) + Send + Sync,
{
    init_job_system();
    range.into_par_iter().for_each(body);
}

/// Execute a closure over a slice in parallel, mutating each element.
///
/// # Examples
/// ```ignore
/// jobs::par_iter_mut(&mut transforms, |t| {
///     t.position += t.velocity * dt;
/// });
/// ```
#[inline]
pub fn par_iter_mut<'a, T, F>(slice: &mut [T], body: F)
where
    T: Send + 'a,
    F: Fn(&mut T) + Send + Sync,
{
    init_job_system();
    slice.par_iter_mut().for_each(body);
}

/// Execute a closure over a slice in parallel (read-only).
#[inline]
pub fn par_iter<'a, T, F>(slice: &[T], body: F)
where
    T: Send + Sync + 'a,
    F: Fn(&T) + Send + Sync,
{
    init_job_system();
    slice.par_iter().for_each(body);
}

/// Execute a closure over chunks of a slice for better cache locality.
/// Ideal when per-element work is too small to amortize rayon overhead.
///
/// # Examples
/// ```ignore
/// jobs::par_chunks_mut(&mut pixels, 64, |chunk| {
///     for pixel in chunk { pixel.apply_filter(); }
/// });
/// ```
#[inline]
pub fn par_chunks_mut<'a, T, F>(slice: &mut [T], chunk_size: usize, body: F)
where
    T: Send + 'a,
    F: Fn(&mut [T]) + Send + Sync,
{
    init_job_system();
    slice.par_chunks_mut(chunk_size).for_each(body);
}

// =============================================================================
// Scoped tasks (guaranteed completion)
// =============================================================================

/// Execute a closure that can spawn scoped tasks.
/// All spawned tasks are guaranteed to complete before this returns.
/// Use for complex parallel algorithms that need nested parallelism.
///
/// # Examples
/// ```ignore
/// jobs::scope(|s| {
///     s.spawn(|_| compute_lighting_pass_1());
///     s.spawn(|_| compute_lighting_pass_2());
///     s.spawn(|_| compute_shadows());
/// });
/// // All three tasks are done here.
/// ```
#[inline]
pub fn scope<'scope, OP, R>(op: OP) -> R
where
    OP: FnOnce(&rayon::Scope<'scope>) -> R + Send,
    R: Send,
{
    init_job_system();
    rayon::scope(op)
}

// =============================================================================
// Async-style task spawning
// =============================================================================

/// Spawn a task that runs in the background. Returns a handle to await
/// the result. Use for truly independent work (asset loading, I/O, etc).
///
/// For render-thread dependencies, prefer `scope()` or `join()`.
#[inline]
pub fn spawn_async<F, R>(task: F) -> std::thread::JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    std::thread::spawn(task)
}

// =============================================================================
// Utility: parallel reduce
// =============================================================================

/// Parallel fold + reduce over a slice. Useful for computing sums,
/// bounding boxes, visibility counts, etc.
///
/// # Examples
/// ```ignore
/// let total_triangles: u64 = jobs::parallel_reduce(
///     &meshes,
///     || 0u64,
///     |acc, mesh| acc + mesh.triangle_count,
///     |a, b| a + b,
/// );
/// ```
#[inline]
pub fn parallel_reduce<'a, T, ID, F, R, A>(
    slice: &'a [T],
    identity: ID,
    fold_op: F,
    reduce_op: R,
) -> A
where
    T: Send + Sync + 'a,
    ID: Fn() -> A + Send + Sync + Clone,
    F: Fn(A, &'a T) -> A + Send + Sync,
    R: Fn(A, A) -> A + Send + Sync,
    A: Send,
{
    init_job_system();
    let identity_fn = identity;
    slice
        .par_iter()
        .fold(identity_fn.clone(), fold_op)
        .reduce(identity_fn, reduce_op)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_for() {
        use std::sync::atomic::{AtomicI32, Ordering};

        let v = (0..100).map(|_| AtomicI32::new(0)).collect::<Vec<_>>();
        parallel_for(0..v.len(), |i| {
            v[i].store(i as i32 * 2, Ordering::Relaxed);
        });
        assert_eq!(v[50].load(Ordering::Relaxed), 100);
    }

    #[test]
    fn test_join() {
        let (a, b) = join(|| 2 + 2, || 3 * 3);
        assert_eq!(a, 4);
        assert_eq!(b, 9);
    }
}
