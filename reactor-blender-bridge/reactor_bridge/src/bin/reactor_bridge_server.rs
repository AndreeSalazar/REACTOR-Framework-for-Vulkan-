// =============================================================================
// reactor-bridge-server — binario standalone para FASE 0
// =============================================================================
//
// Arranca el WebSocket en 127.0.0.1:19840 y espera clientes (Blender addon
// o el test standalone `tests/ping_pong.py`).
//
// Uso:
//   cargo run --bin reactor-bridge-server
//   REACTOR_BRIDGE_LOG=debug cargo run --bin reactor-bridge-server
//   REACTOR_BRIDGE_PORT=20000 cargo run --bin reactor-bridge-server
// =============================================================================

use reactor_bridge::server::{run_forever, BridgeConfig};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Logging configurable por env REACTOR_BRIDGE_LOG (info por defecto).
    let filter = EnvFilter::try_from_env("REACTOR_BRIDGE_LOG")
        .unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_level(true)
        .compact()
        .init();

    let cfg = BridgeConfig::default();
    tracing::info!(
        addr = %cfg.addr(),
        version = reactor_bridge::PROTOCOL_VERSION,
        "REACTOR Bridge server starting"
    );
    tracing::info!("waiting for clients (Blender addon, python tester, …)");

    run_forever(cfg).await
}
