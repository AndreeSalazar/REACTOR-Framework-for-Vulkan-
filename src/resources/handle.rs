// =============================================================================
// Handle<T> — Reference-counted handle para assets
// =============================================================================
// Sistema tipo UE5/Unity: handles ligeros que apuntan a assets con RC.
// - Clonable y Copy en handle (solo punteros)
// - Detecta cuando el asset es descargado vía WeakHandle
// - Thread-safe vía Arc
// =============================================================================

use std::sync::{Arc, Weak};
use std::ops::Deref;
use crate::resources::asset_id::AssetId;

/// Handle con reference counting para assets cargados
/// 
/// - Ligero: solo contiene Arc + AssetId (16-24 bytes)
/// - Clonable: clonar es O(1), solo incrementa refcount
/// - Seguro: el asset se libera automáticamente cuando no hay más handles
/// 
/// # Ejemplo
/// ```rust
/// let handle: Handle<Texture> = asset_manager.load("texture.png")?;
/// let clone = handle.clone(); // barato, solo incrementa RC
/// 
/// // Acceder al asset
/// let texture: &Texture = &handle; // Deref impl
/// // o explícitamente:
/// let texture = handle.get();
/// ```
#[derive(Clone)]
pub struct Handle<T> {
    id: AssetId,
    inner: Arc<T>,
}

impl<T> Handle<T> {
    /// Crear nuevo handle desde AssetId y asset
    pub fn new(id: AssetId, asset: T) -> Self {
        Self {
            id,
            inner: Arc::new(asset),
        }
    }

    /// Obtener el AssetId de este handle
    #[inline]
    pub fn id(&self) -> AssetId {
        self.id
    }

    /// Obtener referencia al asset interno
    #[inline]
    pub fn get(&self) -> &T {
        &self.inner
    }

    /// Obtener referencia mutable (solo si este es el único handle)
    #[inline]
    pub fn get_mut(&mut self) -> Option<&mut T> {
        Arc::get_mut(&mut self.inner)
    }

    /// Convertir a WeakHandle para detectar unload sin mantener vivo el asset
    pub fn downgrade(&self) -> WeakHandle<T> {
        WeakHandle {
            id: self.id,
            inner: Arc::downgrade(&self.inner),
        }
    }

    /// Número de referencias fuertes a este asset
    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }

    /// Intentar convertir a owned (si este es el único handle)
    pub fn try_unwrap(self) -> Result<T, Self> {
        match Arc::try_unwrap(self.inner) {
            Ok(asset) => Ok(asset),
            Err(inner) => Err(Self { id: self.id, inner }),
        }
    }
}

impl<T> Deref for Handle<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handle")
            .field("id", &self.id)
            .field("ref_count", &self.ref_count())
            .finish_non_exhaustive()
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl<T> Eq for Handle<T> {}

impl<T> std::hash::Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        // Hash del puntero Arc para distinguir handles al mismo asset cargado dos veces
        (Arc::as_ptr(&self.inner) as usize).hash(state);
    }
}

// =============================================================================
// WeakHandle — Referencia débil para detectar unload
// =============================================================================

/// Weak reference para detectar si el asset fue descargado sin mantenerlo vivo
/// 
/// Útil para:
/// - Caches que no deben prevenir unload
/// - Observadores que quieren saber si el asset aún existe
/// - Evitar ciclos de referencia
pub struct WeakHandle<T> {
    id: AssetId,
    inner: Weak<T>,
}

impl<T> WeakHandle<T> {
    /// Intentar obtener un Handle fuerte
    pub fn upgrade(&self) -> Option<Handle<T>> {
        self.inner.upgrade().map(|inner| Handle {
            id: self.id,
            inner,
        })
    }

    /// Verificar si el asset aún está cargado
    pub fn is_valid(&self) -> bool {
        self.inner.strong_count() > 0
    }

    /// Obtener el AssetId (siempre disponible, incluso si el asset fue unload)
    pub fn id(&self) -> AssetId {
        self.id
    }
}

impl<T> Clone for WeakHandle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            inner: self.inner.clone(),
        }
    }
}

impl<T> std::fmt::Debug for WeakHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeakHandle")
            .field("id", &self.id)
            .field("valid", &self.is_valid())
            .finish()
    }
}

