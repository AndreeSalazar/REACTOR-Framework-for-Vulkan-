// =============================================================================
// Protocol — REACTOR Bridge (PROTOCOL_VERSION = 1)
// =============================================================================
// Mensajes JSON internally-tagged compartidos entre el servidor Rust y los
// clientes (Blender addon Python, tester standalone, futuro web preview).
//
// Spec completa: ../../proto/messages.md
// =============================================================================

use serde::{Deserialize, Serialize};

/// Versión del protocolo. Incrementar en cualquier cambio breaking.
pub const PROTOCOL_VERSION: u32 = 1;

/// Nombre del servidor (enviado en HelloAck).
pub const SERVER_NAME: &str = "reactor_bridge";

/// Capabilities que el servidor de FASE 0 soporta.
pub const SERVER_CAPABILITIES: &[&str] = &["ping"];

// -----------------------------------------------------------------------------
// Sobre principal
// -----------------------------------------------------------------------------

/// Mensaje top-level — serializa como `{"type": "Foo", "data": {...}}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Message {
    Hello(Hello),
    HelloAck(HelloAck),
    Ping(Ping),
    Pong(Pong),
    Error(ErrorPayload),
    Goodbye(Goodbye),
}

impl Message {
    /// Serializa a JSON UTF-8 listo para enviar por el WebSocket.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Parsea desde un string JSON recibido del WebSocket.
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

// -----------------------------------------------------------------------------
// Hello / HelloAck — handshake
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hello {
    pub version: u32,
    pub client: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloAck {
    pub version: u32,
    pub server: String,
    pub accepted: bool,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub reason: Option<String>,
}

// -----------------------------------------------------------------------------
// Ping / Pong — latencia
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ping {
    pub seq: u32,
    pub ts_micros: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pong {
    pub seq: u32,
    pub client_ts_micros: i64,
    pub server_ts_micros: i64,
}

// -----------------------------------------------------------------------------
// Error / Goodbye
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goodbye {
    #[serde(default)]
    pub reason: String,
}

// -----------------------------------------------------------------------------
// Re-exports cómodos
// -----------------------------------------------------------------------------

pub use ErrorPayload as Error;

// -----------------------------------------------------------------------------
// Códigos de error estándar (ver proto/messages.md)
// -----------------------------------------------------------------------------

pub mod codes {
    pub const INCOMPATIBLE_VERSION: &str = "INCOMPATIBLE_VERSION";
    pub const UNKNOWN_MESSAGE: &str = "UNKNOWN_MESSAGE";
    pub const MALFORMED_PAYLOAD: &str = "MALFORMED_PAYLOAD";
    pub const NOT_HANDSHAKED: &str = "NOT_HANDSHAKED";
    pub const INTERNAL: &str = "INTERNAL";
}

// -----------------------------------------------------------------------------
// Helper de timestamps (microsegundos epoch)
// -----------------------------------------------------------------------------

/// Devuelve el timestamp actual en microsegundos desde UNIX_EPOCH.
pub fn now_micros() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as i64)
        .unwrap_or(0)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_roundtrip() {
        let m = Message::Hello(Hello {
            version: 1,
            client: "blender_addon".into(),
            capabilities: vec!["scene_push".into()],
        });
        let json = m.to_json().unwrap();
        let back = Message::from_json(&json).unwrap();
        match back {
            Message::Hello(h) => {
                assert_eq!(h.version, 1);
                assert_eq!(h.client, "blender_addon");
                assert_eq!(h.capabilities, vec!["scene_push".to_string()]);
            }
            _ => panic!("expected Hello"),
        }
    }

    #[test]
    fn pong_roundtrip() {
        let m = Message::Pong(Pong {
            seq: 7,
            client_ts_micros: 1000,
            server_ts_micros: 1500,
        });
        let json = m.to_json().unwrap();
        assert!(json.contains("\"type\":\"Pong\""));
        let back = Message::from_json(&json).unwrap();
        match back {
            Message::Pong(p) => {
                assert_eq!(p.seq, 7);
                assert_eq!(p.client_ts_micros, 1000);
                assert_eq!(p.server_ts_micros, 1500);
            }
            _ => panic!("expected Pong"),
        }
    }

    #[test]
    fn unknown_type_fails_cleanly() {
        let json = r#"{"type":"NotARealMessage","data":{}}"#;
        assert!(Message::from_json(json).is_err());
    }

    #[test]
    fn version_constant_is_one() {
        assert_eq!(PROTOCOL_VERSION, 1);
    }
}
