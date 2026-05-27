// =============================================================================
// AssetId — Identificador único y estable para assets
// =============================================================================
// Usa hash XXH3 del path normalizado + contenido para garantizar estabilidad
// incluso si el archivo se mueve o modifica.
// =============================================================================

use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use xxhash_rust::xxh3::xxh3_64;

/// Identificador único y estable para assets (hash del path + contenido)
///
/// - Determinista: mismo archivo → mismo AssetId siempre
/// - Robusto: cambios en el contenido cambian el ID (hot-reload)
/// - Eficiente: u64, copiable, comparable en O(1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId(u64);

impl AssetId {
    /// Valor nulo/inválido
    pub const INVALID: Self = Self(0);

    /// Genera AssetId desde path de archivo (hash del path normalizado)
    /// Útil para referencias rápidas cuando no necesitas detectar cambios
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();

        // Normalizar: lowercase + reemplazar separators para consistencia cross-platform
        let normalized = path.to_string_lossy().to_lowercase().replace('\\', "/");

        Self(xxh3_64(normalized.as_bytes()))
    }

    /// Genera AssetId desde path + contenido (detecta cambios para hot-reload)
    /// Más robusto pero requiere leer el archivo
    pub fn from_path_with_content<P: AsRef<Path>>(path: P, content: &[u8]) -> Self {
        let path_hash = Self::from_path(path).0;
        let content_hash = xxh3_64(content);
        // Combinar ambos hashes con XOR para mantener distribución uniforme
        Self(path_hash ^ content_hash)
    }

    /// Genera AssetId desde string arbitrario (para assets procedurales)
    pub fn from_key(s: &str) -> Self {
        Self(xxh3_64(s.as_bytes()))
    }

    /// Genera AssetId combinando múltiples componentes (para assets compuestos)
    pub fn from_components(components: &[&str]) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for comp in components {
            comp.hash(&mut hasher);
        }
        Self(hasher.finish())
    }

    #[inline]
    pub fn as_u64(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        self.0 != 0
    }
}

impl From<u64> for AssetId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<AssetId> for u64 {
    fn from(id: AssetId) -> u64 {
        id.0
    }
}

impl std::fmt::Display for AssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

// =============================================================================
// AssetPath — Wrapper para paths de assets con normalización
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetPath(PathBuf);

impl AssetPath {
    /// Crear desde string, normalizando separators
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let normalized = path
            .as_ref()
            .to_string_lossy()
            .replace('\\', "/")
            .to_lowercase();
        Self(PathBuf::from(normalized))
    }

    /// Obtener AssetId de este path
    pub fn to_id(&self) -> AssetId {
        AssetId::from_path(&self.0)
    }

    /// Obtener referencia al PathBuf interno
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// Convertir a PathBuf owned
    pub fn into_path(self) -> PathBuf {
        self.0
    }
}

// Implementaciones concretas (un blanket `From<P: AsRef<Path>>` chocaría con
// la implementación reflexiva `From<T> for T` que ya provee la std).
impl From<&Path> for AssetPath {
    fn from(path: &Path) -> Self {
        Self::new(path)
    }
}

impl From<PathBuf> for AssetPath {
    fn from(path: PathBuf) -> Self {
        Self::new(path)
    }
}

impl From<&str> for AssetPath {
    fn from(path: &str) -> Self {
        Self::new(path)
    }
}

impl From<String> for AssetPath {
    fn from(path: String) -> Self {
        Self::new(path)
    }
}

impl AsRef<Path> for AssetPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl std::fmt::Display for AssetPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.display().fmt(f)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_id_from_path_consistency() {
        let id1 = AssetId::from_path("assets/models/zombie.glb");
        let id2 = AssetId::from_path("ASSETS/MODELS/ZOMBIE.GLB"); // uppercase
        let id3 = AssetId::from_path("assets\\models\\zombie.glb"); // windows separators

        assert_eq!(id1, id2, "Debe ser case-insensitive");
        assert_eq!(id1, id3, "Debe normalizar separators");
    }

    #[test]
    fn test_asset_id_content_sensitivity() {
        let content1 = b"hello world";
        let content2 = b"hello world!";

        let id1 = AssetId::from_path_with_content("test.txt", content1);
        let id2 = AssetId::from_path_with_content("test.txt", content2);

        assert_ne!(id1, id2, "Contenido diferente debe generar ID diferente");
    }

    #[test]
    fn test_asset_id_invalid() {
        assert!(!AssetId::INVALID.is_valid());
        assert!(AssetId::from_path("test.glb").is_valid());
    }
}
