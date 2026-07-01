#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Meshlet {
    pub vertex_offset: u32,
    pub vertex_count: u32,
    pub index_offset: u32,
    pub index_count: u32,
    pub aabb_min: [f32; 3],
    pub aabb_max: [f32; 3],
    pub cone_apex: [f32; 3],
    pub cone_axis: [f32; 3],
    pub cone_cutoff: f32,
    pub _pad: [f32; 3],
}

pub struct MeshletBuilder;

impl MeshletBuilder {
    pub fn build(
        vertices: &[[f32; 3]],
        indices: &[u32],
        max_vertices: u32,
        max_triangles: u32,
    ) -> Vec<Meshlet> {
        let triangle_count = indices.len() / 3;
        let mut meshlets = Vec::new();

        let triangles_per_meshlet = max_triangles as usize;
        let mut current_triangle = 0;

        while current_triangle < triangle_count {
            let end_triangle = (current_triangle + triangles_per_meshlet).min(triangle_count);
            let index_start = current_triangle * 3;
            let index_end = end_triangle * 3;

            let mut aabb_min = [f32::MAX; 3];
            let mut aabb_max = [f32::MIN; 3];

            for i in (index_start..index_end).step_by(3) {
                for j in 0..3 {
                    let idx = indices[i + j] as usize;
                    if idx < vertices.len() {
                        let v = vertices[idx];
                        for k in 0..3 {
                            aabb_min[k] = aabb_min[k].min(v[k]);
                            aabb_max[k] = aabb_max[k].max(v[k]);
                        }
                    }
                }
            }

            let unique_vertices: std::collections::HashSet<u32> =
                indices[index_start..index_end].iter().copied().collect();

            if unique_vertices.len() <= max_vertices as usize {
                meshlets.push(Meshlet {
                    vertex_offset: 0,
                    vertex_count: unique_vertices.len() as u32,
                    index_offset: index_start as u32,
                    index_count: (index_end - index_start) as u32,
                    aabb_min,
                    aabb_max,
                    cone_apex: [0.0; 3],
                    cone_axis: [0.0, 0.0, 1.0],
                    cone_cutoff: 0.0,
                    _pad: [0.0; 3],
                });
            }

            current_triangle = end_triangle;
        }

        meshlets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meshlet_builder() {
        let vertices: Vec<[f32; 3]> = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let indices: Vec<u32> = vec![0, 1, 2, 1, 3, 2];

        let meshlets = MeshletBuilder::build(&vertices, &indices, 64, 124);
        assert_eq!(meshlets.len(), 1);
        assert_eq!(meshlets[0].index_count, 6);
        assert_eq!(meshlets[0].vertex_count, 4);
    }
}
