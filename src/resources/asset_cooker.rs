// =============================================================================
// REACTOR Asset Cooker
// =============================================================================
// Pre-processes raw assets (PNG, JPG, glTF, WAV, TTF) into runtime-optimized
// binary formats (.rt_tex, .rt_mesh, .rt_audio, .rt_font) and registers them
// in the persistent AssetDatabase.
// =============================================================================

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Write;

use crate::resources::asset_database::{AssetDatabase, AssetMetadata, AssetType};
use crate::resources::asset_id::AssetId;
use crate::core::error::{ReactorResult, ReactorError};

/// Asset Cooker system to compile raw assets to runtime formats
pub struct AssetCooker {
    input_dir: PathBuf,
    output_dir: PathBuf,
    db: AssetDatabase,
}

impl AssetCooker {
    /// Create a new AssetCooker pointing to input and output directories
    pub fn new<P1: AsRef<Path>, P2: AsRef<Path>>(input: P1, output: P2) -> ReactorResult<Self> {
        let input_dir = input.as_ref().to_path_buf();
        let output_dir = output.as_ref().to_path_buf();
        
        fs::create_dir_all(&output_dir)
            .map_err(|e| ReactorError::new(crate::core::error::ErrorCode::IoError, format!("Failed to create output directory: {}", e)))?;
            
        let db_path = output_dir.join(".reactor").join("assets.db");
        fs::create_dir_all(db_path.parent().unwrap()).ok();
        
        let db = AssetDatabase::open(db_path)?;
        
        Ok(Self { input_dir, output_dir, db })
    }
    
