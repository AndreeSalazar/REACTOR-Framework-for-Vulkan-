// =============================================================================
// Server — REACTOR Bridge WebSocket
// FASE 0: acepta conexiones, handshake, ping/pong, error/goodbye
// =============================================================================

use std::net::SocketAddr;

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::WebSocketStream;
use tracing::{debug, error, info, warn};

use crate::protocol::{
    codes, now_micros, Error as ErrorMsg, HelloAck, Message, Pong, PROTOCOL_VERSION,
    SERVER_CAPABILITIES, SERVER_NAME,
};

// -----------------------------------------------------------------------------
// Configuración
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct BridgeConfig {
    pub host: String,
    pub port: u16,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        let host = std::env::var("REACTOR_BRIDGE_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = std::env::var("REACTOR_BRIDGE_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(19840);
        Self { host, port }
    }
}

impl BridgeConfig {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

// -----------------------------------------------------------------------------
// Handle público — para usar desde el runtime REACTOR
// -----------------------------------------------------------------------------

pub struct BridgeHandle {
    shutdown_tx: Option<oneshot::Sender<()>>,
    join: Option<JoinHandle<()>>,
    pub addr: SocketAddr,
}

impl BridgeHandle {
    /// Apaga el servidor de forma limpia.
    pub async fn shutdown(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(j) = self.join.take() {
            let _ = j.await;
        }
    }
}

// -----------------------------------------------------------------------------
// API principal
// -----------------------------------------------------------------------------

/// Arranca el servidor WebSocket y devuelve un handle controlable.
pub async fn spawn(
    cfg: BridgeConfig,
    tx: Option<tokio::sync::mpsc::UnboundedSender<Message>>,
) -> std::io::Result<BridgeHandle> {
    let addr: SocketAddr = cfg
        .addr()
        .parse()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("{e}")))?;
    let listener = TcpListener::bind(addr).await?;
    let local = listener.local_addr()?;
    info!(
        ?local,
        version = PROTOCOL_VERSION,
        "REACTOR Bridge listening"
    );

    let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

    let join = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut shutdown_rx => {
                    info!("REACTOR Bridge shutdown requested");
                    break;
                }
                accept = listener.accept() => {
                    match accept {
                        Ok((stream, peer)) => {
                            info!(%peer, "client connected");
                            let tx_clone = tx.clone();
                            tokio::spawn(handle_client(stream, peer, tx_clone));
                        }
                        Err(e) => {
                            error!(error = %e, "accept failed");
                        }
                    }
                }
            }
        }
    });

    Ok(BridgeHandle {
        shutdown_tx: Some(shutdown_tx),
        join: Some(join),
        addr: local,
    })
}

/// Forma alternativa: arrancar el servidor y bloquear el thread actual hasta
/// Ctrl+C. Útil para el binario standalone.
pub async fn run_forever(cfg: BridgeConfig) -> std::io::Result<()> {
    let handle = spawn(cfg, None).await?;
    info!(addr = %handle.addr, "press Ctrl+C to shut down");
    tokio::signal::ctrl_c().await.ok();
    info!("Ctrl+C received, shutting down…");
    handle.shutdown().await;
    Ok(())
}

// -----------------------------------------------------------------------------
// Conexión por cliente
// -----------------------------------------------------------------------------

async fn handle_client(
    stream: TcpStream,
    peer: SocketAddr,
    tx: Option<tokio::sync::mpsc::UnboundedSender<Message>>,
) {
    let ws = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!(%peer, error = %e, "websocket handshake failed");
            return;
        }
    };
    debug!(%peer, "websocket handshake complete");

    if let Err(e) = run_session(ws, peer, tx).await {
        warn!(%peer, error = %e, "session ended with error");
    } else {
        info!(%peer, "session ended cleanly");
    }
}

#[derive(thiserror::Error, Debug)]
enum SessionError {
    #[error("websocket error: {0}")]
    Ws(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("client never sent Hello before closing")]
    NoHello,
}

