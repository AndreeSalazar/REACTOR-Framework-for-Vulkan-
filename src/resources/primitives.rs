use glam::{Vec2, Vec3};
use crate::resources::vertex::Vertex;

/// Primitive mesh generators
pub struct Primitives;

impl Primitives {
    /// Generate a cube with proper normals
    pub fn cube() -> (Vec<Vertex>, Vec<u32>) {
        let vertices = vec![
            // Front face (Z+)
            Vertex::new(Vec3::new(-0.5, -0.5,  0.5), Vec3::new(0.0, 0.0, 1.0), Vec2::new(0.0, 1.0)),
            Vertex::new(Vec3::new( 0.5, -0.5,  0.5), Vec3::new(0.0, 0.0, 1.0), Vec2::new(1.0, 1.0)),
            Vertex::new(Vec3::new( 0.5,  0.5,  0.5), Vec3::new(0.0, 0.0, 1.0), Vec2::new(1.0, 0.0)),
            Vertex::new(Vec3::new(-0.5,  0.5,  0.5), Vec3::new(0.0, 0.0, 1.0), Vec2::new(0.0, 0.0)),
            // Back face (Z-)
            Vertex::new(Vec3::new( 0.5, -0.5, -0.5), Vec3::new(0.0, 0.0, -1.0), Vec2::new(0.0, 1.0)),
            Vertex::new(Vec3::new(-0.5, -0.5, -0.5), Vec3::new(0.0, 0.0, -1.0), Vec2::new(1.0, 1.0)),
            Vertex::new(Vec3::new(-0.5,  0.5, -0.5), Vec3::new(0.0, 0.0, -1.0), Vec2::new(1.0, 0.0)),
            Vertex::new(Vec3::new( 0.5,  0.5, -0.5), Vec3::new(0.0, 0.0, -1.0), Vec2::new(0.0, 0.0)),
            // Right face (X+)
            Vertex::new(Vec3::new( 0.5, -0.5,  0.5), Vec3::new(1.0, 0.0, 0.0), Vec2::new(0.0, 1.0)),
            Vertex::new(Vec3::new( 0.5, -0.5, -0.5), Vec3::new(1.0, 0.0, 0.0), Vec2::new(1.0, 1.0)),
            Vertex::new(Vec3::new( 0.5,  0.5, -0.5), Vec3::new(1.0, 0.0, 0.0), Vec2::new(1.0, 0.0)),
            Vertex::new(Vec3::new( 0.5,  0.5,  0.5), Vec3::new(1.0, 0.0, 0.0), Vec2::new(0.0, 0.0)),
            // Left face (X-)
            Vertex::new(Vec3::new(-0.5, -0.5, -0.5), Vec3::new(-1.0, 0.0, 0.0), Vec2::new(0.0, 1.0)),
            Vertex::new(Vec3::new(-0.5, -0.5,  0.5), Vec3::new(-1.0, 0.0, 0.0), Vec2::new(1.0, 1.0)),
            Vertex::new(Vec3::new(-0.5,  0.5,  0.5), Vec3::new(-1.0, 0.0, 0.0), Vec2::new(1.0, 0.0)),
            Vertex::new(Vec3::new(-0.5,  0.5, -0.5), Vec3::new(-1.0, 0.0, 0.0), Vec2::new(0.0, 0.0)),
            // Top face (Y+)
            Vertex::new(Vec3::new(-0.5,  0.5,  0.5), Vec3::new(0.0, 1.0, 0.0), Vec2::new(0.0, 1.0)),
            Vertex::new(Vec3::new( 0.5,  0.5,  0.5), Vec3::new(0.0, 1.0, 0.0), Vec2::new(1.0, 1.0)),
            Vertex::new(Vec3::new( 0.5,  0.5, -0.5), Vec3::new(0.0, 1.0, 0.0), Vec2::new(1.0, 0.0)),
            Vertex::new(Vec3::new(-0.5,  0.5, -0.5), Vec3::new(0.0, 1.0, 0.0), Vec2::new(0.0, 0.0)),
            // Bottom face (Y-)
            Vertex::new(Vec3::new(-0.5, -0.5, -0.5), Vec3::new(0.0, -1.0, 0.0), Vec2::new(0.0, 1.0)),
            Vertex::new(Vec3::new( 0.5, -0.5, -0.5), Vec3::new(0.0, -1.0, 0.0), Vec2::new(1.0, 1.0)),
            Vertex::new(Vec3::new( 0.5, -0.5,  0.5), Vec3::new(0.0, -1.0, 0.0), Vec2::new(1.0, 0.0)),
            Vertex::new(Vec3::new(-0.5, -0.5,  0.5), Vec3::new(0.0, -1.0, 0.0), Vec2::new(0.0, 0.0)),
        ];

        let indices = vec![
            0, 1, 2, 2, 3, 0,       // Front
            4, 5, 6, 6, 7, 4,       // Back
            8, 9, 10, 10, 11, 8,   // Right
            12, 13, 14, 14, 15, 12, // Left
            16, 17, 18, 18, 19, 16, // Top
            20, 21, 22, 22, 23, 20, // Bottom
        ];

        (vertices, indices)
    }