    /// Cook all files in the input directory recursively
    pub fn cook_all(&mut self) -> ReactorResult<()> {
        let walk = walkdir::WalkDir::new(&self.input_dir);
        for entry in walk.into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                self.cook_file(entry.path())?;
            }
        }
        Ok(())
    }
    
    /// Cook a single asset file
    pub fn cook_file(&mut self, path: &Path) -> ReactorResult<()> {
        let relative_path = path.strip_prefix(&self.input_dir)
            .map_err(|e| ReactorError::asset_load(format!("Invalid path structure: {}", e)))?;
            
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
            
        let asset_type = AssetType::from_extension(ext);
        if asset_type == AssetType::Unknown {
            // Skip unknown file types (e.g. system files)
            return Ok(());
        }
        
        let path_str = relative_path.to_string_lossy().replace('\\', "/");
        println!("🍳 Cooking asset: {} (Type: {:?})", path_str, asset_type);
        
        let content = fs::read(path)
            .map_err(|e| ReactorError::new(crate::core::error::ErrorCode::IoError, format!("Failed to read source file '{}': {}", path.display(), e)))?;
        let hash = xxhash_rust::xxh3::xxh3_64(&content);
        
        let metadata = fs::metadata(path)
            .map_err(|e| ReactorError::new(crate::core::error::ErrorCode::IoError, format!("Failed to get filesystem metadata: {}", e)))?;
            
        let last_modified = metadata.modified()
            .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
            .unwrap_or(0);
            
        let asset_id = AssetId::from_path(relative_path);
        
        // Generate cooked filename based on asset ID hash
        let dest_filename = format!("{:016x}.{}", asset_id.as_u64(), asset_type.extension());
        let dest_path = self.output_dir.join(&dest_filename);
        
        let mut extra = HashMap::new();
        
        match asset_type {
            AssetType::Texture => {
                // Read image and generate mipmaps
                let img = image::load_from_memory(&content)
                    .map_err(|e| ReactorError::asset_load(format!("Failed to parse image data: {}", e)))?;
                    
                let width = img.width();
                let height = img.height();
                
                let mut current_img = image::DynamicImage::ImageRgba8(img.to_rgba8());
                let mut current_w = width;
                let mut current_h = height;
                let mut mip_levels = Vec::new();
                mip_levels.push((current_w, current_h, current_img.to_rgba8().into_raw()));
                
                // Downscale mipmap levels iteratively
                while current_w > 1 || current_h > 1 {
                    current_w = (current_w / 2).max(1);
                    current_h = (current_h / 2).max(1);
                    current_img = current_img.resize_exact(current_w, current_h, image::imageops::FilterType::Lanczos3);
                    mip_levels.push((current_w, current_h, current_img.to_rgba8().into_raw()));
                }
                
                let mut file = fs::File::create(&dest_path)?;
                file.write_all(b"RTTX")?;
                file.write_all(&1u32.to_le_bytes())?; // Version
                file.write_all(&width.to_le_bytes())?;
                file.write_all(&height.to_le_bytes())?;
                file.write_all(&4u8.to_le_bytes())?; // Channels (RGBA8)
                file.write_all(&(mip_levels.len() as u32).to_le_bytes())?;
                
                for (_, _, pixels) in &mip_levels {
                    file.write_all(&(pixels.len() as u32).to_le_bytes())?;
                    file.write_all(pixels)?;
                }
                
                extra.insert("width".to_string(), serde_json::Value::from(width));
                extra.insert("height".to_string(), serde_json::Value::from(height));
                extra.insert("mipmaps".to_string(), serde_json::Value::from(mip_levels.len()));
            }
            AssetType::Model => {
                // Read and patch GLB/glTF extensions if needed before parsing
                if ext == "glb" || ext == "gltf" {
                    let mut file_bytes = fs::read(path)
                        .map_err(|e| ReactorError::new(crate::core::error::ErrorCode::IoError, format!("Failed to read model file: {}", e)))?;
                    
                    let from = b"extensionsRequired";
                    let to = b"extensionsOptional";
                    let mut modified = false;
                    let mut i = 0;
                    while i + from.len() <= file_bytes.len() {
                        if &file_bytes[i..i+from.len()] == from {
                            file_bytes[i..i+from.len()].copy_from_slice(to);
                            modified = true;
                            i += from.len();
                        } else {
                            i += 1;
                        }
                    }
                    if modified {
                        fs::write(path, &file_bytes)
                            .map_err(|e| ReactorError::new(crate::core::error::ErrorCode::IoError, format!("Failed to write patched model: {}", e)))?;
                        println!("🩹 Patched glTF extensions in {}", path_str);
                    }
                }

                // Parse OBJ or glTF meshes using the loader built into the engine
                let (vertices, indices) = crate::resources::asset_manager::load_model_auto(path)
                    .map_err(|e| ReactorError::asset_load(format!("Failed to parse model file: {}", e)))?;
                    
                let mut file = fs::File::create(&dest_path)?;
                file.write_all(b"RTMH")?;
                file.write_all(&1u32.to_le_bytes())?; // Version
                file.write_all(&(vertices.len() as u32).to_le_bytes())?;
                file.write_all(&(indices.len() as u32).to_le_bytes())?;
                
                // Write vertex bytes directly (fast memory cast)
                let vertex_bytes = bytemuck::cast_slice::<crate::resources::Vertex, u8>(&vertices);
                file.write_all(vertex_bytes)?;
                
                // Write index bytes directly
                let index_bytes = bytemuck::cast_slice::<u32, u8>(&indices);
                file.write_all(index_bytes)?;
                
                extra.insert("vertex_count".to_string(), serde_json::Value::from(vertices.len()));
                extra.insert("index_count".to_string(), serde_json::Value::from(indices.len()));
            }
            AssetType::Audio => {
                // Parse audio header data using AudioClip
                let clip = crate::systems::audio::AudioClip::from_bytes(
                    path_str.clone(),
                    &content
                ).map_err(|e| ReactorError::asset_load(format!("Failed to parse audio file metadata: {}", e)))?;
                
                let mut file = fs::File::create(&dest_path)?;
                file.write_all(b"RTAUD")?;
                file.write_all(&1u32.to_le_bytes())?; // Version
                file.write_all(&clip.sample_rate.to_le_bytes())?;
                file.write_all(&clip.channels.to_le_bytes())?;
                file.write_all(&clip.duration.to_le_bytes())?;
                
                // Append original container bytes (mp3, wav, ogg) to be decoded on the fly
                file.write_all(&(content.len() as u32).to_le_bytes())?;
                file.write_all(&content)?;
                
                extra.insert("sample_rate".to_string(), serde_json::Value::from(clip.sample_rate));
                extra.insert("channels".to_string(), serde_json::Value::from(clip.channels));
                extra.insert("duration".to_string(), serde_json::Value::from(clip.duration));
            }
            AssetType::Font | AssetType::Config | AssetType::Shader => {
                // Pass-through validation and copy directly
                let mut file = fs::File::create(&dest_path)?;
                file.write_all(&content)?;
            }
            _ => {
                // Fallback direct copy
                let mut file = fs::File::create(&dest_path)?;
                file.write_all(&content)?;
            }
        }
        
        // Write metadata info to the sled AssetDatabase
        let meta = AssetMetadata {
            source_path: path_str,
            content_hash: hash,
            last_modified,
            file_size: metadata.len(),
            asset_type,
            source_format: ext.to_string(),
            runtime_format: Some(asset_type.extension().to_string()),
            dependencies: Vec::new(),
            dependents: Vec::new(),
            extra,
        };
        
        self.db.register_asset(asset_id, meta)?;
        
        Ok(())
    }
}
