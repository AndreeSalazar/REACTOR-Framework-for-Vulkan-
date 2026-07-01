"""WebSocket client for REACTOR Bridge.

Pure Python WebSocket implementation using only stdlib.
No external dependencies required.

Designed for use inside Blender (no pip packages needed)
and standalone test scripts.
"""

import base64
import json
import queue
import random
import socket
import struct
import threading
from typing import Optional, Callable


class WebSocketError(Exception):
    """Raised on WebSocket connection or protocol errors."""


class WebSocketClient:
    """Raw WebSocket client (RFC 6455) for REACTOR Bridge."""

    def __init__(
        self,
        host: str = "127.0.0.1",
        port: int = 19840,
        on_message: Optional[Callable[[str], None]] = None,
        on_close: Optional[Callable[[], None]] = None,
        on_error: Optional[Callable[[Exception], None]] = None,
    ):
        self.host = host
        self.port = port
        self.on_message = on_message
        self.on_close = on_close
        self.on_error = on_error

        self.sock: Optional[socket.socket] = None
        self.connected = False
        self._read_thread: Optional[threading.Thread] = None
        self._write_thread: Optional[threading.Thread] = None
        self._send_queue: queue.Queue = queue.Queue()
        self._lock = threading.Lock()

    # ------------------------------------------------------------------
    # Lifecycle
    # ------------------------------------------------------------------

    def connect(self, timeout: float = 5.0) -> None:
        """Open TCP connection and perform WebSocket handshake."""
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.settimeout(timeout)
        self.sock.connect((self.host, self.port))
        self.sock.settimeout(None)

        self._handshake()

        self.connected = True
        self._read_thread = threading.Thread(target=self._read_loop, daemon=True)
        self._write_thread = threading.Thread(target=self._write_loop, daemon=True)
        self._read_thread.start()
        self._write_thread.start()

    def close(self) -> None:
        """Close the WebSocket connection gracefully."""
        with self._lock:
            if not self.connected:
                return
            self.connected = False
            try:
                self._send_frame(b"", opcode=8)  # Close frame
            except Exception:
                pass
            try:
                if self.sock:
                    self.sock.close()
            except Exception:
                pass

    def send(self, text: str) -> None:
        """Queue a text message for sending."""
        if self.connected:
            self._send_queue.put(text)

    # ------------------------------------------------------------------
    # Internal: WebSocket framing (RFC 6455)
    # ------------------------------------------------------------------

    def _handshake(self) -> None:
        """Perform the HTTP Upgrade handshake."""
        assert self.sock is not None
        key = base64.b64encode(
            bytes(random.getrandbits(8) for _ in range(16))
        ).decode()
        request = (
            f"GET / HTTP/1.1\r\n"
            f"Host: {self.host}:{self.port}\r\n"
            f"Upgrade: websocket\r\n"
            f"Connection: Upgrade\r\n"
            f"Sec-WebSocket-Key: {key}\r\n"
            f"Sec-WebSocket-Version: 13\r\n\r\n"
        )
        self.sock.sendall(request.encode())

        response = b""
        while b"\r\n\r\n" not in response:
            chunk = self.sock.recv(1024)
            if not chunk:
                break
            response += chunk

        if b"101 Switching Protocols" not in response:
            raise WebSocketError("Handshake failed: no 101 Switching Protocols")

    def _send_frame(self, data: bytes, opcode: int = 1) -> None:
        """Send a single WebSocket frame (masked, as required by clients)."""
        assert self.sock is not None
        header = bytearray([0x80 | opcode])
        length = len(data)

        if length < 126:
            header.append(0x80 | length)
        elif length < 65536:
            header.append(0x80 | 126)
            header.extend(struct.pack("!H", length))
        else:
            header.append(0x80 | 127)
            header.extend(struct.pack("!Q", length))

        mask = bytes(random.getrandbits(8) for _ in range(4))
        header.extend(mask)
        masked = bytearray(data[i] ^ mask[i % 4] for i in range(length))

        self.sock.sendall(bytes(header) + masked)

    def _read_loop(self) -> None:
        """Continuously read frames from the socket."""
        assert self.sock is not None
        while self.connected:
            try:
                header = self._recv_exactly(2)
                if not header:
                    break

                opcode = header[0] & 0x0F
                masked = (header[1] & 0x80) != 0
                length = header[1] & 0x7F

                if length == 126:
                    raw = self._recv_exactly(2)
                    length = struct.unpack("!H", raw)[0]
                elif length == 127:
                    raw = self._recv_exactly(8)
                    length = struct.unpack("!Q", raw)[0]

                mask = self._recv_exactly(4) if masked else None
                payload = self._recv_exactly(length) if length > 0 else b""

                if payload is None and length > 0:
                    break

                if mask and payload:
                    payload = bytes(payload[i] ^ mask[i % 4] for i in range(length))

                if opcode == 1:  # Text frame
                    if self.on_message:
                        self.on_message(payload.decode("utf-8"))
                elif opcode == 8:  # Close frame
                    break
            except Exception as e:
                if self.on_error:
                    self.on_error(e)
                break

        self.connected = False
        if self.on_close:
            self.on_close()

    def _write_loop(self) -> None:
        """Continuously drain the send queue to the socket."""
        while self.connected:
            try:
                msg = self._send_queue.get(timeout=0.1)
                with self._lock:
                    self._send_frame(msg.encode("utf-8"), opcode=1)
            except queue.Empty:
                continue
            except Exception:
                break

    def _recv_exactly(self, n: int) -> Optional[bytes]:
        """Read exactly n bytes from the socket."""
        assert self.sock is not None
        data = b""
        while len(data) < n:
            try:
                chunk = self.sock.recv(n - len(data))
                if not chunk:
                    return None
                data += chunk
            except Exception:
                return None
        return data
