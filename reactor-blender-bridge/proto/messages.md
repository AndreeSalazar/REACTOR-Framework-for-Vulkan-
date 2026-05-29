# REACTOR Bridge — Protocolo (PROTOCOL_VERSION = 1)

Especificación independiente de lenguaje. Implementaciones de referencia:
- Rust: [`reactor_bridge::protocol`](../reactor_bridge/src/protocol.rs)
- Python: [`blender_addon.transport.protocol`](../blender_addon/transport/protocol.py)

---

## 🚚 Transporte

- **WebSocket** (RFC 6455) sobre TCP.
- **Default**: `ws://127.0.0.1:19840`.
- **Mensajes**: marcos de texto WebSocket conteniendo **JSON UTF-8**.
- **Encoding alternativo** (futuro): MessagePack como marcos binarios — el primer byte de la cadena determina (`{` = JSON, otro = msgpack).

---

## 📦 Sobre de mensaje

Todo mensaje sigue el patrón **internally-tagged** de serde:

```json
{
  "type": "<MessageType>",
  "data": { ... }
}
```

- `type`: nombre PascalCase del mensaje.
- `data`: payload específico.

Rust lo genera con:

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Message { Hello(Hello), HelloAck(HelloAck), ... }
```

Python lo emite a mano:

```python
{"type": "Hello", "data": {...}}
```

---

## 🔁 Flujo de conexión

```diagram
Cliente                            Servidor
  │                                   │
  ├──── Hello { client, caps } ─────▶│
  │                                   │
  │◀──── HelloAck { server, caps } ──┤
  │                                   │
  ├──── Ping { seq, ts_micros } ─────▶│
  │◀──── Pong { seq, ts, server_ts } ─┤
  │           (loop ~1 Hz)            │
  │                                   │
  ├──── Goodbye { reason } ──────────▶│
  │◀──── (close frame) ───────────────┤
