/// WebSocket client for connecting to a REACTOR Bridge server.
///
/// Useful for testing, diagnostics, or when REACTOR needs to connect
/// to an external bridge (e.g., multi-engine setups).
///
/// ```ignore
/// use reactor_bridge::client::{BridgeClient, ClientConfig};
///
/// let (client, mut rx) = BridgeClient::connect(ClientConfig::default()).await?;
/// client.send(Message::Hello(Hello { .. })).await?;
/// while let Some(msg) = rx.recv().await {
///     // process msg
/// }
/// ```

use std::net::SocketAddr;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tracing::{debug, error, info};

use crate::protocol::Message;

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ClientConfig {
    fn default() -> Self {
        let host =
            std::env::var("REACTOR_BRIDGE_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = std::env::var("REACTOR_BRIDGE_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(19840);
        Self { host, port }
    }
}

impl ClientConfig {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

pub struct BridgeClient {
    write: mpsc::UnboundedSender<String>,
    #[allow(dead_code)]
    join: tokio::task::JoinHandle<()>,
}

impl BridgeClient {
    pub async fn connect(
        cfg: ClientConfig,
    ) -> Result<(Self, mpsc::UnboundedReceiver<Message>), Box<dyn std::error::Error>> {
        let addr: SocketAddr = cfg.addr().parse()?;
        let stream = TcpStream::connect(addr).await?;
        let ws: WebSocketStream<TcpStream> =
            tokio_tungstenite::client_async(format!("ws://{}/", cfg.addr()), stream)
                .await?
                .0;

        info!("connected to REACTOR Bridge at {}", cfg.addr());

        let (mut ws_tx, mut ws_rx) = ws.split();
        let (write_tx, mut write_rx) = mpsc::unbounded_channel::<String>();
        let (read_tx, read_rx) = mpsc::unbounded_channel::<Message>();

        let join = tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Outbound: drain write queue → WebSocket
                    Some(text) = write_rx.recv() => {
                        if let Err(e) = ws_tx.send(WsMessage::Text(text)).await {
                            error!("send error: {e}");
                            break;
                        }
                    }
                    // Inbound: WebSocket → read channel
                    Some(Ok(msg)) = ws_rx.next() => {
                        match msg {
                            WsMessage::Text(text) => {
                                match Message::from_json(&text) {
                                    Ok(parsed) => {
                                        let _ = read_tx.send(parsed);
                                    }
                                    Err(e) => {
                                        debug!("failed to parse message: {e}");
                                    }
                                }
                            }
                            WsMessage::Close(_) => break,
                            _ => {}
                        }
                    }
                    else => break,
                }
            }
            info!("disconnected from REACTOR Bridge");
        });

        Ok((Self { write: write_tx, join }, read_rx))
    }

    pub fn send(&self, msg: &Message) -> Result<(), crate::Error> {
        let json = msg.to_json().map_err(crate::Error::Serialize)?;
        self.write.send(json).map_err(|_| crate::Error::ChannelClosed)?;
        Ok(())
    }
}
