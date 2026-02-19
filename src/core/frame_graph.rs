// =============================================================================
// FrameGraph - Deterministic Render Graph for REACTOR
// =============================================================================
// Un FrameGraph explícito, declarativo, 100% visible:
// - Sin auto-reordenamientos ocultos
// - Barreras Vulkan generadas pero visibles
// - Posibilidad de congelar el graph y debuggear frame por frame
// =============================================================================

use std::collections::{HashMap, HashSet};

/// Identificador único de un recurso en el FrameGraph
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ResourceId(pub u32);

/// Identificador único de un pass en el FrameGraph
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PassId(pub u32);

/// Tipo de recurso
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceType {
    /// Textura/Image
    Texture,
    /// Buffer
    Buffer,
    /// Depth buffer
    DepthBuffer,
    /// Render target
    RenderTarget,
    /// Swapchain image
    Swapchain,
}

/// Descriptor de un recurso
#[derive(Clone, Debug)]
pub struct ResourceDesc {
    pub id: ResourceId,
    pub name: String,
    pub resource_type: ResourceType,
    pub width: u32,
    pub height: u32,
    pub format: ResourceFormat,
    pub persistent: bool,
}

/// Formato de recurso simplificado
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceFormat {
    RGBA8,
    RGBA16F,
    RGBA32F,
    R8,
    R16F,
    R32F,
    Depth32F,
    Depth24Stencil8,
}

/// Tipo de acceso a un recurso
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessType {
    /// Lectura (shader sample, etc.)
    Read,
    /// Escritura (render target, storage, etc.)
    Write,
    /// Lectura y escritura
    ReadWrite,
}

/// Dependencia de un pass
#[derive(Clone, Debug)]
pub struct PassDependency {
    pub resource: ResourceId,
    pub access: AccessType,
}

/// Descriptor de un render pass
#[derive(Clone, Debug)]
pub struct PassDesc {
    pub id: PassId,
    pub name: String,
    pub reads: Vec<ResourceId>,
    pub writes: Vec<ResourceId>,
    pub enabled: bool,
    pub order: i32,
}

/// Barrera de sincronización generada
#[derive(Clone, Debug)]
pub struct Barrier {
    pub resource: ResourceId,
    pub from_pass: Option<PassId>,
    pub to_pass: PassId,
    pub from_access: AccessType,
    pub to_access: AccessType,
}

/// Builder para crear passes de forma fluida
pub struct PassBuilder<'a> {
    graph: &'a mut FrameGraph,
    pass_id: PassId,
    name: String,
    reads: Vec<ResourceId>,
    writes: Vec<ResourceId>,
    order: i32,
}

impl<'a> PassBuilder<'a> {
    /// Agregar recursos que este pass lee
    pub fn reads(mut self, resources: &[ResourceId]) -> Self {
        self.reads.extend(resources);
        self
    }

    /// Agregar un recurso que este pass lee
    pub fn read(mut self, resource: ResourceId) -> Self {
        self.reads.push(resource);
        self
    }

    /// Agregar recursos que este pass escribe
    pub fn writes(mut self, resources: &[ResourceId]) -> Self {
        self.writes.extend(resources);
        self
    }

    /// Agregar un recurso que este pass escribe
    pub fn write(mut self, resource: ResourceId) -> Self {
        self.writes.push(resource);
        self
    }

    /// Establecer orden explícito
    pub fn order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    /// Finalizar y registrar el pass
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

/// FrameGraph principal
#[derive(Clone, Debug, Default)]
pub struct FrameGraph {
    /// Recursos registrados
    resources: HashMap<ResourceId, ResourceDesc>,
    /// Passes registrados
    passes: HashMap<PassId, PassDesc>,
    /// Barreras calculadas
    barriers: Vec<Barrier>,
    /// Orden de ejecución calculado
    execution_order: Vec<PassId>,
    /// Contador de recursos
    next_resource_id: u32,
    /// Contador de passes
    next_pass_id: u32,
    /// Si el graph está compilado
    compiled: bool,
    /// Estadísticas
    pub stats: FrameGraphStats,
}

/// Estadísticas del FrameGraph
#[derive(Clone, Debug, Default)]
pub struct FrameGraphStats {
    pub total_passes: u32,
    pub enabled_passes: u32,
    pub total_resources: u32,
    pub transient_resources: u32,
    pub barriers_generated: u32,
}

impl FrameGraph {
    /// Crear nuevo FrameGraph
    pub fn new() -> Self {
        Self::default()
    }

