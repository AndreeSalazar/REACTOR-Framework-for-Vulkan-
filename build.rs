// =============================================================================
// REACTOR Build Script — Auto-compile GLSL shaders to SPIR-V
// =============================================================================
// Requires `glslc` (from Vulkan SDK) to be in PATH.
//
// Estructura esperada:
//
//   shaders/
//   ├── lib/        ← snippets reutilizables (.glsl) — solo `#include`,
//   │                  no se compilan standalone
//   ├── core/       ← basic + textured (used by built-in pipelines)
//   ├── post/       ← post_process chain
//   └── live/       ← Blender Live Link mini-PBR
//
// Cada source `.vert`/`.frag` listado en `SHADER_ALIASES` produce un único
// `.spv` en `shaders/<alias>.spv` (la ruta canónica que esperan los
// `include_bytes!` del runtime). El resto de shaders (si los hubiera) se
// compilan en el mismo directorio que la fuente.
//
// Los archivos `.glsl` dentro de `shaders/lib/` se exponen al preprocesador
// vía `-I shaders/lib`, permitiendo `#include "pbr.glsl"` con la extensión
// GL_GOOGLE_include_directive.
// =============================================================================

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Filename → ruta canónica de output (relativa al workspace).
fn shader_aliases() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    // ── Core (builtin pipelines) ─────────────────────────────────────────
    m.insert("shader.vert", "shaders/vert.spv");
    m.insert("shader.frag", "shaders/frag.spv");
    m.insert("texture.vert", "shaders/texture_vert.spv");
    m.insert("texture.frag", "shaders/texture_frag.spv");
    // ── Post-process chain ───────────────────────────────────────────────
    m.insert("post_process.vert", "shaders/post_process_vert.spv");
    m.insert("post_process.frag", "shaders/post_process_frag.spv");
    // ── Blender Live Link ────────────────────────────────────────────────
    m.insert("blender_live.vert", "shaders/blender_live_vert.spv");
    m.insert("blender_live.frag", "shaders/blender_live_frag.spv");
    m.insert("shadow.vert", "shaders/shadow_vert.spv");
    m.insert("shadow.frag", "shaders/shadow_frag.spv");
    m
}

fn compile_shader(src: &str, dst: &str) {
    let src_path = Path::new(src);
    let dst_path = Path::new(dst);

    println!("cargo:rerun-if-changed={}", src);

    // Skip if output is newer than source.
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
        .args([
            "-I",
            "shaders/lib", // permite `#include "pbr.glsl"` desde cualquier shader
            "-O",          // optimización por defecto
            src,
            "-o",
            dst,
        ])
        .status();

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => {
            eprintln!("glslc failed for {} with exit code: {:?}", src, s.code());
            // Don't fail the build — shaders may already exist as .spv
        }
        Err(e) => {
            eprintln!("Could not run glslc (is Vulkan SDK installed?): {}", e);
        }
    }
}

fn main() {
    let shaders_dir = Path::new("shaders");
    if !shaders_dir.exists() {
        eprintln!("cargo:warning=Shaders directory not found: shaders/");
        return;
    }

    // Re-run cuando cambie cualquier helper de la lib.
    println!("cargo:rerun-if-changed=shaders/lib");

    let aliases = shader_aliases();
    let mut compiled = 0;

    for entry in walkdir::WalkDir::new(shaders_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Procesar .vert / .frag / .comp.
        let Some(ext) = path.extension() else {
            continue;
        };
        let ext_str = ext.to_string_lossy();
        if ext_str != "vert" && ext_str != "frag" && ext_str != "comp" {
            continue;
        }

        let src = path.to_string_lossy().replace('\\', "/");
        let filename = path.file_name().unwrap().to_string_lossy().to_string();

        // Si el filename tiene un alias canónico, compila SOLO ahí (sin duplicar
        // un .spv junto al fuente).
        if let Some(&alias_dst) = aliases.get(filename.as_str()) {
            compile_shader(&src, alias_dst);
            compiled += 1;
        } else {
            // Shader sin alias → output en el mismo directorio que la fuente.
            let dst = src.replace(&format!(".{}", ext_str), ".spv");
            compile_shader(&src, &dst);
            compiled += 1;
        }
    }

    println!("cargo:warning=Compiled {} shaders", compiled);
}
