# =============================================================================
# REACTOR — Legacy Module Cleanup Script
# =============================================================================
# Deletes the 14 legacy files that were superseded by the modular architecture.
# These files are no longer referenced by lib.rs or any other module.
#
# Run this script from the project root:
#   .\cleanup.ps1
# =============================================================================

$legacy = @(
    "src\vulkan_context.rs",
    "src\swapchain.rs",
    "src\pipeline.rs",
    "src\buffer.rs",
    "src\vertex.rs",
    "src\mesh.rs",
    "src\material.rs",
    "src\input.rs",
    "src\ecs.rs",
    "src\ray_tracing.rs",
    "src\scene.rs",
    "src\gpu_detector.rs",
    "src\cpu_detector.rs",
    "src\resolution_detector.rs"
)

Write-Host ""
Write-Host "REACTOR — Cleaning up legacy modules..." -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host ""

$deleted = 0
$not_found = 0

foreach ($file in $legacy) {
    if (Test-Path $file) {
        $size = (Get-Item $file).Length
        Remove-Item $file -Force
        Write-Host "  [DELETED]  $file ($($size) bytes)" -ForegroundColor Green
        $deleted++
    } else {
        Write-Host "  [SKIP]     $file (not found)" -ForegroundColor DarkGray
        $not_found++
    }
}

Write-Host ""
Write-Host "Done. Deleted $deleted files, $not_found not found." -ForegroundColor Cyan
Write-Host ""
Write-Host "Verify with: cargo check --workspace" -ForegroundColor Yellow
Write-Host ""
