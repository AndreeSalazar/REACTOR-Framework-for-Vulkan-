// =============================================================================
// REACTOR Bridge — Blender Live Link
// FASE 0 — Cimientos del protocolo (handshake + ping/pong + errores)
// =============================================================================
//
// Este crate expone el `protocol` y `server` para que tanto el binario
// standalone (`reactor-bridge-server`) como una futura integración dentro
// del runtime de REACTOR (plugin) puedan reutilizarlo.
//
// Uso desde el runtime REACTOR (futuras fases):
//
// ```ignore
// let handle = reactor_bridge::server::spawn(BridgeConfig::default()).await?;
// // ... reactor.run(...) ...
// handle.shutdown().await;
// ```

pub mod protocol;
pub mod server;

pub use protocol::{
    Error as ErrorMsg, Goodbye, Hello, HelloAck, Message, Ping, Pong, TransformUpdated,
    PROTOCOL_VERSION,
};
pub use server::{BridgeConfig, BridgeHandle};
