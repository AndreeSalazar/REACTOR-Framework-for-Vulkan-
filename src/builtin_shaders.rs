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
    crate::base_shader::BaseShaderAsset::CoreVert.words()
}

/// SPIR-V del fragment shader por defecto (color vertex con luz simple).
pub fn frag_default() -> Vec<u32> {
    crate::base_shader::BaseShaderAsset::CoreFrag.words()
}

/// SPIR-V del vertex shader con textura.
pub fn vert_textured() -> Vec<u32> {
    crate::base_shader::BaseShaderAsset::TextureVert.words()
}

/// SPIR-V del fragment shader con textura difusa.
pub fn frag_textured() -> Vec<u32> {
    crate::base_shader::BaseShaderAsset::TextureFrag.words()
}
