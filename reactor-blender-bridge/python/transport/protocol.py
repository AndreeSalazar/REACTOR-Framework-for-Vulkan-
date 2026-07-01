"""Protocol definitions for REACTOR Bridge (PROTOCOL_VERSION = 1).

Mirrors the Rust `reactor_bridge::protocol` types for use from
Python clients (Blender addon, test harness, CLI tools).

Spec: ../../proto/messages.md
"""

import json
import time
from dataclasses import dataclass, field, asdict
from typing import Optional

PROTOCOL_VERSION: int = 1

SERVER_NAME: str = "reactor_bridge"
SERVER_CAPABILITIES: list[str] = field(default_factory=lambda: ["ping"])


class MessageType:
    """Canonical message type strings matching Rust serde tags."""

    HELLO = "Hello"
    HELLO_ACK = "HelloAck"
    PING = "Ping"
    PONG = "Pong"
    ERROR = "Error"
    GOODBYE = "Goodbye"
    TRANSFORM_UPDATED = "TransformUpdated"
    ENTITY_CREATED = "EntityCreated"
    ENTITY_REMOVED = "EntityRemoved"
    MESH_UPLOADED = "MeshUploaded"
    MATERIAL_UPDATED = "MaterialUpdated"
    LIGHT_UPDATED = "LightUpdated"
    CAMERA_UPDATED = "CameraUpdated"
    TEXTURE_UPLOADED = "TextureUploaded"


# ---------------------------------------------------------------------------
# Payload dataclasses (mirror Rust structs)
# ---------------------------------------------------------------------------


@dataclass
class Hello:
    version: int = PROTOCOL_VERSION
    client: str = "blender_addon"
    capabilities: list[str] = field(default_factory=lambda: ["ping", "scene_sync"])


@dataclass
class HelloAck:
    version: int = PROTOCOL_VERSION
    server: str = SERVER_NAME
    accepted: bool = True
    capabilities: list[str] = field(default_factory=lambda: ["ping"])
    reason: Optional[str] = None


@dataclass
class Ping:
    seq: int = 0
    ts_micros: int = 0


@dataclass
class Pong:
    seq: int = 0
    client_ts_micros: int = 0
    server_ts_micros: int = 0


@dataclass
class Error:
    code: str = "INTERNAL"
    message: str = ""


@dataclass
class Goodbye:
    reason: str = ""


@dataclass
class TransformUpdated:
    id: str = ""
    matrix: list[float] = field(default_factory=lambda: [0.0] * 16)
    color: Optional[list[float]] = None
    metallic: Optional[float] = None
    roughness: Optional[float] = None
    albedo_path: Optional[str] = None
    normal_path: Optional[str] = None
    metallic_path: Optional[str] = None
    roughness_path: Optional[str] = None
    emission_color: Optional[list[float]] = None
    emission_strength: Optional[float] = None


# ---------------------------------------------------------------------------
# Error codes (matching Rust reactor_bridge::protocol::codes)
# ---------------------------------------------------------------------------

CODES = {
    "INCOMPATIBLE_VERSION": "INCOMPATIBLE_VERSION",
    "UNKNOWN_MESSAGE": "UNKNOWN_MESSAGE",
    "MALFORMED_PAYLOAD": "MALFORMED_PAYLOAD",
    "NOT_HANDSHAKED": "NOT_HANDSHAKED",
    "INTERNAL": "INTERNAL",
}


# ---------------------------------------------------------------------------
# Serialization helpers
# ---------------------------------------------------------------------------


_PAYLOAD_MAP = {
    MessageType.HELLO: Hello,
    MessageType.HELLO_ACK: HelloAck,
    MessageType.PING: Ping,
    MessageType.PONG: Pong,
    MessageType.ERROR: Error,
    MessageType.GOODBYE: Goodbye,
    MessageType.TRANSFORM_UPDATED: TransformUpdated,
}


def make_message(msg_type: str, data: dict) -> str:
    """Build a JSON wire message: {"type": msg_type, "data": data}."""
    return json.dumps({"type": msg_type, "data": data}, ensure_ascii=False)


def parse_message(wire: str) -> tuple[str, dict]:
    """Parse a JSON wire message into (type, data_dict).

    Raises ValueError on malformed input.
    """
    try:
        obj = json.loads(wire)
    except json.JSONDecodeError as e:
        raise ValueError(f"JSON parse error: {e}") from e

    if not isinstance(obj, dict) or "type" not in obj:
        raise ValueError("Message missing 'type' field")

    msg_type = obj["type"]
    data = obj.get("data", {})
    if not isinstance(data, dict):
        raise ValueError("Message 'data' field must be a dict")

    return msg_type, data


def now_micros() -> int:
    """Current time in microseconds since UNIX_EPOCH."""
    return int(time.time() * 1_000_000)