    /// Generate a UV sphere
    pub fn sphere(segments: u32, rings: u32) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for ring in 0..=rings {
            let v = ring as f32 / rings as f32;
            let phi = v * std::f32::consts::PI;

            for segment in 0..=segments {
                let u = segment as f32 / segments as f32;
                let theta = u * std::f32::consts::TAU;

                let x = phi.sin() * theta.cos();
                let y = phi.cos();
                let z = phi.sin() * theta.sin();

                let position = Vec3::new(x, y, z) * 0.5;
                let normal = Vec3::new(x, y, z).normalize();
                let uv = Vec2::new(u, v);

                vertices.push(Vertex::new(position, normal, uv));
            }
        }

        for ring in 0..rings {
            for segment in 0..segments {
                let current = ring * (segments + 1) + segment;
                let next = current + segments + 1;

                indices.push(current);
                indices.push(next);
                indices.push(current + 1);

                indices.push(current + 1);
                indices.push(next);
                indices.push(next + 1);
            }
        }

        (vertices, indices)
    }

    /// Generate a plane (XZ)
    pub fn plane(subdivisions: u32) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let step = 1.0 / subdivisions as f32;

        for z in 0..=subdivisions {
            for x in 0..=subdivisions {
                let px = x as f32 * step - 0.5;
                let pz = z as f32 * step - 0.5;
                let u = x as f32 / subdivisions as f32;
                let v = z as f32 / subdivisions as f32;

                vertices.push(Vertex::new(
                    Vec3::new(px, 0.0, pz),
                    Vec3::Y,
                    Vec2::new(u, v),
                ));
            }
        }

        for z in 0..subdivisions {
            for x in 0..subdivisions {
                let current = z * (subdivisions + 1) + x;
                let next = current + subdivisions + 1;

                indices.push(current);
                indices.push(next);
                indices.push(current + 1);

                indices.push(current + 1);
                indices.push(next);
                indices.push(next + 1);
            }
        }

        (vertices, indices)
    }

    /// Generate a cylinder
    pub fn cylinder(segments: u32, height: f32, radius: f32) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let half_height = height / 2.0;

        // Side vertices
        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = angle.cos() * radius;
            let z = angle.sin() * radius;
            let u = i as f32 / segments as f32;

            // Bottom
            vertices.push(Vertex::new(
                Vec3::new(x, -half_height, z),
                Vec3::new(angle.cos(), 0.0, angle.sin()),
                Vec2::new(u, 1.0),
            ));

            // Top
            vertices.push(Vertex::new(
                Vec3::new(x, half_height, z),
                Vec3::new(angle.cos(), 0.0, angle.sin()),
                Vec2::new(u, 0.0),
            ));
        }

        // Side indices
        for i in 0..segments {
            let base = i * 2;
            indices.push(base);
            indices.push(base + 1);
            indices.push(base + 2);

            indices.push(base + 1);
            indices.push(base + 3);
            indices.push(base + 2);
        }

        // Top cap center
        let top_center_idx = vertices.len() as u32;
        vertices.push(Vertex::new(
            Vec3::new(0.0, half_height, 0.0),
            Vec3::Y,
            Vec2::new(0.5, 0.5),
        ));

        // Bottom cap center
        let bottom_center_idx = vertices.len() as u32;
        vertices.push(Vertex::new(
            Vec3::new(0.0, -half_height, 0.0),
            Vec3::NEG_Y,
            Vec2::new(0.5, 0.5),
        ));

        // Cap vertices and indices
        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = angle.cos() * radius;
            let z = angle.sin() * radius;
            let u = angle.cos() * 0.5 + 0.5;
            let v = angle.sin() * 0.5 + 0.5;

            // Top cap
            let top_idx = vertices.len() as u32;
            vertices.push(Vertex::new(
                Vec3::new(x, half_height, z),
                Vec3::Y,
                Vec2::new(u, v),
            ));

            // Bottom cap
            let bottom_idx = vertices.len() as u32;
            vertices.push(Vertex::new(
                Vec3::new(x, -half_height, z),
                Vec3::NEG_Y,
                Vec2::new(u, v),
            ));

            if i < segments {
                // Top cap triangle
                indices.push(top_center_idx);
                indices.push(top_idx);
                indices.push(top_idx + 2);

                // Bottom cap triangle
                indices.push(bottom_center_idx);
                indices.push(bottom_idx + 2);
                indices.push(bottom_idx);
            }
        }

        (vertices, indices)
    }

    /// Generate a cone
    pub fn cone(segments: u32, height: f32, radius: f32) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let half_height = height / 2.0;

        // Apex
        let apex_idx = 0u32;
        vertices.push(Vertex::new(
            Vec3::new(0.0, half_height, 0.0),
            Vec3::Y,
            Vec2::new(0.5, 0.0),
        ));

        // Base vertices
        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = angle.cos() * radius;
            let z = angle.sin() * radius;

            // Calculate normal (pointing outward and up)
            let slope = radius / height;
            let normal = Vec3::new(angle.cos(), slope, angle.sin()).normalize();

            vertices.push(Vertex::new(
                Vec3::new(x, -half_height, z),
                normal,
                Vec2::new(i as f32 / segments as f32, 1.0),
            ));
        }

        // Side triangles
        for i in 0..segments {
            indices.push(apex_idx);
            indices.push(1 + i);
            indices.push(2 + i);
        }

        // Base center
        let base_center_idx = vertices.len() as u32;
        vertices.push(Vertex::new(
            Vec3::new(0.0, -half_height, 0.0),
            Vec3::NEG_Y,
            Vec2::new(0.5, 0.5),
        ));

        // Base cap
        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = angle.cos() * radius;
            let z = angle.sin() * radius;

            let idx = vertices.len() as u32;
            vertices.push(Vertex::new(
                Vec3::new(x, -half_height, z),
                Vec3::NEG_Y,
                Vec2::new(angle.cos() * 0.5 + 0.5, angle.sin() * 0.5 + 0.5),
            ));

            if i < segments {
                indices.push(base_center_idx);
                indices.push(idx + 1);
                indices.push(idx);
            }
        }

        (vertices, indices)
    }

    /// Generate a torus
    pub fn torus(ring_segments: u32, tube_segments: u32, ring_radius: f32, tube_radius: f32) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for ring in 0..=ring_segments {
            let ring_angle = (ring as f32 / ring_segments as f32) * std::f32::consts::TAU;
            let ring_cos = ring_angle.cos();
            let ring_sin = ring_angle.sin();

            for tube in 0..=tube_segments {
                let tube_angle = (tube as f32 / tube_segments as f32) * std::f32::consts::TAU;
                let tube_cos = tube_angle.cos();
                let tube_sin = tube_angle.sin();

                let x = (ring_radius + tube_radius * tube_cos) * ring_cos;
                let y = tube_radius * tube_sin;
                let z = (ring_radius + tube_radius * tube_cos) * ring_sin;

                let normal = Vec3::new(
                    tube_cos * ring_cos,
                    tube_sin,
                    tube_cos * ring_sin,
                ).normalize();

                vertices.push(Vertex::new(
                    Vec3::new(x, y, z),
                    normal,
                    Vec2::new(ring as f32 / ring_segments as f32, tube as f32 / tube_segments as f32),
                ));
            }
        }

        for ring in 0..ring_segments {
            for tube in 0..tube_segments {
                let current = ring * (tube_segments + 1) + tube;
                let next = current + tube_segments + 1;

                indices.push(current);
                indices.push(next);
                indices.push(current + 1);

                indices.push(current + 1);
                indices.push(next);
                indices.push(next + 1);
            }
        }

        (vertices, indices)
    }

    /// Generate a quad (XY plane, facing +Z)
    pub fn quad() -> (Vec<Vertex>, Vec<u32>) {
        let vertices = vec![
            Vertex::new(Vec3::new(-0.5, -0.5, 0.0), Vec3::Z, Vec2::new(0.0, 1.0)),
            Vertex::new(Vec3::new( 0.5, -0.5, 0.0), Vec3::Z, Vec2::new(1.0, 1.0)),
            Vertex::new(Vec3::new( 0.5,  0.5, 0.0), Vec3::Z, Vec2::new(1.0, 0.0)),
            Vertex::new(Vec3::new(-0.5,  0.5, 0.0), Vec3::Z, Vec2::new(0.0, 0.0)),
        ];

        let indices = vec![0, 1, 2, 2, 3, 0];

        (vertices, indices)
    }

    /// Generate a fullscreen triangle (for post-processing)
    pub fn fullscreen_triangle() -> (Vec<Vertex>, Vec<u32>) {
        let vertices = vec![
            Vertex::new(Vec3::new(-1.0, -1.0, 0.0), Vec3::Z, Vec2::new(0.0, 0.0)),
            Vertex::new(Vec3::new( 3.0, -1.0, 0.0), Vec3::Z, Vec2::new(2.0, 0.0)),
            Vertex::new(Vec3::new(-1.0,  3.0, 0.0), Vec3::Z, Vec2::new(0.0, 2.0)),
        ];

        let indices = vec![0, 1, 2];

        (vertices, indices)
    }
}
