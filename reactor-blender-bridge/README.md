# REACTOR ⇄ Blender Live Link — Bridge

Sincronización en tiempo real entre **Blender** (DCC) y **REACTOR** (runtime
Vulkan) sobre WebSocket localhost.

> **Estado actual: FASE 0 — Cimientos del protocolo** ✅
> Handshake, ping/pong, sync de transformaciones y materiales PBR.
> Las fases siguientes (mesh geometry, animaciones, luces, bidireccional…)
> se construyen sobre este transporte.

---

## 📁 Estructura

```text
reactor-blender-bridge/
├── README.md                       ← este archivo
├── proto/
│   └── messages.md                 ← especificación del protocolo (cross-lang)
│
├── rust/
│   └── reactor_bridge/             ← crate Rust (servidor + cliente WebSocket)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs              ← re-exports
│           ├── protocol.rs         ← Message enum + payload types
│           ├── server.rs           ← WebSocket server (tokio-tungstenite)
│           ├── client.rs           ← WebSocket client (para diagnóstico)
│           └── bin/
│               └── reactor_bridge_server.rs  ← binario standalone
│
├── python/                         ← Python package standalone
│   ├── __init__.py
│   ├── transport/                  ← capa de transporte (sin Blender deps)
│   │   ├── __init__.py
│   │   ├── protocol.py             ← tipos de mensaje, serialización
│   │   └── websocket_client.py     ← WebSocket client stdlib puro
│   ├── scripts/
│   │   └── empaquetar_addon.py     ← empaquetador del addon para Blender
│   └── tests/
│       ├── test_protocol.py        ← unit tests del protocolo
│       └── test_handshake.py       ← integración con servidor real
│
├── blender_addon/                  ← addon Blender (cliente)
│   ├── __init__.py
│   ├── manifest.toml
│   ├── prefs.py
│   ├── panel.py
│   ├── transport/                  ← re-export desde python.transport
│   │   └── __init__.py
│   ├── operators/
│   │   └── connect.py              ← WebSocket client + Blender operators
│   ├── handlers/
│   │   └── depsgraph.py            ← auto-sync de escena Blender
│   └── encoders/
│       └── transform.py            ← Z-Up → Y-Up matrix conversion
│
└── shaders/
    └── live_link/                  ← shaders PBR para Live Link
        ├── blender_live.vert
        ├── blender_live.frag       ← PBR completo (IBL + CSM + SSS + …)
        ├── shadow.vert
        └── shadow.frag
```

---

## 🚀 Quick Start — Probar el ping/pong (5 minutos)

### 1. Arrancar el servidor (Rust)

```powershell
cargo run --bin reactor-bridge-server
```

Verás:

```
[INFO] REACTOR Bridge server listening on 127.0.0.1:19840
[INFO] PROTOCOL_VERSION = 1
[INFO] Waiting for clients (Blender addon or standalone)…
```

### 2. Probar con el test Python standalone (sin Blender)

En otra terminal:

```powershell
pip install websockets
python reactor-blender-bridge/tests/ping_pong.py
```

Verás:

```
→ Hello sent (client=python_tester v=1)
← HelloAck (server=reactor_bridge v=1) capabilities=['ping']
ping seq=0 → pong rtt=0.8ms server_ts=...
ping seq=1 → pong rtt=0.6ms server_ts=...
...
```

### 3. Probar con Blender

1. Abrir Blender 4.2+.
2. `Edit → Preferences → Add-ons → Install…`
3. Seleccionar la carpeta `blender_addon/` (o un zip de la misma).
4. Activar **"REACTOR Live Link"**.
5. En el `N-panel` del 3D Viewport aparecerá una pestaña **"REACTOR"**.
6. Clic en **Connect** → debería mostrar "Connected · ping 0.8ms".

---

## 🧪 Mensajes implementados en FASE 0

| Mensaje      | Dirección        | Descripción                                       |
|--------------|------------------|---------------------------------------------------|
| `Hello`      | Cliente → Server | Apertura: versión protocolo, nombre cliente, caps |
| `HelloAck`   | Server → Cliente | Acepta o rechaza por versión incompatible        |
| `Ping`       | Cliente → Server | `{seq, ts_micros}` — para medir latencia          |
| `Pong`       | Server → Cliente | `{seq, ts_micros, server_ts_micros}`              |
| `Error`      | Bidireccional    | `{code, message}`                                 |
| `Goodbye`    | Bidireccional    | Cierre limpio con razón                           |

Las fases siguientes (1-8 del README raíz) añaden:
`EntityCreated`, `TransformUpdated`, `MeshUploaded`, `MaterialUpdated`,
`LightUpdated`, `CameraUpdated`, `TextureUploaded`, `AnimationKeyframe`,
`PickRequest/Response`, `ScenePushFull/Delta`.

---

## 🔧 Configuración

| Variable              | Default                | Descripción                          |
|-----------------------|------------------------|--------------------------------------|
| `REACTOR_BRIDGE_HOST` | `127.0.0.1`            | Sólo localhost por defecto (seguro)  |
| `REACTOR_BRIDGE_PORT` | `19840`                | Puerto WebSocket                     |
| `REACTOR_BRIDGE_LOG`  | `info`                 | Niveles: `trace/debug/info/warn/error` |

---

## 🎯 Siguientes fases (resumen)

Ver el [README raíz](../README.md#-reactor--blender-live-link--construir-el-juego-en-tiempo-real) para la hoja de ruta completa de 8 fases.

- **FASE 1** — Addon Blender mínimo: panel, operadores Connect/Disconnect, status bar.
- **FASE 2** — Bridge server completo: integración con `ReactorContext`.
- **FASE 3** — Sync de mesh + materiales PBR.
- **FASE 4** — Animaciones, armatures, shape keys.
- **FASE 5** — Luces, cámara y world (HDRI).
- **FASE 6** — Bidireccional: picking, gizmos, recorder.
- **FASE 7** — Asset cooker integrado.
- **FASE 8** — Live scripting, multi-cliente, AI assist.
