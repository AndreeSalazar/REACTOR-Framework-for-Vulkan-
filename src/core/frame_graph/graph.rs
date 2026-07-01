use std::collections::{HashMap, HashSet};
use crate::core::frame_graph::types::*;
use crate::core::frame_graph::builder::PassBuilder;

#[derive(Clone, Debug, Default)]
pub struct FrameGraph {
    pub(super) resources: HashMap<ResourceId, ResourceDesc>,
    pub(super) passes: HashMap<PassId, PassDesc>,
    barriers: Vec<Barrier>,
    execution_order: Vec<PassId>,
    next_resource_id: u32,
    next_pass_id: u32,
    compiled: bool,
    pub stats: FrameGraphStats,
}

impl FrameGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_resource(
        &mut self,
        name: &str,
        resource_type: ResourceType,
        width: u32,
        height: u32,
        format: ResourceFormat,
    ) -> ResourceId {
        let id = ResourceId(self.next_resource_id);
        self.next_resource_id += 1;

        let desc = ResourceDesc {
            id,
            name: name.to_string(),
            resource_type,
            width,
            height,
            format,
            persistent: false,
        };

        self.resources.insert(id, desc);
        self.compiled = false;
        id
    }

    pub fn create_persistent_resource(
        &mut self,
        name: &str,
        resource_type: ResourceType,
        width: u32,
        height: u32,
        format: ResourceFormat,
    ) -> ResourceId {
        let id = self.create_resource(name, resource_type, width, height, format);
        if let Some(res) = self.resources.get_mut(&id) {
            res.persistent = true;
        }
        id
    }

    pub fn pass(&mut self, name: &str) -> PassBuilder<'_> {
        let id = PassId(self.next_pass_id);
        self.next_pass_id += 1;
        self.compiled = false;

        PassBuilder::new(self, id, name.to_string())
    }

    pub fn set_pass_enabled(&mut self, pass: PassId, enabled: bool) {
        if let Some(p) = self.passes.get_mut(&pass) {
            p.enabled = enabled;
            self.compiled = false;
        }
    }

    pub fn compile(&mut self) {
        if self.compiled {
            return;
        }

        self.barriers.clear();
        self.execution_order.clear();

        let mut enabled_passes: Vec<_> = self
            .passes
            .values()
            .filter(|p| p.enabled)
            .cloned()
            .collect();

        enabled_passes.sort_by_key(|p| p.order);

        let mut sorted = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();

        for pass in &enabled_passes {
            if !visited.contains(&pass.id) {
                self.topological_sort(
                    pass.id,
                    &enabled_passes,
                    &mut visited,
                    &mut temp_visited,
                    &mut sorted,
                );
            }
        }

        self.execution_order = sorted;
        self.generate_barriers();

        self.stats.total_passes = self.passes.len() as u32;
        self.stats.enabled_passes = self.execution_order.len() as u32;
        self.stats.total_resources = self.resources.len() as u32;
        self.stats.transient_resources =
            self.resources.values().filter(|r| !r.persistent).count() as u32;
        self.stats.barriers_generated = self.barriers.len() as u32;

        self.compiled = true;
    }

    fn topological_sort(
        &self,
        pass_id: PassId,
        passes: &[PassDesc],
        visited: &mut HashSet<PassId>,
        temp_visited: &mut HashSet<PassId>,
        sorted: &mut Vec<PassId>,
    ) {
        if temp_visited.contains(&pass_id) {
            let pass_name = passes
                .iter()
                .find(|p| p.id == pass_id)
                .map(|p| p.name.as_str())
                .unwrap_or("unknown");
            log::error!(
                "FrameGraph cycle detected involving pass '{}' (id: {:?}). \
                 This indicates a circular dependency in resource reads/writes. \
                 Please review your pass configuration.",
                pass_name,
                pass_id
            );
            #[cfg(debug_assertions)]
            panic!(
                "FrameGraph cycle detected! Pass '{}' (id: {:?}) creates a circular dependency.",
                pass_name, pass_id
            );
            #[cfg(not(debug_assertions))]
            return;
        }
        if visited.contains(&pass_id) {
            return;
        }

        temp_visited.insert(pass_id);

        if let Some(pass) = passes.iter().find(|p| p.id == pass_id) {
            for read_resource in &pass.reads {
                for other_pass in passes {
                    if other_pass.id != pass_id && other_pass.writes.contains(read_resource) {
                        self.topological_sort(other_pass.id, passes, visited, temp_visited, sorted);
                    }
                }
            }
        }

        temp_visited.remove(&pass_id);
        visited.insert(pass_id);
        sorted.push(pass_id);
    }

    fn generate_barriers(&mut self) {
        let mut last_write: HashMap<ResourceId, PassId> = HashMap::new();

        for &pass_id in &self.execution_order {
            if let Some(pass) = self.passes.get(&pass_id) {
                for &resource in &pass.reads {
                    if let Some(&writer) = last_write.get(&resource) {
                        self.barriers.push(Barrier {
                            resource,
                            from_pass: Some(writer),
                            to_pass: pass_id,
                            from_access: AccessType::Write,
                            to_access: AccessType::Read,
                        });
                    }
                }

                for &resource in &pass.writes {
                    last_write.insert(resource, pass_id);
                }
            }
        }
    }

    pub fn execution_order(&self) -> &[PassId] {
        &self.execution_order
    }

    pub fn barriers(&self) -> &[Barrier] {
        &self.barriers
    }

    pub fn get_pass(&self, id: PassId) -> Option<&PassDesc> {
        self.passes.get(&id)
    }

    pub fn get_resource(&self, id: ResourceId) -> Option<&ResourceDesc> {
        self.resources.get(&id)
    }

    pub fn print_debug(&self) {
        println!("╔══════════════════════════════════════════════════════════════════╗");
        println!("║                      FrameGraph Debug                            ║");
        println!("╠══════════════════════════════════════════════════════════════════╣");

        println!("║ Resources ({}):", self.resources.len());
        for (id, res) in &self.resources {
            println!(
                "║   [{:2}] {} ({:?}) {}x{}",
                id.0, res.name, res.resource_type, res.width, res.height
            );
        }

        println!("║");
        println!("║ Execution Order ({} passes):", self.execution_order.len());
        for (i, &pass_id) in self.execution_order.iter().enumerate() {
            if let Some(pass) = self.passes.get(&pass_id) {
                let reads: Vec<_> = pass.reads.iter().map(|r| r.0.to_string()).collect();
                let writes: Vec<_> = pass.writes.iter().map(|r| r.0.to_string()).collect();
                println!(
                    "║   {}. {} (reads: [{}], writes: [{}])",
                    i + 1,
                    pass.name,
                    reads.join(","),
                    writes.join(",")
                );
            }
        }

        println!("║");
        println!("║ Barriers ({}):", self.barriers.len());
        for barrier in &self.barriers {
            let from = barrier
                .from_pass
                .map(|p| p.0.to_string())
                .unwrap_or("?".to_string());
            println!(
                "║   Resource {} : Pass {} → Pass {} ({:?} → {:?})",
                barrier.resource.0, from, barrier.to_pass.0, barrier.from_access, barrier.to_access
            );
        }

        println!("╚══════════════════════════════════════════════════════════════════╝");
    }

    pub fn reset(&mut self) {
        self.resources.retain(|_, r| r.persistent);
        self.passes.clear();
        self.barriers.clear();
        self.execution_order.clear();
        self.next_pass_id = 0;
        self.compiled = false;
    }
}