    /// Registrar un nuevo recurso
    pub fn create_resource(&mut self, name: &str, resource_type: ResourceType, width: u32, height: u32, format: ResourceFormat) -> ResourceId {
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

    /// Registrar recurso persistente (no se destruye entre frames)
    pub fn create_persistent_resource(&mut self, name: &str, resource_type: ResourceType, width: u32, height: u32, format: ResourceFormat) -> ResourceId {
        let id = self.create_resource(name, resource_type, width, height, format);
        if let Some(res) = self.resources.get_mut(&id) {
            res.persistent = true;
        }
        id
    }

    /// Crear un nuevo pass con builder
    pub fn pass(&mut self, name: &str) -> PassBuilder<'_> {
        let id = PassId(self.next_pass_id);
        self.next_pass_id += 1;
        self.compiled = false;

        PassBuilder {
            graph: self,
            pass_id: id,
            name: name.to_string(),
            reads: Vec::new(),
            writes: Vec::new(),
            order: 0,
        }
    }

    /// Habilitar/deshabilitar un pass
    pub fn set_pass_enabled(&mut self, pass: PassId, enabled: bool) {
        if let Some(p) = self.passes.get_mut(&pass) {
            p.enabled = enabled;
            self.compiled = false;
        }
    }

    /// Compilar el graph (calcular orden y barreras)
    pub fn compile(&mut self) {
        if self.compiled {
            return;
        }

        self.barriers.clear();
        self.execution_order.clear();

        // Recopilar passes habilitados
        let mut enabled_passes: Vec<_> = self.passes.values()
            .filter(|p| p.enabled)
            .cloned()
            .collect();

        // Ordenar por orden explícito primero
        enabled_passes.sort_by_key(|p| p.order);

        // Ordenamiento topológico basado en dependencias
        let mut sorted = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();

        for pass in &enabled_passes {
            if !visited.contains(&pass.id) {
                self.topological_sort(pass.id, &enabled_passes, &mut visited, &mut temp_visited, &mut sorted);
            }
        }

        self.execution_order = sorted;

        // Generar barreras
        self.generate_barriers();

        // Actualizar estadísticas
        self.stats.total_passes = self.passes.len() as u32;
        self.stats.enabled_passes = self.execution_order.len() as u32;
        self.stats.total_resources = self.resources.len() as u32;
        self.stats.transient_resources = self.resources.values().filter(|r| !r.persistent).count() as u32;
        self.stats.barriers_generated = self.barriers.len() as u32;

        self.compiled = true;
    }

