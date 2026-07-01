"""Transport package for blender_addon.

Re-exports from the canonical python.transport module.
When bundled as a Blender extension, this file can be replaced
with a bundled copy of python/transport/ contents.
"""

import os
import sys

# Make the parent python/ package discoverable
_pkg_root = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "python"))
if _pkg_root not in sys.path:
    sys.path.insert(0, _pkg_root)

from transport.protocol import (
    PROTOCOL_VERSION,
    make_message,
    parse_message,
    MessageType,
    now_micros,
)
from transport.websocket_client import WebSocketClient, WebSocketError

__all__ = [
    "PROTOCOL_VERSION",
    "make_message",
    "parse_message",
    "MessageType",
    "now_micros",
    "WebSocketClient",
    "WebSocketError",
]
