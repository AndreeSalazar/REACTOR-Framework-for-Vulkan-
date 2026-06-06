// =============================================================================
// XENOFALL — World / Scenario Construction
// =============================================================================
// Static and semi-static scene geometry lives here so the game loop can focus on
// gameplay. This is also the future home for sectors, portals, static caches,
// decals attached to the world, and delta-rendering visibility metadata.
// =============================================================================

use crate::xenofall::constants::{CORRIDOR_HALF_WIDTH, CORRIDOR_HEIGHT, PILLAR_X};
use reactor_vulkan::prelude::*;

#[derive(Default)]
pub struct WorldGeometry {
    pub floor_indices: Vec<usize>,
    pub puddle_indices: Vec<usize>,
    pub wall_indices: Vec<usize>,
    pub pillar_indices: Vec<usize>,
}

impl WorldGeometry {
    pub fn floor_count(&self) -> usize {
        self.floor_indices.len()
    }

    pub fn puddle_count(&self) -> usize {
        self.puddle_indices.len()
    }
}

pub fn build_corridor(ctx: &mut ReactorContext) -> WorldGeometry {
    let mut world = WorldGeometry::default();
    let w = CORRIDOR_HALF_WIDTH; // 3.5m from center
    let h = CORRIDOR_HEIGHT; // 3.5m
    let pw = PILLAR_X; // 2.8m from center

    // ── Suelo del corredor (7m wide) ──
    for i in 0..12 {
        let z = -(i as f32 * 8.0 + 4.0);
        if let Ok(idx) = ctx.spawn_plane(Vec3::new(0.0, 0.0, z), w * 2.0) {
            world.floor_indices.push(idx);
        }
    }

    build_water_puddles(ctx, &mut world);

    // ── Paredes laterales (3.5m tall, matching corridor height) ──
    for i in 0..20 {
        let z = -(i as f32 * 5.0 + 2.5);
        if let Ok(left) = ctx.spawn_cube(Vec3::ZERO) {
            ctx.set_transform(
                left,
                Mat4::from_scale_rotation_translation(
                    Vec3::new(0.25, h, 5.0),
                    Quat::IDENTITY,
                    Vec3::new(-w, h * 0.5, z),
                ),
            );
            world.wall_indices.push(left);
        }
        if let Ok(right) = ctx.spawn_cube(Vec3::ZERO) {
            ctx.set_transform(
                right,
                Mat4::from_scale_rotation_translation(
                    Vec3::new(0.25, h, 5.0),
                    Quat::IDENTITY,
                    Vec3::new(w, h * 0.5, z),
                ),
            );
            world.wall_indices.push(right);
        }
    }

    // ── Pilares decorativos (human-scale, ~0.4m × 3.2m × 0.4m) ──
    for i in 0..10 {
        let z = -(i as f32 * 10.0 + 7.0);
        if let Ok(idx) = ctx.spawn_cube(Vec3::ZERO) {
            ctx.set_transform(
                idx,
                Mat4::from_scale_rotation_translation(
                    Vec3::new(0.4, h - 0.3, 0.4),
                    Quat::IDENTITY,
                    Vec3::new(-pw, (h - 0.3) * 0.5, z),
                ),
            );
            world.pillar_indices.push(idx);
        }
        if let Ok(idx) = ctx.spawn_cube(Vec3::ZERO) {
            ctx.set_transform(
                idx,
                Mat4::from_scale_rotation_translation(
                    Vec3::new(0.4, h - 0.3, 0.4),
                    Quat::IDENTITY,
                    Vec3::new(pw, (h - 0.3) * 0.5, z),
                ),
            );
            world.pillar_indices.push(idx);
        }
    }

    // ── Techo (barras transversales spanning corridor width) ──
    for i in 0..10 {
        let z = -(i as f32 * 10.0 + 5.0);
        if let Ok(idx) = ctx.spawn_cube(Vec3::ZERO) {
            ctx.set_transform(
                idx,
                Mat4::from_scale_rotation_translation(
                    Vec3::new(w * 2.0, 0.15, 0.4),
                    Quat::IDENTITY,
                    Vec3::new(0.0, h, z),
                ),
            );
            world.wall_indices.push(idx);
        }
    }

    world
}

fn build_water_puddles(ctx: &mut ReactorContext, world: &mut WorldGeometry) {
    // Fase 2: charcos pequeños, cacheables y baratos. Sirven para probar
    // SSR/TAA y materiales reflectivos antes de implementar agua reactiva.
    const PUDDLES: &[(f32, f32, f32)] = &[
        (-1.15, -9.5, 1.25),
        (1.05, -21.0, 0.95),
        (-0.55, -34.0, 1.55),
        (1.45, -52.0, 1.10),
        (-1.35, -68.0, 1.35),
        (0.35, -82.5, 0.85),
    ];

    for &(x, z, size) in PUDDLES {
        if let Ok(idx) = ctx.spawn_plane(Vec3::new(x, 0.012, z), size) {
            world.puddle_indices.push(idx);
        }
    }
}

pub fn apply_world_materials(ctx: &mut ReactorContext, world: &WorldGeometry) {
    for &idx in &world.floor_indices {
        if let Some(obj) = ctx.scene.objects.get_mut(idx) {
            obj.color = Vec4::new(0.08, 0.08, 0.09, 1.0); // Concreto oscuro mojado
            obj.metallic = 0.0;
            obj.roughness = 0.12; // muy liso = reflejos nítidos
        }
    }

    for &idx in &world.puddle_indices {
        if let Some(obj) = ctx.scene.objects.get_mut(idx) {
            obj.color = Vec4::new(0.035, 0.055, 0.065, 0.78); // agua sucia fría
            obj.metallic = 0.0;
            obj.roughness = 0.025; // espejo imperfecto para SSR/TAA
        }
    }

    for &idx in &world.wall_indices {
        if let Some(obj) = ctx.scene.objects.get_mut(idx) {
            obj.color = Vec4::new(0.14, 0.13, 0.12, 1.0); // Concreto sucio mate
            obj.metallic = 0.05;
            obj.roughness = 0.85; // rugoso = sin reflejos
        }
    }

    for &idx in &world.pillar_indices {
        if let Some(obj) = ctx.scene.objects.get_mut(idx) {
            obj.color = Vec4::new(0.22, 0.18, 0.14, 1.0); // Metal oxidado
            obj.metallic = 0.65;
            obj.roughness = 0.45;
        }
    }
}
