#![allow(unused_imports)]

use crate::Xenofall;
use crate::xenofall::{
    audio::GameAudio,
    cards::{Build, CardType},
    constants::*,
    helpers::{
        hash_rand, hash_rand_signed, pick_random_cards, ray_headshot_intersect,
        ray_sphere_intersect,
    },
    types::{Enemy, EnemyState, FireMode, GameState, Impact, Tracer, WaveDef},
    vfx::VfxPools,
    world::WorldGeometry,
};
use reactor_vulkan::graphics::post_process::PostProcessEffect;
use reactor_vulkan::prelude::*;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

impl Xenofall {
    pub(crate) fn build_corridor(&mut self, ctx: &mut ReactorContext) {
        self.world = crate::xenofall::world::build_corridor(ctx);
    }

    pub(crate) fn build_pools(&mut self, ctx: &mut ReactorContext) {
        self.vfx = crate::xenofall::vfx::build_pools(ctx);
    }

    pub(crate) fn apply_render_showcase_profile(&mut self, ctx: &mut ReactorContext) {
        crate::xenofall::render_lab::apply_professional_profile(ctx);
    }

    pub(crate) fn apply_render_showcase_materials(&mut self, ctx: &mut ReactorContext) {
        crate::xenofall::world::apply_world_materials(ctx, &self.world);
        crate::xenofall::vfx::apply_vfx_materials(ctx, &self.vfx);
    }

    pub(crate) fn print_render_showcase_budget(&self, ctx: &ReactorContext) {
        crate::xenofall::render_lab::log_phase_one_budget(ctx);
    }


}
