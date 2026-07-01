"""Standalone test client for REACTOR Bridge.

Tests WebSocket connectivity, handshake, and ping/pong
without needing Blender.

Usage:
    python python/tests/test_handshake.py
"""

import json
import os
import sys
import time

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from transport.protocol import (
    PROTOCOL_VERSION,
    make_message,
    parse_message,
    MessageType,
)
from transport.websocket_client import WebSocketClient


def on_message(msg: str):
    msg_type, data = parse_message(msg)
    now = time.time()

    if msg_type == MessageType.HELLO_ACK:
        if data.get("accepted"):
            print(f"← HelloAck accepted (server={data.get('server')})")
        else:
            print(f"← HelloAck rejected: {data.get('reason')}")
    elif msg_type == MessageType.PONG:
        client_ts = data.get("client_ts_micros", 0) / 1_000_000
        rtt_ms = (now - client_ts) * 1000
        print(f"← Pong seq={data.get('seq')} rtt={rtt_ms:.1f}ms")
    elif msg_type == MessageType.ERROR:
        print(f"← Error: {data.get('code')} — {data.get('message')}")
    else:
        print(f"← {msg_type}: {data}")


def on_close():
    print("Connection closed")


def main():
    host = os.environ.get("REACTOR_BRIDGE_HOST", "127.0.0.1")
    port = int(os.environ.get("REACTOR_BRIDGE_PORT", "19840"))

    client = WebSocketClient(
        host=host,
        port=port,
        on_message=on_message,
        on_close=on_close,
    )

    print(f"Connecting to {host}:{port}...")
    client.connect()

    hello = make_message(MessageType.HELLO, {
        "version": PROTOCOL_VERSION,
        "client": "python_tester",
        "capabilities": ["ping"],
    })
    client.send(hello)
    print(f"→ Hello (v={PROTOCOL_VERSION})")

    seq = 0
    try:
        while client.connected:
            time.sleep(1.5)
            seq += 1
            ts = int(time.time() * 1_000_000)
            ping = make_message(MessageType.PING, {"seq": seq, "ts_micros": ts})
            client.send(ping)
            print(f"→ Ping seq={seq}")
    except KeyboardInterrupt:
        print("\nShutting down...")
        goodbye = make_message(MessageType.GOODBYE, {"reason": "tester exit"})
        client.send(goodbye)
        client.close()


if __name__ == "__main__":
    main()
