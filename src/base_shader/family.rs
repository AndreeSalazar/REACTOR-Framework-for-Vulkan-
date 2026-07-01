#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BaseShaderFamily {
    CoreForward,
    CoreTextured,
    BlenderLivePbr,
    ShadowDepth,
    Deferred,
    PostFullscreen,
    PostCompute,
    IblBake,
}

impl BaseShaderFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CoreForward => "core.forward",
            Self::CoreTextured => "core.textured",
            Self::BlenderLivePbr => "live.pbr",
            Self::ShadowDepth => "shadow.depth",
            Self::Deferred => "deferred.gbuffer",
            Self::PostFullscreen => "post.fullscreen",
            Self::PostCompute => "post.compute",
            Self::IblBake => "ibl.bake",
        }
    }
}