    /// Ordenamiento topológico recursivo
    fn topological_sort(
        &self,
        pass_id: PassId,
        passes: &[PassDesc],
        visited: &mut HashSet<PassId>,
        temp_visited: &mut HashSet<PassId>,
        sorted: &mut Vec<PassId>,
    ) {
        if temp_visited.contains(&pass_id) {
            // Ciclo detectado - ignorar (o panic en debug)
            return;
        }
        if visited.contains(&pass_id) {
            return;
        }

        temp_visited.insert(pass_id);

        // Encontrar dependencias (passes que escriben recursos que este lee)
        if let Some(pass) = passes.iter().find(|p| p.id == pass_id) {
            for read_resource in &pass.reads {
                // Buscar pass que escribe este recurso
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

    /// Generar barreras de sincronización
    fn generate_barriers(&mut self) {
        let mut last_write: HashMap<ResourceId, PassId> = HashMap::new();

        for &pass_id in &self.execution_order {
            if let Some(pass) = self.passes.get(&pass_id) {
                // Para cada recurso que leemos
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

                // Actualizar último escritor
                for &resource in &pass.writes {
                    last_write.insert(resource, pass_id);
                }
            }
        }
    }

    /// Obtener orden de ejecución
    pub fn execution_order(&self) -> &[PassId] {
        &self.execution_order
    }

    /// Obtener barreras generadas
    pub fn barriers(&self) -> &[Barrier] {
        &self.barriers
    }

    /// Obtener información de un pass
    pub fn get_pass(&self, id: PassId) -> Option<&PassDesc> {
        self.passes.get(&id)
    }

    /// Obtener información de un recurso
    pub fn get_resource(&self, id: ResourceId) -> Option<&ResourceDesc> {
        self.resources.get(&id)
    }

    /// Imprimir el graph para debug
    pub fn print_debug(&self) {
        println!("╔══════════════════════════════════════════════════════════════════╗");
        println!("║                      FrameGraph Debug                            ║");
        println!("╠══════════════════════════════════════════════════════════════════╣");
        
        println!("║ Resources ({}):", self.resources.len());
        for (id, res) in &self.resources {
            println!("║   [{:2}] {} ({:?}) {}x{}", 
                id.0, res.name, res.resource_type, res.width, res.height);
        }
        
        println!("║");
        println!("║ Execution Order ({} passes):", self.execution_order.len());
        for (i, &pass_id) in self.execution_order.iter().enumerate() {
            if let Some(pass) = self.passes.get(&pass_id) {
                let reads: Vec<_> = pass.reads.iter().map(|r| r.0.to_string()).collect();
                let writes: Vec<_> = pass.writes.iter().map(|r| r.0.to_string()).collect();
                println!("║   {}. {} (reads: [{}], writes: [{}])", 
                    i + 1, pass.name, reads.join(","), writes.join(","));
            }
        }
        
        println!("║");
        println!("║ Barriers ({}):", self.barriers.len());
        for barrier in &self.barriers {
            let from = barrier.from_pass.map(|p| p.0.to_string()).unwrap_or("?".to_string());
            println!("║   Resource {} : Pass {} → Pass {} ({:?} → {:?})",
                barrier.resource.0, from, barrier.to_pass.0, 
                barrier.from_access, barrier.to_access);
        }
        
        println!("╚══════════════════════════════════════════════════════════════════╝");
    }

    /// Reset para nuevo frame
    pub fn reset(&mut self) {
        // Mantener recursos persistentes, limpiar transient
        self.resources.retain(|_, r| r.persistent);
        self.passes.clear();
        self.barriers.clear();
        self.execution_order.clear();
        self.next_pass_id = 0;
        self.compiled = false;
    }
}

/// Preset de FrameGraph para deferred rendering
pub fn create_deferred_graph(width: u32, height: u32) -> FrameGraph {
    let mut graph = FrameGraph::new();

    // Recursos
    let depth = graph.create_resource("Depth", ResourceType::DepthBuffer, width, height, ResourceFormat::Depth32F);
    let albedo = graph.create_resource("GBuffer_Albedo", ResourceType::RenderTarget, width, height, ResourceFormat::RGBA8);
    let normal = graph.create_resource("GBuffer_Normal", ResourceType::RenderTarget, width, height, ResourceFormat::RGBA16F);
    let position = graph.create_resource("GBuffer_Position", ResourceType::RenderTarget, width, height, ResourceFormat::RGBA32F);
    let lit = graph.create_resource("Lit", ResourceType::RenderTarget, width, height, ResourceFormat::RGBA16F);
    let final_output = graph.create_resource("Final", ResourceType::Swapchain, width, height, ResourceFormat::RGBA8);

    // Passes
    graph.pass("GBuffer")
        .write(depth)
        .write(albedo)
        .write(normal)
        .write(position)
        .order(0)
        .build();

    graph.pass("Lighting")
        .read(depth)
        .read(albedo)
        .read(normal)
        .read(position)
        .write(lit)
        .order(1)
        .build();

    graph.pass("PostProcess")
        .read(lit)
        .write(final_output)
        .order(2)
        .build();

    graph.compile();
    graph
}

/// Preset de FrameGraph para forward rendering simple
pub fn create_forward_graph(width: u32, height: u32) -> FrameGraph {
    let mut graph = FrameGraph::new();

    let depth = graph.create_resource("Depth", ResourceType::DepthBuffer, width, height, ResourceFormat::Depth32F);
    let color = graph.create_resource("Color", ResourceType::Swapchain, width, height, ResourceFormat::RGBA8);

    graph.pass("Forward")
        .write(depth)
        .write(color)
        .order(0)
        .build();

    graph.compile();
    graph
}