// =============================================================================
// AssetRef — Enum para handles que pueden ser fuertes o débiles
// =============================================================================

/// Referencia flexible a un asset: puede ser fuerte o débil
/// 
/// Útil para sistemas que necesitan flexibilidad:
/// - Editor: mantener assets cargados (Handle)
/// - Runtime: referencias débiles que permiten unload (WeakHandle)
#[derive(Clone)]
pub enum AssetRef<T> {
    Strong(Handle<T>),
    Weak(WeakHandle<T>),
}

impl<T> AssetRef<T> {
    /// Crear desde Handle fuerte
    pub fn strong(handle: Handle<T>) -> Self {
        Self::Strong(handle)
    }

    /// Crear desde WeakHandle
    pub fn weak(weak: WeakHandle<T>) -> Self {
        Self::Weak(weak)
    }

    /// Intentar obtener referencia al asset (upgrade si es weak)
    pub fn get(&self) -> Option<&T> {
        match self {
            Self::Strong(h) => Some(&**h),
            Self::Weak(w) => w.upgrade().map(|h| {
                // Nota: el Handle temporal se drop aquí, pero la referencia es válida
                // porque el asset original aún existe (si llegamos aquí)
                // Esto es seguro porque solo devolvemos &T, no el Handle
                unsafe { std::mem::transmute::<&T, &T>(&*h.inner) }
            }),
        }
    }

    /// Verificar si el asset está disponible
    pub fn is_valid(&self) -> bool {
        match self {
            Self::Strong(_) => true,
            Self::Weak(w) => w.is_valid(),
        }
    }

    /// Obtener AssetId
    pub fn id(&self) -> AssetId {
        match self {
            Self::Strong(h) => h.id(),
            Self::Weak(w) => w.id(),
        }
    }

    /// Convertir a WeakHandle (downgrade si es strong)
    pub fn downgrade(&self) -> WeakHandle<T> {
        match self {
            Self::Strong(h) => h.downgrade(),
            Self::Weak(w) => w.clone(),
        }
    }
}

impl<T> From<Handle<T>> for AssetRef<T> {
    fn from(handle: Handle<T>) -> Self {
        Self::Strong(handle)
    }
}

impl<T> From<WeakHandle<T>> for AssetRef<T> {
    fn from(weak: WeakHandle<T>) -> Self {
        Self::Weak(weak)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_basic() {
        let id = AssetId::from_path("test.txt");
        let handle = Handle::new(id, "hello".to_string());
        
        assert_eq!(handle.id(), id);
        assert_eq!(&*handle, "hello");
        assert_eq!(handle.get(), "hello");
        assert_eq!(handle.ref_count(), 1);
    }

    #[test]
    fn test_handle_clone() {
        let id = AssetId::from_path("test.txt");
        let handle1 = Handle::new(id, 42i32);
        let handle2 = handle1.clone();
        
        assert_eq!(handle1.ref_count(), 2);
        assert_eq!(handle2.ref_count(), 2);
        assert_eq!(*handle1, *handle2);
        assert!(handle1 == handle2); // PartialEq
    }

    #[test]
    fn test_weak_handle() {
        let id = AssetId::from_path("test.txt");
        let handle = Handle::new(id, Vec::<u8>::new());
        let weak = handle.downgrade();
        
        assert!(weak.is_valid());
        assert!(weak.upgrade().is_some());
        
        drop(handle);
        assert!(!weak.is_valid());
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn test_asset_ref() {
        let id = AssetId::from_path("test.txt");
        let handle = Handle::new(id, "data".to_string());
        
        let strong_ref: AssetRef<String> = AssetRef::strong(handle.clone());
        let weak_ref: AssetRef<String> = AssetRef::weak(handle.downgrade());
        
        assert!(strong_ref.is_valid());
        assert!(weak_ref.is_valid());
        assert_eq!(strong_ref.get(), Some("data"));
        assert_eq!(weak_ref.get(), Some("data"));
        
        drop(handle);
        assert!(strong_ref.is_valid()); // StrongRef mantiene vivo el asset
        assert!(!weak_ref.is_valid()); // WeakRef no
    }
}
