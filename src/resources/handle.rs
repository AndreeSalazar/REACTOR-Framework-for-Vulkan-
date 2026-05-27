use std::ops::Deref;
use std::sync::{Arc, Weak};

use crate::resources::asset_id::AssetId;

/// Reference-counted handle for loaded assets.
pub struct Handle<T> {
    id: AssetId,
    inner: Arc<T>,
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self { id: self.id, inner: self.inner.clone() }
    }
}

impl<T> Handle<T> {
    /// Create a new handle from an asset id and owned asset data.
    pub fn new(id: AssetId, asset: T) -> Self {
        Self { id, inner: Arc::new(asset) }
    }

    /// Clone the internal `Arc<T>` when shared ownership is needed.
    pub fn arc(&self) -> Arc<T> {
        self.inner.clone()
    }

    #[inline]
    pub fn id(&self) -> AssetId {
        self.id
    }

    #[inline]
    pub fn get(&self) -> &T {
        &self.inner
    }

    #[inline]
    pub fn get_mut(&mut self) -> Option<&mut T> {
        Arc::get_mut(&mut self.inner)
    }

    pub fn downgrade(&self) -> WeakHandle<T> {
        WeakHandle {
            id: self.id,
            inner: Arc::downgrade(&self.inner),
        }
    }

    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }

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
        (Arc::as_ptr(&self.inner) as usize).hash(state);
    }
}

/// Weak asset reference that can detect unload without keeping data alive.
pub struct WeakHandle<T> {
    id: AssetId,
    inner: Weak<T>,
}

impl<T> WeakHandle<T> {
    pub fn upgrade(&self) -> Option<Handle<T>> {
        self.inner
            .upgrade()
            .map(|inner| Handle { id: self.id, inner })
    }

    pub fn is_valid(&self) -> bool {
        self.inner.strong_count() > 0
    }

    pub fn id(&self) -> AssetId {
        self.id
    }
}

impl<T> Clone for WeakHandle<T> {
    fn clone(&self) -> Self {
        Self { id: self.id, inner: self.inner.clone() }
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

/// Flexible asset reference. `Strong` keeps data alive; `Weak` allows unload.
pub enum AssetRef<T> {
    Strong(Handle<T>),
    Weak(WeakHandle<T>),
}

impl<T> Clone for AssetRef<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Strong(h) => Self::Strong(h.clone()),
            Self::Weak(w) => Self::Weak(w.clone()),
        }
    }
}

impl<T> AssetRef<T> {
    pub fn strong(handle: Handle<T>) -> Self {
        Self::Strong(handle)
    }

    pub fn weak(weak: WeakHandle<T>) -> Self {
        Self::Weak(weak)
    }

    /// Borrow the asset only when this reference is already strong.
    pub fn get(&self) -> Option<&T> {
        match self {
            Self::Strong(h) => Some(&**h),
            Self::Weak(_) => None,
        }
    }

    /// Upgrade to a strong handle if the asset is still loaded.
    pub fn upgrade(&self) -> Option<Handle<T>> {
        match self {
            Self::Strong(h) => Some(h.clone()),
            Self::Weak(w) => w.upgrade(),
        }
    }

    pub fn is_valid(&self) -> bool {
        match self {
            Self::Strong(_) => true,
            Self::Weak(w) => w.is_valid(),
        }
    }

    pub fn id(&self) -> AssetId {
        match self {
            Self::Strong(h) => h.id(),
            Self::Weak(w) => w.id(),
        }
    }

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
        assert!(handle1 == handle2);
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
        assert_eq!(strong_ref.get().map(String::as_str), Some("data"));

        let upgraded = weak_ref.upgrade().expect("weak ref should upgrade");
        assert_eq!(upgraded.get(), "data");
        drop(upgraded);

        drop(handle);
        assert!(strong_ref.is_valid());
        assert!(weak_ref.is_valid());
        drop(strong_ref);
        assert!(!weak_ref.is_valid());
    }
}
