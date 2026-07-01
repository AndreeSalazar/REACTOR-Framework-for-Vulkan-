#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BaseShaderStage {
    Vertex,
    Fragment,
    Compute,
}

impl BaseShaderStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vertex => "vertex",
            Self::Fragment => "fragment",
            Self::Compute => "compute",
        }
    }
}
