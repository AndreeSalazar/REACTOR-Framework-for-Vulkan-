// =============================================================================
// XENOFALL — World / Scenario Construction
// =============================================================================
// Static and semi-static scene geometry lives here so the game loop can focus on
// gameplay. This is also the future home for sectors, portals, static caches,
// decals attached to the world, and delta-rendering visibility metadata.
// =============================================================================

use crate::xenofall::constants::{CORRIDOR_HALF_WIDTH, CORRIDOR_HEIGHT, PILLAR_X};
use reactor_vulkan::prelude::*;

const BACKROOM_CELL: f32 = 7.0;
const BACKROOM_HALF_CELL: f32 = BACKROOM_CELL * 0.5;
const BACKROOM_ROOM_COUNT: i32 = 15;
const BACKROOM_LANE_COUNT: i32 = 3;
const BACKROOM_WALL_THICKNESS: f32 = 0.18;
const BACKROOM_PARTITION_WIDTH: f32 = 0.16;

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
    build_backrooms_test_scene(ctx)
}

pub fn build_backrooms_test_scene(ctx: &mut ReactorContext) -> WorldGeometry {
    let mut world = WorldGeometry::default();
    let w = CORRIDOR_HALF_WIDTH; // 3.5m from center
    let h = CORRIDOR_HEIGHT; // 3.5m
    let pw = PILLAR_X; // 2.8m from center

    // ── Backrooms floor/carpet grid ──
    for room in 0..BACKROOM_ROOM_COUNT {
        let z = -(room as f32 * BACKROOM_CELL + BACKROOM_HALF_CELL);
        for lane in 0..BACKROOM_LANE_COUNT {
            let lane_offset = lane - BACKROOM_LANE_COUNT / 2;
            let x = lane_offset as f32 * BACKROOM_CELL;
            if let Ok(idx) = ctx.spawn_plane(Vec3::new(x, 0.0, z), BACKROOM_CELL) {
                world.floor_indices.push(idx);
            }
        }
    }

    build_water_puddles(ctx, &mut world);

    // ── Long exterior walls: liminal tunnel bounds ──
    let full_depth = BACKROOM_ROOM_COUNT as f32 * BACKROOM_CELL;
    let center_z = -full_depth * 0.5;
    let total_half_width = BACKROOM_LANE_COUNT as f32 * BACKROOM_CELL * 0.5;
    for &x in &[-total_half_width, total_half_width] {
        if let Ok(idx) = ctx.spawn_cube(Vec3::ZERO) {
            ctx.set_transform(
                idx,
                Mat4::from_scale_rotation_translation(
                    Vec3::new(BACKROOM_WALL_THICKNESS, h, full_depth),
                    Quat::IDENTITY,
                    Vec3::new(x, h * 0.5, center_z),
                ),
            );
            world.wall_indices.push(idx);
        }
    }

    // ── Interior partial walls/partitions with staggered openings ──
    for room in 0..BACKROOM_ROOM_COUNT {
        let z = -(room as f32 * BACKROOM_CELL + BACKROOM_HALF_CELL);
        for lane_edge in 1..BACKROOM_LANE_COUNT {
            let x = -total_half_width + lane_edge as f32 * BACKROOM_CELL;
            let gap_shift = if room % 2 == 0 { -1.6 } else { 1.6 };
            for &segment_z in &[z - 1.9 + gap_shift, z + 1.9 + gap_shift] {
                if let Ok(idx) = ctx.spawn_cube(Vec3::ZERO) {
                    ctx.set_transform(
                        idx,
                        Mat4::from_scale_rotation_translation(
                            Vec3::new(BACKROOM_PARTITION_WIDTH, h, 2.2),
                            Quat::IDENTITY,
                            Vec3::new(x, h * 0.5, segment_z),
                        ),
                    );
                    world.wall_indices.push(idx);
                }
            }
        }
    }

    // ── Cross walls create the backrooms maze rhythm but leave the rail path open ──
    for room in 1..BACKROOM_ROOM_COUNT {
        let z = -(room as f32 * BACKROOM_CELL);
        for lane in 0..BACKROOM_LANE_COUNT {
            let lane_offset = lane - BACKROOM_LANE_COUNT / 2;
            let x = lane_offset as f32 * BACKROOM_CELL;
            let keep_rail_open = lane_offset == 0 && room % 3 != 0;
            if keep_rail_open {
                continue;
            }
            if let Ok(idx) = ctx.spawn_cube(Vec3::ZERO) {
                ctx.set_transform(
                    idx,
                    Mat4::from_scale_rotation_translation(
                        Vec3::new(BACKROOM_CELL * 0.72, h, BACKROOM_WALL_THICKNESS),
                        Quat::IDENTITY,
                        Vec3::new(x, h * 0.5, z),
                    ),
                );
                world.wall_indices.push(idx);
            }
        }
    }

    // ── Fluorescent ceiling strips and low ceiling beams ──
    for room in 0..BACKROOM_ROOM_COUNT {
        let z = -(room as f32 * BACKROOM_CELL + BACKROOM_HALF_CELL);
        for lane in 0..BACKROOM_LANE_COUNT {
            let lane_offset = lane - BACKROOM_LANE_COUNT / 2;
            let x = lane_offset as f32 * BACKROOM_CELL;
            if let Ok(idx) = ctx.spawn_cube(Vec3::ZERO) {
                ctx.set_transform(
                    idx,
                    Mat4::from_scale_rotation_translation(
                        Vec3::new(1.2, 0.06, 0.32),
                        Quat::IDENTITY,
                        Vec3::new(x, h - 0.04, z),
                    ),
                );
                world.wall_indices.push(idx);
            }
            if room % 2 == 0 {
                if let Ok(idx) = ctx.spawn_cube(Vec3::ZERO) {
                    ctx.set_transform(
                        idx,
                        Mat4::from_scale_rotation_translation(
                            Vec3::new(BACKROOM_CELL, 0.12, 0.18),
                            Quat::IDENTITY,
                            Vec3::new(x, h - 0.18, z + 2.4),
                        ),
                    );
                    world.wall_indices.push(idx);
                }
            }
        }
    }

    // ── Columns/pillars: good occluders for future Hi-Z/delta tests ──
    for room in 0..BACKROOM_ROOM_COUNT {
        if room % 2 != 0 {
            continue;
        }
        let z = -(room as f32 * BACKROOM_CELL + 5.2);
        for &x in &[-pw, pw] {
            if let Ok(idx) = ctx.spawn_cube(Vec3::ZERO) {
                ctx.set_transform(
                    idx,
                    Mat4::from_scale_rotation_translation(
                        Vec3::new(0.42, h - 0.25, 0.42),
                        Quat::IDENTITY,
                        Vec3::new(x, (h - 0.25) * 0.5, z),
                    ),
                );
                world.pillar_indices.push(idx);
            }
        }
    }

    // ── End cap wall after final room ──
    if let Ok(idx) = ctx.spawn_cube(Vec3::ZERO) {
        ctx.set_transform(
            idx,
            Mat4::from_scale_rotation_translation(
                Vec3::new(total_half_width * 2.0, h, BACKROOM_WALL_THICKNESS),
                Quat::IDENTITY,
                Vec3::new(0.0, h * 0.5, -full_depth),
            ),
        );
        world.wall_indices.push(idx);
    }

    // ── Original corridor side hint remains as subtle guide rails for gameplay scale ──
    for i in 0..4 {
        let z = -(i as f32 * 18.0 + 9.0);
        if let Ok(idx) = ctx.spawn_cube(Vec3::ZERO) {
            ctx.set_transform(
                idx,
                Mat4::from_scale_rotation_translation(
                    Vec3::new(0.12, h * 0.85, 1.6),
                    Quat::IDENTITY,
                    Vec3::new(-w, h * 0.42, z),
                ),
            );
            world.wall_indices.push(idx);
        }
        if let Ok(idx) = ctx.spawn_cube(Vec3::ZERO) {
            ctx.set_transform(
                idx,
                Mat4::from_scale_rotation_translation(
                    Vec3::new(0.12, h * 0.85, 1.6),
                    Quat::IDENTITY,
                    Vec3::new(w, h * 0.42, z),
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
        (-5.8, -9.5, 1.25),
        (1.05, -15.0, 0.95),
        (5.35, -24.0, 1.55),
        (-1.45, -38.0, 1.10),
        (-7.35, -52.0, 1.35),
        (0.35, -66.5, 0.85),
        (6.25, -78.0, 1.25),
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
            obj.color = Vec4::new(0.42, 0.36, 0.18, 1.0); // alfombra amarilla húmeda
            obj.metallic = 0.0;
            obj.roughness = 0.62;
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
            obj.color = Vec4::new(0.92, 0.78, 0.36, 1.0); // wallpaper Backrooms
            obj.metallic = 0.0;
            obj.roughness = 0.78;
        }
    }

    for &idx in &world.pillar_indices {
        if let Some(obj) = ctx.scene.objects.get_mut(idx) {
            obj.color = Vec4::new(0.76, 0.63, 0.25, 1.0);
            obj.metallic = 0.0;
            obj.roughness = 0.7;
        }
    }
}
