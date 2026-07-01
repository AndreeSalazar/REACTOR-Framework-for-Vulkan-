"""Unit tests for REACTOR Bridge Python transport layer."""

import json
import os
import sys
import time
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from transport.protocol import (
    PROTOCOL_VERSION,
    make_message,
    parse_message,
    MessageType,
    now_micros,
)
from transport.websocket_client import WebSocketClient, WebSocketError


class TestProtocol(unittest.TestCase):
    def test_protocol_version(self):
        self.assertEqual(PROTOCOL_VERSION, 1)

    def test_make_hello(self):
        wire = make_message(MessageType.HELLO, {
            "version": 1,
            "client": "test",
            "capabilities": [],
        })
        obj = json.loads(wire)
        self.assertEqual(obj["type"], "Hello")
        self.assertEqual(obj["data"]["version"], 1)
        self.assertEqual(obj["data"]["client"], "test")

    def test_parse_hello_ack(self):
        wire = json.dumps({
            "type": "HelloAck",
            "data": {"version": 1, "server": "reactor_bridge", "accepted": True, "capabilities": ["ping"]},
        })
        msg_type, data = parse_message(wire)
        self.assertEqual(msg_type, "HelloAck")
        self.assertTrue(data["accepted"])

    def test_parse_invalid_raises(self):
        with self.assertRaises(ValueError):
            parse_message("not json")
        with self.assertRaises(ValueError):
            parse_message('{"no_type": true}')
        with self.assertRaises(ValueError):
            parse_message('{"type": "Hello", "data": "not_dict"}')

    def test_make_and_parse_ping(self):
        ts = now_micros()
        wire = make_message(MessageType.PING, {"seq": 42, "ts_micros": ts})
        msg_type, data = parse_message(wire)
        self.assertEqual(msg_type, "Ping")
        self.assertEqual(data["seq"], 42)
        self.assertEqual(data["ts_micros"], ts)

    def test_make_and_parse_transform(self):
        matrix = [float(i) for i in range(16)]
        wire = make_message(MessageType.TRANSFORM_UPDATED, {
            "id": "Cube.001",
            "matrix": matrix,
            "color": [1.0, 0.5, 0.0, 1.0],
            "metallic": 0.8,
            "roughness": 0.2,
        })
        msg_type, data = parse_message(wire)
        self.assertEqual(msg_type, "TransformUpdated")
        self.assertEqual(data["id"], "Cube.001")
        self.assertEqual(data["metallic"], 0.8)

    def test_now_micros_type(self):
        ts = now_micros()
        self.assertIsInstance(ts, int)
        self.assertGreater(ts, 1_700_000_000_000_000)  # sanity check


if __name__ == "__main__":
    unittest.main()
