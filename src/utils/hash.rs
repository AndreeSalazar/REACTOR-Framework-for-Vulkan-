//! Hashing utilities
//! 
//! Fast hashing functions for resource IDs and asset lookups.

use std::hash::{Hash, Hasher};

/// Fast hash for u64 values using FNV-1a algorithm
#[inline]
pub fn hash_u64(value: u64) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    
    let mut hash = FNV_OFFSET;
    hash ^= value;
    hash = hash.wrapping_mul(FNV_PRIME);
    hash
}

/// Hash a string using FNV-1a
#[inline]
pub fn hash_str(s: &str) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    
    let mut hash = FNV_OFFSET;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Hash any type that implements Hash
#[inline]
pub fn hash<T: Hash>(value: &T) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}
