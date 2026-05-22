# REACTOR — Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.2.0] — 2026-05-22

### 🏗️ Architecture — Workspace + Core UE5-style Subsystems

This release transforms REACTOR from a single-crate framework into a proper
Cargo workspace, and adds the foundational subsystems that every modern
game engine needs: profiling, logging, parallel jobs, and frame-scoped
memory allocation.

### Added

#### Core Subsystems (UE5-inspired)
- **`core::profiler`** — Hierarchical profiling via `tracing`.
  - `profile_scope!("name")` macro for zero-cost instrumentation.
  - `CpuTimer` for lightweight scope timing.
  - `PerfCounter` with exponential moving average for rolling stats.
  - `begin_frame()` / `get_frame_id()` for frame-level tracking.
  - Tracy-ready: enable `tracy` feature for real-time GPU/CPU profiling.

- **`core::logging`** — Structured, layered logging.
  - `tracing-subscriber` with `fmt` layer (colored, compact).
  - `REACTOR_LOG` env var for granular control (e.g. `debug`, `reactor=trace,winit=warn`).
  - `r_info!`, `r_warn!`, `r_error!`, `r_debug!`, `r_trace!` convenience macros.
  - Idempotent `init_logger()` — safe to call multiple times.

- **`core::jobs`** — Parallel job system (rayon-backed).
  - `parallel_for(range, body)` — data-parallel loops.
  - `join(a, b)` — fork-join parallelism.
  - `scope(|s| { s.spawn(...) })` — guaranteed-completion scoped tasks.
  - `par_iter_mut(slice, body)` — parallel mutation over slices.
  - `par_chunks_mut(slice, chunk_size, body)` — cache-friendly chunked work.
  - `parallel_reduce(slice, init, fold, reduce)` — parallel fold + reduce.
  - Lazy thread pool initialization (N-1 workers, 4MB stacks).

- **`core::linear_allocator`** — Frame-scoped bump allocator.
  - `LinearAllocator` — O(1) alloc, O(1) reset, zero fragmentation.
  - `BumpArena` — typed wrapper returning `&mut T` with arena lifetime.
  - Alignment-aware allocation with configurable alignment.
  - Peak usage tracking for telemetry.
  - `allocate_slice::<T>(count)` for typed bulk allocation.

#### Workspace Configuration
- **Cargo workspace** with `reactor-vulkan` (root) + `Editor-REACTOR` as members.
- **`[workspace.dependencies]`** for shared version pinning across crates.
- **`rust-toolchain.toml`** — pins stable channel + rustfmt + clippy + rust-analyzer.
- **`.rustfmt.toml`** — consistent style (100 col width, grouped imports, wrapped comments).
- **`clippy.toml`** — engine-grade thresholds (MSRV 1.70, complexity 50, args 10).

#### Cleanup Scripts
- **`cleanup.ps1`** — PowerShell script to delete 14 legacy dead-code files.
- **`cleanup.sh`** — Bash equivalent for Linux/macOS.

### Changed

- **`Cargo.toml`** — bumped to `1.2.0`, added workspace config, new dependencies:
  `tracing`, `tracing-subscriber`, `rayon`, `parking_lot`.
- **`src/lib.rs`** — complete rewrite:
  - Removed all 14 legacy `pub mod` declarations (dead code).
  - Removed all legacy re-exports (`pub use reactor::Reactor`, etc.).
  - Removed all `*New` suffixes from modular re-exports.
  - All re-exports are now canonical: `VulkanContext`, `Swapchain`, `Mesh`, etc.
- **`src/core/mod.rs`** — added exports for `profiler`, `logging`, `jobs`, `linear_allocator`.
- **`Fases.md`** — marked Fase 0.2, 0.3, and new Fase 1.4 as complete.

### Verified

- All 14 legacy files confirmed as **dead code** via grep across entire codebase:
  - 0 references in examples, editor, or other modules.
  - Only referenced in `lib.rs` declarations (now removed).
- No `*New` type used anywhere outside `lib.rs` — safe to remove.
- Prelude unchanged — all examples continue to work without modification.

### Migration Guide

**For users of `reactor-vulkan`:**

If you imported types directly from the root (not via `prelude`), the `*New`
suffix is gone:

```rust
// Before (v1.1.0)
use reactor_vulkan::{VulkanContextNew, PipelineNew, MeshNew};

// After (v1.2.0)
use reactor_vulkan::{VulkanContext, Pipeline, Mesh};
```

If you use the prelude (recommended), no changes needed:
```rust
use reactor_vulkan::prelude::*; // Still works exactly the same
```

**For contributors:**

1. Run `.\cleanup.ps1` (Windows) or `bash cleanup.sh` (Linux/macOS) to delete
   the 14 legacy files that are no longer compiled.
2. Run `cargo fmt --all` to apply the new formatting rules.
3. Run `cargo clippy --workspace -- -D warnings` to check for lint issues.

---

## [1.1.0] — 2026-05-01

### Removed
- C ABI (`cpp/reactor_c_api/`) — 3300 LOC deleted.
- C++ SDK (`cpp/reactor_cpp/`) — 1477 LOC deleted.
- C++ examples + CMake build artifacts (~2 GB).
- `vcpkg.json`, `docs/cpp-guide.md`, `docs/cpp_editor_parity_roadmap.md`.

### Added
- `Fases.md` — complete roadmap to SDK v2.0.
- 100% Rust-only development path.

---

## [1.0.5] — 2026-02-01

### Added
- Declarative FrameGraph (forward / deferred presets, auto-barriers).
- ECS with component queries (Transform, MeshRenderer, Light, Camera, RigidBody).
- PBR metallic/roughness workflow + material instances + emissive + alpha modes.
- Telemetry (FPS, draw calls, triangles, VRAM, memory budget).
- Compute pipeline (dispatch + barriers).
- Play-mode bridge for future editor.
- Scene serialization to JSON.
- Auto-compilation of shaders via `build.rs`.
- MSAA 4x by default, auto-detected Ray Tracing.
- 3000+ FPS on RTX 3060.

---

## [1.0.0] — [1.0.4]

### Added
- Base lifecycle, input, camera, lighting, scene.
- SPIR-V embedded shaders.
- Editor REACTOR (`egui` + `egui_dock`): Viewport, Hierarchy, Inspector, Console, Asset Browser.

---

## [0.4.x]

### Added
- Initial Rust version.
- Vulkan 1.3 base.