async fn run_session(
    mut ws: WebSocketStream<TcpStream>,
    peer: SocketAddr,
    tx: Option<tokio::sync::mpsc::UnboundedSender<Message>>,
) -> Result<(), SessionError> {
    // 1) Esperar Hello antes que nada
    let mut handshaked = false;

    while let Some(raw) = ws.next().await {
        let raw = raw?;
        match raw {
            WsMessage::Text(text) => {
                let parsed = Message::from_json(&text);

                // Forward parsed message to channel
                if let Ok(ref msg) = parsed {
                    if let Some(ref sender) = tx {
                        let _ = sender.send(msg.clone());
                    }
                }

                let reply = match (parsed, handshaked) {
                    (Ok(Message::Hello(h)), false) => {
                        info!(%peer, version = h.version, client = %h.client,
                               caps = ?h.capabilities, "Hello received");
                        let accepted = h.version == PROTOCOL_VERSION;
                        if !accepted {
                            warn!(%peer, got = h.version, want = PROTOCOL_VERSION,
                                  "incompatible protocol version");
                        } else {
                            handshaked = true;
                        }
                        Some(Message::HelloAck(HelloAck {
                            version: PROTOCOL_VERSION,
                            server: SERVER_NAME.into(),
                            accepted,
                            capabilities: SERVER_CAPABILITIES
                                .iter()
                                .map(|s| s.to_string())
                                .collect(),
                            reason: if accepted {
                                None
                            } else {
                                Some(format!(
                                    "server requires PROTOCOL_VERSION={PROTOCOL_VERSION}, got {}",
                                    h.version
                                ))
                            },
                        }))
                    }

                    (Ok(_), false) => {
                        warn!(%peer, "message arrived before Hello");
                        Some(Message::Error(ErrorMsg {
                            code: codes::NOT_HANDSHAKED.into(),
                            message: "send Hello before any other message".into(),
                        }))
                    }

                    (Ok(Message::Ping(p)), true) => {
                        debug!(%peer, seq = p.seq, "Ping");
                        Some(Message::Pong(Pong {
                            seq: p.seq,
                            client_ts_micros: p.ts_micros,
                            server_ts_micros: now_micros(),
                        }))
                    }

                    (Ok(Message::TransformUpdated(t)), true) => {
                        debug!(%peer, entity = %t.id, "TransformUpdated received");
                        None
                    }

                    (Ok(Message::Goodbye(g)), _) => {
                        info!(%peer, reason = %g.reason, "Goodbye");
                        let _ = ws.send(WsMessage::Close(None)).await;
                        return Ok(());
                    }

                    (Ok(Message::Error(e)), _) => {
                        warn!(%peer, code = %e.code, msg = %e.message, "client-side error");
                        None
                    }

                    (Ok(other), true) => {
                        warn!(%peer, ?other, "unhandled message type for FASE 0");
                        Some(Message::Error(ErrorMsg {
                            code: codes::UNKNOWN_MESSAGE.into(),
                            message: format!(
                                "this FASE 0 server only handles Hello/Ping/Goodbye/TransformUpdated, got {:?}",
                                std::mem::discriminant(&other)
                            ),
                        }))
                    }

                    (Err(e), _) => {
                        warn!(%peer, error = %e, "malformed JSON");
                        Some(Message::Error(ErrorMsg {
                            code: codes::MALFORMED_PAYLOAD.into(),
                            message: format!("{e}"),
                        }))
                    }
                };

                if let Some(reply) = reply {
                    let json = match reply.to_json() {
                        Ok(j) => j,
                        Err(e) => {
                            error!(%peer, error = %e, "failed to serialize reply");
                            continue;
                        }
                    };
                    ws.send(WsMessage::Text(json)).await?;
                }
            }

            WsMessage::Binary(_) => {
                // Reservado para fases futuras (msgpack, mesh data).
                warn!(%peer, "binary frames not supported in FASE 0");
            }

            WsMessage::Ping(payload) => {
                // Pong WebSocket nativo (distinto de nuestro Ping aplicativo).
                ws.send(WsMessage::Pong(payload)).await?;
            }

            WsMessage::Pong(_) => { /* ignorar */ }

            WsMessage::Close(_) => {
                info!(%peer, "client closed");
                return Ok(());
            }

            WsMessage::Frame(_) => { /* ignorar low-level frames */ }
        }
    }

    if handshaked {
        Ok(())
    } else {
        Err(SessionError::NoHello)
    }
}
