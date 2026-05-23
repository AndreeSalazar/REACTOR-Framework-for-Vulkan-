# 🎨 FASE 3: Asset Pipeline para REACTOR

Sistema completo de carga, gestión y hot-reload de assets para juegos REACTOR.

## ✨ Características

- **glTF 2.0 Loading**: Carga de modelos 3D con soporte PBR completo
- **AssetId estable**: Hash basado en contenido para identificación única
- **Handle<T>**: Sistema de referencias con reference counting (tipo UE5/Unity)
- **Hot-Reload**: Detección automática de cambios en archivos con recarga en vivo
- **Async Loading**: Cola de carga en background sin bloquear el frame
- **Asset Database**: Persistencia de metadata con `sled` (KV store embedded)
- **KTX2 Support**: Texturas comprimidas con Basis Universal

## 📦 Estructura de Assets

```
assets/
├── models/
│   ├── zombie_basic.glb      # Modelo zombie estándar
│   ├── zombie_fast.glb       # Zombie corredor
│   ├── zombie_tank.glb       # Zombie resistente  
│   ├── weapon_pistol.glb     # Arma del jugador
│   └── corridor_segment.glb  # Segmento reutilizable
├── textures/
│   ├── zombie/
│   │   ├── diffuse.ktx2      # BC7 compressed
│   │   ├── normal.ktx2
│   │   └── roughness.ktx2
│   ├── weapons/
│   │   └── pistol_*.ktx2
│   └── environment/
│       ├── walls_*.ktx2
│       └── floor_*.ktx2
├── materials/
│   ├── zombie_skin.ron
│   ├── metal_weapon.ron
│   └── concrete_wall.ron
└── config/
    └── waves.json            # Definición de oleadas
```

## 🚀 Uso Básico

### Cargar un modelo glTF

```rust
use reactor_vulkan::prelude::*;
use reactor_vulkan::resources::{AssetType, LoadPriority};

fn init(&mut self, ctx: &mut ReactorContext) {
    // Carga síncrona (bloqueante, usar solo en init)
    if let Ok(model) = ctx.load_gltf("assets/models/zombie.glb") {
        println!("Modelo cargado: {} meshes, {} materiales", 
                 model.meshes.len(), model.materials.len());
    }
    
    // Carga asíncrona (no bloqueante)
    let _ = ctx.load_gltf_async("assets/models/zombie.glb");
    
    // Carga en cola de background (recomendado para streaming)
    let zombie_id = AssetId::from_path("assets/models/zombie.glb");
    let load_rx = ctx.asset_loader_queue.enqueue_gltf(
        zombie_id,
        "assets/models/zombie.glb".into(),
        LoadPriority::High,
    );
}
```

### Spawn de modelo en escena

```rust
fn spawn_enemy(&mut self, ctx: &mut ReactorContext, pos: Vec3) {
    // Spawn directo con transform
    if let Ok(indices) = ctx.spawn_gltf(
        "assets/models/zombie.glb",
        Mat4::from_translation(pos)
    ) {
        // indices contiene los scene_index de cada mesh del modelo
        for idx in indices {
            // Guardar para actualizar posición/animación después
            self.enemy_indices.push(idx);
        }
    }
}
```

### Hot-Reload de assets

```rust
fn init(&mut self, ctx: &mut ReactorContext) {
    // Trackear asset para hot-reload
    let asset_id = ctx.track_asset_for_reload(
        "assets/models/zombie.glb",
        AssetType::Model
    ).unwrap();
    
    // En el game loop, procesar eventos de reload
    // (implementar en tu sistema de eventos)
}
```

### Estadísticas del Asset Pipeline

```rust
fn update(&mut self, ctx: &mut ReactorContext) {
    let stats = ctx.asset_stats();
    
    println!("Asset Pipeline Stats:");
    println!("  - Loader Queue: {} queued, {} loading", 
             stats.loader_queue.queued, stats.loader_queue.loading);
    println!("  - GLTF Cache: {} models, {} textures",
             stats.gltf_cache.models_cached, stats.gltf_cache.textures_cached);
    println!("  - DB Size: {} entries, {} bytes on disk",
             stats.db.total_entries, stats.db.size_on_disk);
}
```

## 🛠️ Conversión de Assets

### Blender → glTF

```bash
# Exportar desde Blender (File → Export → glTF 2.0)
# Configuración recomendada:
# - Format: glTF Binary (.glb)
# - Include: ✓ Selected Objects, ✓ Visible Objects
# - Transform: ✓ Apply Modifiers, ✓ UVs, ✓ Normals, ✓ Tangents
# - Animation: ✓ Skinning (si aplica)
```

### Texturas → KTX2 (optimizado)

```bash
# Usando toktx (Basis Universal CLI)
toktx --bcmp --generate_mipmap assets/textures/zombie_diffuse.png assets/textures/zombie_diffuse.ktx2

# Opciones recomendadas:
# --bcmp: Compresión BC7 (calidad máxima)
# --generate_mipmap: Generar mipmaps automáticamente
# --assign_oetf linear: Para texturas no-sRGB (normals, roughness)
```

### Script de conversión automática

