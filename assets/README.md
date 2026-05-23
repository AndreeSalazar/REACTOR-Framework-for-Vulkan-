# 📦 Assets para XENOFALL — Asset Pipeline (Fase 3)

## Estructura de Directorios

```
assets/
├── models/
│   ├── zombie_basic.glb      # Zombie estándar (low-poly)
│   ├── zombie_fast.glb       # Zombie corredor
│   ├── zombie_tank.glb       # Zombie resistente  
│   ├── weapon_pistol.glb     # Pistola del jugador
│   └── corridor_segment.glb  # Segmento reutilizable del corredor
│
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
│
├── materials/
│   ├── zombie_skin.ron
│   ├── metal_weapon.ron
│   └── concrete_wall.ron
│
└── config/
    └── waves.json            # Definición de oleadas externa
```

## 🔧 Cómo Añadir un Modelo glTF

### 1. Exportar desde Blender/3ds Max/Maya

```bash
# Blender: File → Export → glTF 2.0 (.glb/.gltf)
# Configuración recomendada:
#   ☑ Include → Selected Objects
#   ☑ Transform → +Y Up
#   ☑ Geometry → Apply Modifiers
#   ☑ Animation → Si tiene animaciones
```

### 2. Colocar en `assets/models/`

```bash
cp mi_zombie.glb assets/models/zombie_basic.glb
```

### 3. (Opcional) Optimizar con glTF-Transform

```bash
# Instalar: https://gltf-transform.dev/
gltf-transform optimize assets/models/zombie_basic.glb --texture-compress ktx2 --meshopt
```

### 4. Usar en el código

```rust
// En tu juego, usar spawn_gltf en lugar de spawn_cube:
fn spawn_enemy(&mut self, ctx: &mut ReactorContext, pos: Vec3) {
    // Intentar cargar modelo glTF
    if let Ok(indices) = ctx.spawn_gltf(
        "assets/models/zombie_basic.glb",
        Mat4::from_translation(pos)
    ) {
        // Modelo cargado exitosamente
        for scene_idx in indices {
            // Usar scene_idx para controlar el enemigo
        }
    } else {
        // Fallback a primitiva si el modelo no está disponible
        if let Ok(idx) = ctx.spawn_cube(Vec3::ZERO) {
            ctx.set_transform(idx, Mat4::from_translation(pos));
        }
    }
}
```

## 🔄 Hot-Reload de Assets

El Asset Pipeline soporta recarga automática:

1. **Editar** un archivo `.glb` o textura en `assets/`
2. **Guardar** el archivo
3. **REACTOR detecta el cambio** y recarga automáticamente
4. **El juego se actualiza** sin reiniciar

```rust
// Trackear un asset para hot-reload en init():
ctx.track_asset_for_reload("assets/models/zombie.glb", AssetType::Model)?;
```

## 📊 Asset Loading Asíncrono

Para no bloquear el frame inicial, usa la cola de carga:

```rust
use reactor_vulkan::resources::{AssetId, LoadPriority};

// Enqueue de carga en background
let path = "assets/models/zombie_basic.glb";
let id = AssetId::from_path(path);

let receiver = ctx.asset_loader_queue.enqueue_gltf(
    id,
    path.into(),
    LoadPriority::High, // Critical, High, Normal, Low
);

// El receiver se completa cuando el asset está listo
// (puedes usar tokio::spawn para manejar el resultado)
```

## 🎨 Formatos Soportados

| Tipo | Extensiones | Notas |
|------|-------------|-------|
| **Modelos** | `.glb`, `.gltf` | glTF 2.0 con PBR |
| **Texturas** | `.png`, `.jpg`, `.ktx2`, `.dds` | KTX2 recomendado para BC7/ASTC |
| **Materiales** | `.ron`, `.json` | Definiciones PBR personalizadas |
| **Animaciones** | `.glb` (embedded) | glTF animations (Fase 3.2) |

## 🚀 Optimizaciones Recomendadas

1. **Usar KTX2 para texturas**: Compresión BC7/ASTC reduce VRAM 4-8x
2. **Meshopt para geometría**: Reduce tamaño de meshes 30-50%
3. **Mipmaps generados**: Mejor calidad a distancia + menos aliasing
4. **Assets compartidos**: Reutilizar materiales entre instancias

```bash
# Ejemplo de pipeline de optimización:
gltf-transform draco assets/models/zombie.glb output.glb
gltf-transform webp output.glb --texture-quality 80
```

## 🐛 Troubleshooting

### Modelo no aparece
- ✅ Verificar que el path es relativo a `assets/`
- ✅ Chequear logs: `[AssetHotReload] Watching: assets`
- ✅ Validar glTF con https://github.khronos.org/glTF-Validator/

### Texturas faltantes
- ✅ Asegurar que las URIs en el glTF son relativas
- ✅ Verificar formato soportado (PNG/JPG/KTX2)
- ✅ Chequear permisos de lectura en `assets/`

### Hot-reload no funciona
- ✅ Verificar que `notify` está habilitado en tu plataforma
- ✅ Chequear que el archivo no está bloqueado por otro proceso
- ✅ Revisar logs: `[AssetHotReload] Recargando: ...`

---

> 💡 **Pro Tip**: Usa `ctx.asset_stats()` en tu HUD para monitorear el estado del Asset Pipeline en tiempo real.

```rust
// En update_hud():
let stats = ctx.asset_stats();
println!("Assets: {} loaded, {} queued, {} hot-reload tracked",
    stats.db.total_entries,
    stats.loader_queue.queued,
    stats.hot_reload.map(|h| h.tracked_count).unwrap_or(0)
);
```
