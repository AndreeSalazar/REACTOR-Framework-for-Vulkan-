// =============================================================================
// FontAsset — Representación de fuentes TTF/OTF
// =============================================================================

use std::path::Path;
use crate::core::error::{ReactorResult, ReactorError};

#[derive(Clone, Debug)]
pub struct FontAsset {
    /// Datos binarios de la fuente TTF/OTF
    pub font_data: Vec<u8>,
    /// Nombre de la familia de fuente o archivo
    pub family_name: String,
}

impl FontAsset {
    /// Cargar fuente desde un archivo TTF/OTF
    pub fn from_file<P: AsRef<Path>>(path: P) -> ReactorResult<Self> {
        let path_ref = path.as_ref();
        let font_data = std::fs::read(path_ref)
            .map_err(|e| ReactorError::internal(format!("Failed to read font {}: {}", path_ref.display(), e)))?;
        
        let family_name = path_ref.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed_font")
            .to_string();
            
        Ok(Self {
            font_data,
            family_name,
        })
    }

    /// Cargar fuente desde bytes en memoria
    pub fn from_bytes(bytes: &[u8], name: &str) -> Self {
        Self {
            font_data: bytes.to_vec(),
            family_name: name.to_string(),
        }
    }
}
