macro_rules! define_handle {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
        #[repr(transparent)]
        pub struct $name(pub u32);

        impl $name {
            pub const INVALID: Self = Self(u32::MAX);

            #[inline(always)]
            pub fn index(&self) -> u32 {
                self.0
            }

            #[inline(always)]
            pub fn is_valid(&self) -> bool {
                self.0 != u32::MAX
            }

            #[inline(always)]
            pub fn from_index(i: u32) -> Self {
                Self(i)
            }
        }

        impl From<u32> for $name {
            #[inline(always)]
            fn from(v: u32) -> Self {
                Self(v)
            }
        }

        impl From<$name> for u32 {
            #[inline(always)]
            fn from(h: $name) -> Self {
                h.0
            }
        }
    };
}

define_handle!(TextureHandle, "Índice al array bindless de texturas (binding 0).");
define_handle!(SamplerHandle, "Índice al array bindless de samplers (binding 1).");
define_handle!(BufferHandle, "Índice al array bindless de buffers genéricos (binding 2).");
define_handle!(MeshHandle, "Índice al array bindless de MeshData (binding 3).");
define_handle!(MaterialHandle, "Índice al array bindless de MaterialData (binding 4).");
