use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

fn shader_aliases() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    // ── Core (builtin pipelines) ─────────────────────────────────────────
    m.insert("shader.vert", "shaders/vert.spv");
    m.insert("shader.frag", "shaders/frag.spv");
    m.insert("texture.vert", "shaders/texture_vert.spv");
    m.insert("texture.frag", "shaders/texture_frag.spv");
    // ── Deferred / G-Buffer ──────────────────────────────────────────────
    m.insert("gbuffer.vert", "shaders/deferred/gbuffer_vert.spv");
    m.insert("gbuffer.frag", "shaders/deferred/gbuffer_frag.spv");
    m.insert("lighting_resolve.frag", "shaders/deferred/lighting_resolve.spv");
    // ── Post-process chain ───────────────────────────────────────────────
    m.insert("post_process.vert", "shaders/post_process_vert.spv");
    m.insert("post_process.frag", "shaders/post_process_frag.spv");
    m.insert("decal.frag", "shaders/post/decal.spv");
    m.insert("auto_exposure.comp", "shaders/post/auto_exposure.spv");
    m.insert("bloom_downsample.comp", "shaders/post/bloom_downsample.spv");
    m.insert("bloom_upsample.comp", "shaders/post/bloom_upsample.spv");
    m.insert("depth_resolve.comp", "shaders/post/depth_resolve.spv");
    m.insert("gtao.comp", "shaders/post/gtao.spv");
    m.insert("hiz_build.comp", "shaders/post/hiz_build.spv");
    m.insert("lens_flare.comp", "shaders/post/lens_flare.spv");
    m.insert("ssgi_hiz.comp", "shaders/post/ssgi_hiz.spv");
    m.insert("taa_resolve.comp", "shaders/post/taa_resolve.spv");
    m.insert("volumetric_clouds.comp", "shaders/post/volumetric_clouds.spv");
    m.insert("volumetric_fog.comp", "shaders/post/volumetric_fog.spv");
    // ── Compute ──────────────────────────────────────────────────────────
    m.insert("cull.comp", "shaders/compute/cull.spv");
    m.insert("light_cull.comp", "shaders/compute/light_cull.spv");
    // ── IBL baking ───────────────────────────────────────────────────────
    m.insert("equirect_to_cube.comp", "shaders/ibl/equirect_to_cube.spv");
    m.insert("irradiance.comp", "shaders/ibl/irradiance.spv");
    m.insert("prefilter.comp", "shaders/ibl/prefilter.spv");
    m.insert("brdf_lut.comp", "shaders/ibl/brdf_lut.spv");
    // ── Particles ────────────────────────────────────────────────────────
    m.insert("particle.vert", "shaders/particles/particle_vert.spv");
    m.insert("particle.frag", "shaders/particles/particle_frag.spv");
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

    if dst_path.exists() {
        if let Ok(dst_meta) = dst_path.metadata() {
            if let Ok(dst_time) = dst_meta.modified() {
                if let Some(input_time) = newest_shader_input_time(src_path) {
                    if dst_time >= input_time {
                        return;
                    }
                } else {
                    return;
                }
            }
        }
    }

    println!("cargo:warning=Compiling shader: {} -> {}", src, dst);

    let status = Command::new("glslc")
        .args([
            "-I",
            "shaders/lib",
            "-O",
            src,
            "-o",
            dst,
        ])
        .status();

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => {
            eprintln!("glslc failed for {} with exit code: {:?}", src, s.code());
        }
        Err(e) => {
            eprintln!("Could not run glslc (is Vulkan SDK installed?): {}", e);
        }
    }
}

fn newest_shader_input_time(src_path: &Path) -> Option<SystemTime> {
    let mut newest = src_path.metadata().ok()?.modified().ok()?;
    let lib_dir = Path::new("shaders/lib");

    if let Ok(entries) = fs::read_dir(lib_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("glsl") {
                continue;
            }
            if let Ok(time) = entry.metadata().and_then(|meta| meta.modified()) {
                if time > newest {
                    newest = time;
                }
            }
        }
    }

    Some(newest)
}

fn main() {
    let shaders_dir = Path::new("shaders");
    if !shaders_dir.exists() {
        eprintln!("cargo:warning=Shaders directory not found: shaders/");
        return;
    }

    println!("cargo:rerun-if-changed=shaders");
    println!("cargo:rerun-if-changed=shaders/lib");

    let aliases = shader_aliases();
    let mut compiled = 0;

    for entry in walkdir::WalkDir::new(shaders_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        let Some(ext) = path.extension() else { continue };
        let ext_str = ext.to_string_lossy();
        if ext_str != "vert" && ext_str != "frag" && ext_str != "comp" {
            continue;
        }

        let src = path.to_string_lossy().replace('\\', "/");
        let filename = path.file_name().unwrap().to_string_lossy().to_string();

        if let Some(&alias_dst) = aliases.get(filename.as_str()) {
            compile_shader(&src, alias_dst);
            compiled += 1;
        } else {
            let dst = src.replace(&format!(".{}", ext_str), ".spv");
            compile_shader(&src, &dst);
            compiled += 1;
        }
    }

    println!("cargo:warning=Compiled {} shaders", compiled);
}
