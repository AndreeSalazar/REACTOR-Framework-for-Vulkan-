// =============================================================================
// REACTOR Build Script — Auto-compile GLSL shaders to SPIR-V
// =============================================================================
// Requires `glslc` (from Vulkan SDK) to be in PATH.
// Shaders are only recompiled when source files change.
// =============================================================================

use std::process::Command;
use std::path::Path;

fn compile_shader(src: &str, dst: &str) {
    let src_path = Path::new(src);
    let dst_path = Path::new(dst);

    // Tell cargo to rerun if the source changes
    println!("cargo:rerun-if-changed={}", src);

    // Skip if output is newer than source
    if dst_path.exists() {
        if let (Ok(src_meta), Ok(dst_meta)) = (src_path.metadata(), dst_path.metadata()) {
            if let (Ok(src_time), Ok(dst_time)) = (src_meta.modified(), dst_meta.modified()) {
                if dst_time >= src_time {
                    return;
                }
            }
        }
    }

    println!("cargo:warning=Compiling shader: {} -> {}", src, dst);

    let status = Command::new("glslc")
        .args([src, "-o", dst])
        .status();

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => {
            eprintln!("glslc failed for {} with exit code: {:?}", src, s.code());
            // Don't fail the build — shaders may already exist as .spv
        }
        Err(e) => {
            eprintln!("Could not run glslc (is Vulkan SDK installed?): {}", e);
            // Don't fail — allow building without glslc if .spv files exist
        }
    }
}

fn main() {
    // Core shaders used by examples and C API
    compile_shader("shaders/shader.vert", "shaders/vert.spv");
    compile_shader("shaders/shader.frag", "shaders/frag.spv");
    compile_shader("shaders/texture.vert", "shaders/texture_vert.spv");
    compile_shader("shaders/texture.frag", "shaders/texture_frag.spv");
}
