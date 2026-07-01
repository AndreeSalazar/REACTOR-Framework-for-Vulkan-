use crate::core::frame_graph::types::*;
use crate::core::frame_graph::graph::FrameGraph;

pub struct PassBuilder<'a> {
    graph: &'a mut FrameGraph,
    pass_id: PassId,
    name: String,
    reads: Vec<ResourceId>,
    writes: Vec<ResourceId>,
    order: i32,
}

impl<'a> PassBuilder<'a> {
    pub fn new(graph: &'a mut FrameGraph, pass_id: PassId, name: String) -> Self {
        Self { graph, pass_id, name, reads: Vec::new(), writes: Vec::new(), order: 0 }
    }

    pub fn reads(mut self, resources: &[ResourceId]) -> Self {
        self.reads.extend(resources);
        self
    }

    pub fn read(mut self, resource: ResourceId) -> Self {
        self.reads.push(resource);
        self
    }

    pub fn writes(mut self, resources: &[ResourceId]) -> Self {
        self.writes.extend(resources);
        self
    }

    pub fn write(mut self, resource: ResourceId) -> Self {
        self.writes.push(resource);
        self
    }

    pub fn order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    pub fn build(self) -> PassId {
        let pass = PassDesc {
            id: self.pass_id,
            name: self.name,
            reads: self.reads,
            writes: self.writes,
            enabled: true,
            order: self.order,
        };
        self.graph.passes.insert(self.pass_id, pass);
        self.pass_id
    }
}
