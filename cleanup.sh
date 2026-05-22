#!/usr/bin/env bash
# =============================================================================
# REACTOR — Legacy Module Cleanup Script (Linux/macOS)
# =============================================================================
# Run from project root:  bash cleanup.sh

set -euo pipefail

LEGACY=(
    src/vulkan_context.rs
    src/swapchain.rs
    src/pipeline.rs
    src/buffer.rs
    src/vertex.rs
    src/mesh.rs
    src/material.rs
    src/input.rs
    src/ecs.rs
    src/ray_tracing.rs
    src/scene.rs
    src/gpu_detector.rs
    src/cpu_detector.rs
    src/resolution_detector.rs
)

echo ""
echo "REACTOR — Cleaning up legacy modules..."
echo "========================================="
echo ""

deleted=0
not_found=0

for f in "${LEGACY[@]}"; do
    if [ -f "$f" ]; then
        size=$(stat -f%z "$f" 2>/dev/null || stat -c%s "$f" 2>/dev/null || echo "?")
        rm -f "$f"
        echo "  [DELETED]  $f ($size bytes)"
        ((deleted++))
    else
        echo "  [SKIP]     $f (not found)"
        ((not_found++))
    fi
done

echo ""
echo "Done. Deleted $deleted files, $not_found not found."
echo ""
echo "Verify with: cargo check --workspace"
