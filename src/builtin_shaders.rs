// =============================================================================
// shaders_builtin.rs — SPIR-V embebidos para uso plug-and-play
// =============================================================================
// Permite crear materiales sin necesidad de cargar shaders desde disco.
//
// Uso:
//   let vert = reactor::builtin_shaders::vert_default();
//   let frag = reactor::builtin_shaders::frag_default();
//   let mat = ctx.create_material(vert, frag)?;
//
// O directamente:
//   let mat = ctx.default_material()?;
// =============================================================================

/// SPIR-V del vertex shader por defecto (Position + Normal + UV → fragment).
pub fn vert_default() -> Vec<u32> {
    read_spv(include_bytes!("../shaders/vert.spv"))
}

/// SPIR-V del fragment shader por defecto (color vertex con luz simple).
pub fn frag_default() -> Vec<u32> {
    read_spv(include_bytes!("../shaders/frag.spv"))
}

/// SPIR-V del vertex shader con textura.
pub fn vert_textured() -> Vec<u32> {
    read_spv(include_bytes!("../shaders/texture_vert.spv"))
}

/// SPIR-V del fragment shader con textura difusa.
pub fn frag_textured() -> Vec<u32> {
    read_spv(include_bytes!("../shaders/texture_frag.spv"))
}

fn read_spv(bytes: &[u8]) -> Vec<u32> {
    ash::util::read_spv(&mut std::io::Cursor::new(bytes))
        .expect("Embedded SPIR-V is invalid (this is a REACTOR bug)")
}
