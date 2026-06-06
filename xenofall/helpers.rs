// Extracted from xenofall.rs — Helpers
// =============================================================================
// HELPERS — Pure utility functions
// =============================================================================

use crate::xenofall::cards::CardType;
use reactor_vulkan::prelude::*;

pub fn hash_rand(seed: u32) -> f32 {
    let mut x = seed.wrapping_mul(2654435761);
    x ^= x >> 16;
    x = x.wrapping_mul(0x85ebca6b);
    x ^= x >> 13;
    x = x.wrapping_mul(0xc2b2ae35);
    x ^= x >> 16;
    (x % 10000) as f32 / 10000.0
}

pub fn hash_rand_signed(seed: u32) -> f32 {
    hash_rand(seed) * 2.0 - 1.0
}

/// Pick N unique random cards from all cards
pub fn pick_random_cards(seed: u32, count: usize) -> Vec<CardType> {
    let all = CardType::all();
    let mut indices: Vec<usize> = (0..all.len()).collect();
    let mut result = Vec::new();

    for i in 0..count.min(all.len()) {
        let remaining = indices.len();
        if remaining == 0 {
            break;
        }
        let pick = (hash_rand(seed.wrapping_add(i as u32 * 37)) * remaining as f32) as usize;
        let pick = pick.min(remaining - 1);
        result.push(all[indices[pick]]);
        indices.swap_remove(pick);
    }
    result
}

pub fn ray_sphere_intersect(origin: Vec3, dir: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    let oc = origin - center;
    let a = dir.dot(dir);
    let b = 2.0 * oc.dot(dir);
    let c = oc.dot(oc) - radius * radius;
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        None
    } else {
        let t = (-b - disc.sqrt()) / (2.0 * a);
        if t > 0.0 {
            Some(t)
        } else {
            None
        }
    }
}

pub fn ray_headshot_intersect(origin: Vec3, dir: Vec3, center: Vec3) -> Option<f32> {
    let head_center = center + Vec3::new(0.0, 0.5, 0.0);
    ray_sphere_intersect(origin, dir, head_center, 0.3)
}
