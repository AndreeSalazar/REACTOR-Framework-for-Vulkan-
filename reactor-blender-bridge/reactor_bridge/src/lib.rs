// =============================================================================
// REACTOR Bridge — Blender Live Link
// =============================================================================
//
// Transporte bidireccional REACTOR ⇄ Blender sobre WebSocket localhost.
//
// Módulos:
//   protocol   → tipos de mensaje (Message, Hello, Ping, TransformUpdated…)
//   server     → WebSocket server (acepta conexiones del addon Blender)
//   client     → WebSocket client (conecta a un servidor bridge)
//
// Uso (servidor):
// ```ignore
// let handle = reactor_bridge::server::spawn(cfg, tx).await?;
// // ... reactor.run(...) ...
// handle.shutdown().await;
// ```

pub mod client;
pub mod protocol;
pub mod server;

pub use client::{BridgeClient, ClientConfig};
pub use protocol::{
    Error as ErrorMsg, Goodbye, Hello, HelloAck, Message, Ping, Pong, TransformUpdated,
    PROTOCOL_VERSION,
};
pub use server::{BridgeConfig, BridgeHandle};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("serialization error: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("channel closed")]
    ChannelClosed,
}
