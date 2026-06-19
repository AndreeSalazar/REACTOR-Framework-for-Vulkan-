//! Thin wrapper around `ReactorContext` that adds builder-style helpers.

use crate::app::ReactorContext;

pub struct App<'a> {
    ctx: &'a mut ReactorContext,
}

impl<'a> App<'a> {
    pub fn new(ctx: &'a mut ReactorContext) -> Self {
        Self { ctx }
    }

    pub fn ctx(&mut self) -> &mut ReactorContext {
        self.ctx
    }

    pub fn camera(&mut self) -> crate::app_helpers::camera_setup::CameraSetup<'_> {
        crate::app_helpers::camera_setup::CameraSetup::new(self.ctx)
    }

    pub fn lighting(&mut self) -> crate::app_helpers::lighting_setup::LightingSetup<'_> {
        crate::app_helpers::lighting_setup::LightingSetup::new(self.ctx)
    }

    pub fn mesh(&mut self) -> crate::app_helpers::mesh_builder::MeshBuilder<'_> {
        crate::app_helpers::mesh_builder::MeshBuilder::new(self.ctx)
    }
}