```

Si la versión es incompatible, el servidor responde **HelloAck con `accepted=false`** y cierra.

---

## 🧬 Definiciones (FASE 0)

### `Hello` (Cliente → Server)

```json
{
  "type": "Hello",
  "data": {
    "version": 1,
    "client": "blender_addon",
    "capabilities": ["scene_push", "live_transforms"]
  }
}
```

### `HelloAck` (Server → Cliente)

```json
{
  "type": "HelloAck",
  "data": {
    "version": 1,
    "server": "reactor_bridge",
    "accepted": true,
    "capabilities": ["ping", "scene_apply"],
    "reason": null
  }
}
```

### `Ping` (Cliente → Server)

```json
{
  "type": "Ping",
  "data": {
    "seq": 42,
    "ts_micros": 1717084800123456
  }
}
```

### `Pong` (Server → Cliente)

```json
{
  "type": "Pong",
  "data": {
    "seq": 42,
    "client_ts_micros": 1717084800123456,
    "server_ts_micros": 1717084800124000
  }
}
```

### `Error` (Bidireccional)

```json
{
  "type": "Error",
  "data": {
    "code": "UNKNOWN_MESSAGE",
    "message": "Unsupported message type: Foo"
  }
}
```

### `Goodbye` (Bidireccional)

```json
{
  "type": "Goodbye",
  "data": { "reason": "user closed Blender" }
}
```

---

## 🚨 Códigos de error estándar

| Código                   | Significado                                          |
|--------------------------|------------------------------------------------------|
| `INCOMPATIBLE_VERSION`   | El cliente usa una versión de protocolo no soportada |
| `UNKNOWN_MESSAGE`        | Tipo de mensaje desconocido                          |
| `MALFORMED_PAYLOAD`      | El JSON no valida contra el schema                   |
| `NOT_HANDSHAKED`         | Se envió un mensaje antes del Hello                  |
| `INTERNAL`               | Excepción en el servidor (ver `message`)             |

---

## 🧬 Definiciones Avanzadas (Fases 1-8)

Los siguientes esquemas especifican los tipos de mensajes dinámicos que se construirán sobre el sobre de transporte.

### `EntityCreated` (Creación de Entidad)
```json
{
  "type": "EntityCreated",
  "data": {
    "id": "e7c2a123-28db-4cb8-8c10-53bc1bf5b8f3",
    "name": "SciFi_Door_01",
    "kind": "Mesh",
    "parent_id": null
  }
}
```
*Tipos de `kind` válidos: `"Mesh"`, `"Light"`, `"Camera"`, `"Empty"`.*

### `EntityRemoved` (Eliminación de Entidad)
```json
{
  "type": "EntityRemoved",
  "data": {
    "id": "e7c2a123-28db-4cb8-8c10-53bc1bf5b8f3"
  }
}
```

### `TransformUpdated` (Actualización de Matriz de Mundo)
```json
{
  "type": "TransformUpdated",
  "data": {
    "id": "e7c2a123-28db-4cb8-8c10-53bc1bf5b8f3",
    "matrix": [
      1.0, 0.0, 0.0, 0.0,
      0.0, 0.0, 1.0, 0.0,
      0.0, -1.0, 0.0, 0.0,
      2.5, 0.0, -5.0, 1.0
    ]
  }
}
```
*`matrix` contiene 16 floats representando la matriz $T_{\text{Reactor}}$ en orden fila-major.*

### `MeshUploaded` (Subida de Datos de Geometría)
```json
{
  "type": "MeshUploaded",
  "data": {
    "id": "mesh_door_geo",
    "vertices": [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, ...],
    "normals": [0.0, 1.0, 0.0, 0.0, 1.0, 0.0, ...],
    "uvs": [0.0, 0.0, 1.0, 0.0, ...],
    "tangents": [1.0, 0.0, 0.0, 1.0, 0.0, 0.0, ...],
    "indices": [0, 1, 2, 2, 3, 0, ...]
  }
}
```

### `MaterialUpdated` (Propiedades de Material PBR)
```json
{
  "type": "MaterialUpdated",
  "data": {
    "id": "mat_door_gold",
    "albedo_color": [1.0, 0.84, 0.0, 1.0],
    "metallic": 1.0,
    "roughness": 0.15,
    "emissive": [0.0, 0.0, 0.0],
    "albedo_texture_id": "tex_gold_albedo_id",
    "normal_texture_id": "tex_gold_normal_id"
  }
}
```

### `LightUpdated` (Propiedades Lumínicas)
```json
{
  "type": "LightUpdated",
  "data": {
    "id": "light_sun_01",
    "kind": "Directional",
    "color": [1.0, 0.95, 0.85],
    "intensity": 5.0,
    "range": 100.0
  }
}
```

### `CameraUpdated` (Datos de Proyección de Cámara)
```json
{
  "type": "CameraUpdated",
  "data": {
    "id": "camera_main",
    "fov": 60.0,
    "near": 0.1,
    "far": 1000.0,
    "aspect": 1.7778
  }
}
```

### `TextureUploaded` (Subida de Imagen o Canal Cocinado)
```json
{
  "type": "TextureUploaded",
  "data": {
    "id": "tex_gold_albedo_id",
    "width": 1024,
    "height": 1024,
    "format": "RGBA8",
    "pixels": "iVBORw0KGgoAAAANSUhEUgAA..."
  }
}
```

---

## 📐 Sistema de Coordenadas y Conversión de Base
Para mapear de Blender (**Z-Up, Right-Handed**) a REACTOR/Vulkan (**Y-Up, Right-Handed**):

```math
M_{B\to R} = \begin{bmatrix}
1 & 0 & 0 & 0 \\
0 & 0 & 1 & 0 \\
0 & -1 & 0 & 0 \\
0 & 0 & 0 & 1
\end{bmatrix}
```

La traslación de una posición $(X_B, Y_B, Z_B)$ a REACTOR es:
$$X_R = X_B,\; Y_R = Z_B,\; Z_R = -Y_B$$

Y para las matrices de transformación del mundo $T$:
$$T_{\text{Reactor}} = M_{B\to R} \cdot T_{\text{Blender}} \cdot M_{B\to R}^{-1}$$

