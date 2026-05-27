// Tool to inspect zombie_basic.glb geometry bounds, materials, and textures
use std::path::Path;

fn main() {
    println!("🔍 Inspecting zombie_basic.glb...");
    let path = Path::new("assets/models/zombie_basic.glb");
    if !path.exists() {
        println!("❌ File not found!");
        return;
    }

    match gltf::import(path) {
        Ok((doc, buffers, images)) => {
            println!("✅ glTF loaded successfully.");
            println!("Meshes count: {}", doc.meshes().count());
            println!("Materials count: {}", doc.materials().count());
            println!("Textures count: {}", doc.textures().count());
            println!("Images count: {}", images.len());

            for (i, tex) in doc.textures().enumerate() {
                println!("Texture #{} Index: {}, Source Image: {}", i, tex.index(), tex.source().index());
            }

            for (i, img) in images.iter().enumerate() {
                println!("Image #{} Format: {:?}, Dimensions: {}x{}", i, img.format, img.width, img.height);
            }

            let mut min_x = f32::MAX;
            let mut max_x = f32::MIN;
            let mut min_y = f32::MAX;
            let mut max_y = f32::MIN;
            let mut min_z = f32::MAX;
            let mut max_z = f32::MIN;
            let mut vertex_count = 0;

            for mesh in doc.meshes() {
                println!("Mesh name: {:?}", mesh.name());
                for primitive in mesh.primitives() {
                    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                    if let Some(positions) = reader.read_positions() {
                        for pos in positions {
                            min_x = min_x.min(pos[0]);
                            max_x = max_x.max(pos[0]);
                            min_y = min_y.min(pos[1]);
                            max_y = max_y.max(pos[1]);
                            min_z = min_z.min(pos[2]);
                            max_z = max_z.max(pos[2]);
                            vertex_count += 1;
                        }
                    }
                }
            }

            println!();
            println!("📊 Geometry Bounds:");
            println!("  Vertex Count: {}", vertex_count);
            println!(
                "  X Bounds:     [{:.3}, {:.3}] (Width: {:.3})",
                min_x,
                max_x,
                max_x - min_x
            );
            println!(
                "  Y Bounds:     [{:.3}, {:.3}] (Height: {:.3})",
                min_y,
                max_y,
                max_y - min_y
            );
            println!(
                "  Z Bounds:     [{:.3}, {:.3}] (Depth: {:.3})",
                min_z,
                max_z,
                max_z - min_z
            );
        }
        Err(e) => {
            println!("❌ Failed to load glTF: {}", e);
        }
    }
}
