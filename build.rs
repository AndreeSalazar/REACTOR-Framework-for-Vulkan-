// =============================================================================
// REACTOR Build Script — Auto-compile GLSL shaders to SPIR-V
// =============================================================================
// Requires `glslc` (from Vulkan SDK) to be in PATH.
// Shaders are only recompiled when source files change.
// =============================================================================

use std::path::Path;
use std::process::Command;

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

    let status = Command::new("glslc").args([src, "-o", dst]).status();

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
    // Compilar recursivamente TODOS los shaders GLSL en el directorio shaders/
    let shaders_dir = Path::new("shaders");

    if !shaders_dir.exists() {
        eprintln!("cargo:warning=Shaders directory not found: shaders/");
        return;
    }

    // Recorrer recursivamente todos los archivos .vert y .frag
    let mut compiled = 0;

    for entry in walkdir::WalkDir::new(shaders_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Solo procesar archivos con extensión .vert o .frag
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy();
            if ext_str == "vert" || ext_str == "frag" {
                let src = path.to_string_lossy().to_string();
                let dst = src.replace(&format!(".{}", ext_str), ".spv");

                compile_shader(&src, &dst);
                compiled += 1;

                // Also generate aliases for specific files used by built-in loaders:
                let filename = path.file_name().unwrap().to_string_lossy();
                if filename == "shader.vert" {
                    compile_shader(&src, "shaders/vert.spv");
                    compiled += 1;
                } else if filename == "shader.frag" {
                    compile_shader(&src, "shaders/frag.spv");
                    compiled += 1;
                } else if filename == "texture.vert" {
                    compile_shader(&src, "shaders/texture_vert.spv");
                    compiled += 1;
                } else if filename == "texture.frag" {
                    compile_shader(&src, "shaders/texture_frag.spv");
                    compiled += 1;
                }
            }
        }
    }

    println!("cargo:warning=Compiled {} shaders (including aliases)", compiled);
}
