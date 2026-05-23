#!/bin/bash
# =============================================================================
# REACTOR Asset Cooker Script — Fase 3
# =============================================================================
# Convierte assets RAW a formatos optimizados para runtime:
# - Imágenes PNG/JPG → KTX2 con compresión BC7 + mipmaps
# - Modelos FBX/OBJ → glTF/GLB optimizado
# - Genera metadata para AssetDatabase
#
# Uso: ./scripts/cook_assets.sh [input_dir] [output_dir]
# Ejemplo: ./scripts/cook_assets.sh assets_raw assets_cooked
# =============================================================================

set -e  # Exit on error

# Configuración por defecto
INPUT_DIR="${1:-assets}"
OUTPUT_DIR="${2:-assets_cooked}"
COMPRESS_LEVEL="${COMPRESS_LEVEL:-medium}"  # fast, medium, best

# Colores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info()    { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn()    { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error()   { echo -e "${RED}[ERROR]${NC} $1"; }

# Verificar dependencias
check_dependencies() {
    local missing=()
    
    if ! command -v toktx &> /dev/null; then
        missing+=("toktx (Basis Universal CLI)")
    fi
    if ! command -v gltf-transform &> /dev/null; then
        missing+=("gltf-transform (Node.js package)")
    fi
    if ! command -v xxhsum &> /dev/null; then
        missing+=("xxhsum (xxHash CLI)")
    fi
    
    if [ ${#missing[@]} -gt 0 ]; then
        log_warn "Dependencias faltantes:"
        for dep in "${missing[@]}"; do
            echo "  - $dep"
        done
        echo ""
        echo "Instalar con:"
        echo "  # toktx: https://github.com/BinomialLLC/basis_universal/releases"
        echo "  # gltf-transform: npm install -g @gltf-transform/cli"
        echo "  # xxhsum: apt install xxhash / brew install xxhash"
        echo ""
        log_warn "Continuando sin optimizaciones avanzadas..."
        return 1
    fi
    return 0
}

# Crear estructura de directorios de salida
setup_output_dirs() {
    mkdir -p "$OUTPUT_DIR/models"
    mkdir -p "$OUTPUT_DIR/textures"
    mkdir -p "$OUTPUT_DIR/materials"
    mkdir -p "$OUTPUT_DIR/.reactor"
    log_info "Estructura de salida creada en: $OUTPUT_DIR"
}

# Convertir textura a KTX2 con compresión BC7
convert_texture() {
    local input="$1"
    local output="$2"
    local is_normal="${3:-false}"
    
    if [ ! -f "$input" ]; then
        log_warn "Archivo no encontrado: $input"
        return 1
    fi
    
    # Determinar opciones según tipo de textura
    local toktx_opts="--bcmp --generate_mipmap"
    if [ "$is_normal" = true ] || [[ "$input" == *"_normal"* ]] || [[ "$input" == *"_roughness"* ]] || [[ "$input" == *"_metallic"* ]]; then
        # Texturas lineales (no sRGB): normals, roughness, metallic, AO
        toktx_opts="$toktx_opts --assign_oetf linear"
        log_info "Convirtiendo textura lineal: $(basename "$input")"
    else
        # Texturas sRGB: diffuse/albedo, emissive
        log_info "Convirtiendo textura sRGB: $(basename "$input")"
    fi
    
    # Ejecutar conversión
    if toktx $toktx_opts "$input" "$output" 2>/dev/null; then
        local original_size=$(stat -f%z "$input" 2>/dev/null || stat -c%s "$input" 2>/dev/null)
        local compressed_size=$(stat -f%z "$output" 2>/dev/null || stat -c%s "$output" 2>/dev/null)
        local ratio=$(echo "scale=1; $compressed_size * 100 / $original_size" | bc 2>/dev/null || echo "N/A")
        log_success "✓ $(basename "$input") → $(basename "$output") (${ratio}% del original)"
        return 0
    else
        log_error "Falló conversión de: $input"
        # Fallback: copiar archivo original
        cp "$input" "$output"
        log_warn "Usando archivo original sin compresión"
        return 1
    fi
}

# Optimizar modelo glTF/GLB
optimize_model() {
    local input="$1"
    local output="$2"
    
    if [ ! -f "$input" ]; then
        log_warn "Archivo no encontrado: $input"
        return 1
    fi
    
    log_info "Optimizando modelo: $(basename "$input")"
    
    # Usar gltf-transform para optimizar
    if command -v gltf-transform &> /dev/null; then
        gltf-transform optimize "$input" "$output" \
            --compress draco \
            --texture-compress ktx2 \
            --texture-size 2048 \
            --instance false \
            2>/dev/null && {
            log_success "✓ Modelo optimizado: $(basename "$output")"
            return 0
        }
    fi
    
    # Fallback: copiar archivo original
    cp "$input" "$output"
    log_warn "Usando modelo original sin optimización"
    return 1
}

# Calcular hash XXH3 para AssetId
compute_asset_hash() {
    local file="$1"
    if command -v xxhsum &> /dev/null; then
        xxhsum -q "$file" 2>/dev/null | cut -d' ' -f1
    else
        # Fallback simple: hash del tamaño + nombre
        echo "$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null)_$(basename "$file")" | md5sum | cut -d' ' -f1
    fi
}

# Generar metadata JSON para AssetDatabase
generate_metadata() {
    local source_path="$1"
    local cooked_path="$2"
    local asset_type="$3"
    
    local content_hash=$(compute_asset_hash "$cooked_path")
    local file_size=$(stat -f%z "$cooked_path" 2>/dev/null || stat -c%s "$cooked_path" 2>/dev/null)
    local timestamp=$(date +%s)
    
    cat <<EOF
{
  "source_path": "$source_path",
  "cooked_path": "$cooked_path",
  "content_hash": "$content_hash",
  "file_size": $file_size,
  "asset_type": "$asset_type",
  "cooked_at": $timestamp,
  "format": {
    "original": "${source_path##*.}",
    "runtime": "${cooked_path##*.}"
  }
}
EOF
}

# Procesar todas las texturas
process_textures() {
    log_info "Procesando texturas..."
    
    find "$INPUT_DIR/textures" -type f \( -iname "*.png" -o -iname "*.jpg" -o -iname "*.jpeg" -o -iname "*.tga" \) 2>/dev/null | while read -r texture; do
        local rel_path="${texture#$INPUT_DIR/}"
        local output_path="$OUTPUT_DIR/textures/${rel_path%.*}.ktx2"
        
        mkdir -p "$(dirname "$output_path")"
        
        # Detectar si es textura lineal por nombre
        local is_normal=false
        if [[ "$texture" == *"_normal"* ]] || [[ "$texture" == *"_roughness"* ]] || [[ "$texture" == *"_metallic"* ]] || [[ "$texture" == *"_ao"* ]]; then
            is_normal=true
        fi
        
        convert_texture "$texture" "$output_path" "$is_normal"
        
        # Generar metadata
        local meta_path="${output_path%.*}.meta.json"
        generate_metadata "$rel_path" "${rel_path%.*}.ktx2" "Texture" > "$meta_path"
    done
    
    log_success "Texturas procesadas"
}

# Procesar todos los modelos
process_models() {
    log_info "Procesando modelos 3D..."
    
    # Procesar glTF/GLB existentes (optimizar)
    find "$INPUT_DIR/models" -type f \( -iname "*.gltf" -o -iname "*.glb" \) 2>/dev/null | while read -r model; do
        local rel_path="${model#$INPUT_DIR/}"
        local output_path="$OUTPUT_DIR/models/$rel_path"
        
        mkdir -p "$(dirname "$output_path")"
        optimize_model "$model" "$output_path"
        
        # Generar metadata
        local meta_path="${output_path}.meta.json"
        generate_metadata "$rel_path" "$rel_path" "Model" > "$meta_path"
    done
    
    # Procesar FBX/OBJ (convertir a glTF)
    if command -v gltf-transform &> /dev/null; then
        find "$INPUT_DIR/models" -type f \( -iname "*.fbx" -o -iname "*.obj" \) 2>/dev/null | while read -r model; do
            local rel_path="${model#$INPUT_DIR/}"
            local base_name="${rel_path%.*}"
            local output_path="$OUTPUT_DIR/models/${base_name}.glb"
            
            mkdir -p "$(dirname "$output_path")"
            
            log_info "Convirtiendo $(basename "$model") → GLB"
            gltf-transform convert "$model" "$output_path" 2>/dev/null && {
                optimize_model "$output_path" "$output_path"
                generate_metadata "$rel_path" "${base_name}.glb" "Model" > "${output_path}.meta.json"
                log_success "✓ Conversión completada"
            } || log_error "Falló conversión de: $model"
        done
    else
        log_warn "gltf-transform no disponible: saltando conversión FBX/OBJ"
    fi
    
    log_success "Modelos procesados"
}

# Generar índice de assets para carga rápida
generate_asset_index() {
    log_info "Generando índice de assets..."
    
    local index_file="$OUTPUT_DIR/.reactor/assets_index.json"
    
    echo "{" > "$index_file"
    echo '  "version": "3.0",' >> "$index_file"
    echo '  "generated_at": '$(date +%s)',' >> "$index_file"
    echo '  "assets": {' >> "$index_file"
    
    local first=true
    find "$OUTPUT_DIR" -name "*.meta.json" -type f | while read -r meta; do
        if [ "$first" = true ]; then
            first=false
        else
            echo "," >> "$index_file"
        fi
        
        # Extraer info del metadata
        local source=$(grep -o '"source_path": *"[^"]*"' "$meta" | cut -d'"' -f4)
        local cooked=$(grep -o '"cooked_path": *"[^"]*"' "$meta" | cut -d'"' -f4)
        local hash=$(grep -o '"content_hash": *"[^"]*"' "$meta" | cut -d'"' -f4)
        local type=$(grep -o '"asset_type": *"[^"]*"' "$meta" | cut -d'"' -f4)
        
        echo -n "    \"$hash\": {\"source\": \"$source\", \"cooked\": \"$cooked\", \"type\": \"$type\"}" >> "$index_file"
    done
    
    echo "" >> "$index_file"
    echo "  }" >> "$index_file"
    echo "}" >> "$index_file"
    
    log_success "Índice generado: $index_file"
}

# Main
main() {
    echo "╔════════════════════════════════════════════════════════╗"
    echo "║  🎨 REACTOR Asset Cooker — Fase 3                     ║"
    echo "╠════════════════════════════════════════════════════════╣"
    echo "║  Input:  $INPUT_DIR"
    echo "║  Output: $OUTPUT_DIR"
    echo "║  Compression: $COMPRESS_LEVEL"
    echo "╚════════════════════════════════════════════════════════╝"
    echo ""
    
    check_dependencies || true
    setup_output_dirs
    
    process_textures
    process_models
    generate_asset_index
    
    echo ""
    log_success "✅ Asset cooking completado!"
    echo ""
    echo "📊 Resumen:"
    echo "  - Texturas KTX2: $(find "$OUTPUT_DIR/textures" -name "*.ktx2" 2>/dev/null | wc -l)"
    echo "  - Modelos GLB: $(find "$OUTPUT_DIR/models" -name "*.glb" 2>/dev/null | wc -l)"
    echo "  - Metadata files: $(find "$OUTPUT_DIR" -name "*.meta.json" 2>/dev/null | wc -l)"
    echo ""
    echo "💡 Para usar en tu juego:"
    echo "  1. Configura ctx.asset_db con la ruta a '$OUTPUT_DIR/.reactor'"
    echo "  2. Los assets se cargarán automáticamente desde la carpeta cooked"
    echo "  3. El hot-reload funcionará con los archivos originales en '$INPUT_DIR'"
}

# Ejecutar
main "$@"
