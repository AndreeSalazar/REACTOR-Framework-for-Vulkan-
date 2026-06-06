// =============================================================================
// XENOFALL — Gameplay VFX Pools
// =============================================================================
// Projectile tracers, impact flashes, and muzzle flash are dynamic VFX. Keeping
// their pools separate from gameplay state makes them ready for dirty-tile
// marking and future GPU particle paths.
// =============================================================================

use crate::xenofall::constants::{IMPACT_POOL_SIZE, TRACER_POOL_SIZE};
use reactor_vulkan::prelude::*;

#[derive(Default)]
pub struct VfxPools {
    pub tracer_pool: Vec<usize>,
    pub impact_pool: Vec<usize>,
    pub muzzle_flash_index: Option<usize>,
}

impl VfxPools {
    pub fn tracer_count(&self) -> usize {
        self.tracer_pool.len()
    }

    pub fn impact_count(&self) -> usize {
        self.impact_pool.len()
    }
}

pub fn build_pools(ctx: &mut ReactorContext) -> VfxPools {
    let mut pools = VfxPools::default();

    for _ in 0..TRACER_POOL_SIZE {
        if let Ok(idx) = ctx.spawn_sphere(Vec3::new(0.0, -1000.0, 0.0), 0.04) {
            pools.tracer_pool.push(idx);
        }
    }

    for _ in 0..IMPACT_POOL_SIZE {
        if let Ok(idx) = ctx.spawn_sphere(Vec3::new(0.0, -1000.0, 0.0), 0.12) {
            pools.impact_pool.push(idx);
        }
    }

    if let Ok(idx) = ctx.spawn_sphere(Vec3::new(0.0, -1000.0, 0.0), 0.15) {
        pools.muzzle_flash_index = Some(idx);
    }

    pools
}

pub fn apply_vfx_materials(ctx: &mut ReactorContext, pools: &VfxPools) {
    for (i, &idx) in pools.tracer_pool.iter().enumerate() {
        if let Some(obj) = ctx.scene.objects.get_mut(idx) {
            obj.color = Vec4::new(1.0, 0.78, 0.25, 1.0);
            obj.metallic = 0.0;
            obj.roughness = if i % 2 == 0 { 0.08 } else { 0.16 };
        }
    }

    for &idx in &pools.impact_pool {
        if let Some(obj) = ctx.scene.objects.get_mut(idx) {
            obj.color = Vec4::new(1.0, 0.22, 0.08, 1.0);
            obj.metallic = 0.0;
            obj.roughness = 0.12;
        }
    }

    if let Some(idx) = pools.muzzle_flash_index {
        if let Some(obj) = ctx.scene.objects.get_mut(idx) {
            obj.color = Vec4::new(1.0, 0.62, 0.18, 1.0);
            obj.metallic = 0.0;
            obj.roughness = 0.04;
        }
    }
}