```bash
#!/bin/bash
# scripts/convert_assets.sh

ASSETS_DIR="assets"
OUTPUT_DIR="assets_cooked"

# Convertir texturas a KTX2
find "$ASSETS_DIR/textures" -name "*.png" -o -name "*.jpg" | while read img; do
    rel_path="${img#$ASSETS_DIR/}"
    out_path="$OUTPUT_DIR/$rel_path"
    out_path="${out_path%.*}.ktx2"
    
    mkdir -p "$(dirname "$out_path")"
    
    if [[ "$img" == *"_normal"* ]] || [[ "$img" == *"_roughness"* ]]; then
        toktx --bcmp --assign_oetf linear --generate_mipmap "$img" "$out_path"
    else
        toktx --bcmp --generate_mipmap "$img" "$out_path"
    fi
done

echo "✅ Assets convertidos a $OUTPUT_DIR"
```

## 🔧 Configuración Avanzada

### Personalizar Hot-Reload

```rust
use reactor_vulkan::resources::{HotReloadConfig, AssetHotReloadManager};

let config = HotReloadConfig {
    watch_dirs: vec!["assets/models".into(), "assets/textures".into()],
    extensions: HashSet::from(["glb".into(), "ktx2".into()]),
    debounce_duration: Duration::from_millis(500), // Más lento para archivos grandes
    ignore_patterns: vec![".git".into(), "tmp".into()],
    auto_reload: true,
};
```

### Prioridades de carga

```rust
use reactor_vulkan::resources::LoadPriority;

// Critical: Assets necesarios para el frame actual
ctx.asset_loader_queue.enqueue_gltf(id, path, LoadPriority::Critical);

// High: Assets que se verán pronto (próximos 2-3 frames)
ctx.asset_loader_queue.enqueue_gltf(id, path, LoadPriority::High);

// Normal: Assets en background (precaching)
ctx.asset_loader_queue.enqueue_gltf(id, path, LoadPriority::Normal);

// Low: Assets opcionales o muy lejanos
ctx.asset_loader_queue.enqueue_gltf(id, path, LoadPriority::Low);
```

## 🐛 Troubleshooting

### "Failed to load glTF: ..."

```
Causas comunes:
1. Ruta incorrecta: Verificar que el path sea relativo al ejecutable o usar path absoluto
2. Formato no soportado: Asegurar que el archivo sea .glb o .gltf válido
3. Texturas faltantes: Si el glTF referencia texturas externas, deben estar en la ruta correcta

Solución:
- Usar `gltf-validator` para verificar la validez del archivo:
  npm install -g gltf-validator
  gltf-validator assets/models/zombie.glb
```

### Hot-Reload no detecta cambios

```
Posibles causas:
1. El archivo está en una carpeta ignorada (.git, target, tmp)
2. El debounce_duration es demasiado largo
3. El watcher no tiene permisos en el sistema operativo

Solución:
- Verificar HotReloadConfig::ignore_patterns
- Reducir debounce_duration para testing
- En Linux: aumentar inotify limits:
  echo fs.inotify.max_user_watches=524288 | sudo tee -a /etc/sysctl.conf
  sudo sysctl -p
```

### Memoria alta por cache de assets

```
El Asset Pipeline mantiene assets en memoria por defecto.
Para liberar memoria:

1. Usar WeakHandle para referencias que no deben prevenir unload:
   let weak = handle.downgrade();
   if let Some(strong) = weak.upgrade() { /* asset aún cargado */ }

2. Limpiar cache manualmente:
   ctx.gltf_loader.clear_cache();

3. Configurar límite de cache en AssetDatabase (futuro)
```

## 📊 Métricas de Rendimiento

| Operación | Tiempo típico | Notas |
|-----------|--------------|-------|
| Carga glTF simple (<1MB) | 10-50ms | Depende de complejidad del mesh |
| Carga glTF complejo (10MB+) | 100-500ms | Usar carga asíncrona |
| Hot-reload textura KTX2 | 5-20ms | Compresión BC7 acelera upload GPU |
| AssetId hash calculation | <1ms | XXH3 es muy rápido |
| Handle clone | ~10ns | Solo incrementa refcount |

## 🔄 Migración desde Primitivas

### Antes (cubos primitivos)
```rust
// Spawn enemigo como cubo
if let Ok(idx) = ctx.spawn_cube(pos) {
    ctx.set_transform(idx, Mat4::from_scale(Vec3::new(0.6, 1.6, 0.4)));
    self.enemies.push(Enemy { scene_index: idx, ... });
}
```

### Después (modelo glTF)
```rust
// Spawn enemigo como modelo glTF
if let Ok(indices) = ctx.spawn_gltf("assets/zombie.glb", 
    Mat4::from_scale_rotation_translation(
        Vec3::new(0.8, 0.8, 0.8),
        Quat::IDENTITY,
        pos
    )
) {
    for idx in indices {
        self.enemies.push(Enemy { scene_index: idx, ... });
    }
}
// Fallback automático a cubo si el modelo no está disponible
```

## 🎯 Próximos Pasos (Fase 3.2)

- [ ] Animaciones glTF: Skeletal animation y blending
- [ ] LOD System: Niveles de detalle automáticos
- [ ] Asset Streaming: Carga progresiva de mundos abiertos
- [ ] Dependency Graph: Reload en cascada para materiales/texturas
- [ ] Asset Cooker CLI: `reactor cook` para pre-procesamiento

---

> **Nota**: El Asset Pipeline está diseñado para ser **transparente**. Si un asset no está disponible, el sistema hace fallback gracefully a primitivas, permitiendo que el juego funcione en cualquier entorno de desarrollo.

```
🎮 REACTOR Fase 3 — Assets listos para producción
```
