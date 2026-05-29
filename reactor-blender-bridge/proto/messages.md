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

## 📈 Futuras adiciones (fases 1-8)

Documentadas en `[README raíz](../../README.md#-reactor--blender-live-link…)`.

Mensajes pendientes:
`EntityCreated`, `EntityRemoved`, `TransformUpdated`, `MeshUploaded`,
`MaterialUpdated`, `LightUpdated`, `CameraUpdated`, `TextureUploaded`,
`AnimationKeyframe`, `PickRequest`, `PickResponse`, `ScenePushFull`,
`ScenePushDelta`.
