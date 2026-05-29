import bpy
import socket
import threading
import json
import struct
import base64
import random
import queue
import time

# Global reference to the client instance
_client = None
_timer_handle = None
_msg_queue = queue.Queue()

class RawWebSocketClient:
    def __init__(self, host, port, on_message=None, on_close=None, on_error=None):
        self.host = host
        self.port = port
        self.on_message = on_message
        self.on_close = on_close
        self.on_error = on_error
        self.sock = None
        self.connected = False
        self.read_thread = None
        self.write_thread = None
        self.send_queue = queue.Queue()

    def connect(self):
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.settimeout(5.0)
        self.sock.connect((self.host, self.port))
        self.sock.settimeout(None)
        
        # Handshake
        key = base64.b64encode(bytes(random.getrandbits(8) for _ in range(16))).decode()
        handshake = (
            f"GET / HTTP/1.1\r\n"
            f"Host: {self.host}:{self.port}\r\n"
            f"Upgrade: websocket\r\n"
            f"Connection: Upgrade\r\n"
            f"Sec-WebSocket-Key: {key}\r\n"
            f"Sec-WebSocket-Version: 13\r\n\r\n"
        )
        self.sock.sendall(handshake.encode())
        
        # Read handshake response
        response = b""
        while b"\r\n\r\n" not in response:
            chunk = self.sock.recv(1024)
            if not chunk:
                break
            response += chunk
            
        if b"101 Switching Protocols" not in response:
            raise Exception("Handshake switching protocols failed")
            
        self.connected = True
        self.read_thread = threading.Thread(target=self._read_loop, daemon=True)
        self.write_thread = threading.Thread(target=self._write_loop, daemon=True)
        self.read_thread.start()
        self.write_thread.start()

    def send(self, text_msg):
        if self.connected:
            self.send_queue.put(text_msg)

    def close(self):
        self.connected = False
        if self.sock:
            try:
                self._send_frame(b"", opcode=8) # Close frame
                self.sock.close()
            except:
                pass

    def _send_frame(self, data, opcode=1):
        if not self.sock:
            return
        header = bytearray([0x80 | opcode])
        payload_len = len(data)
        
        if payload_len < 126:
            header.append(0x80 | payload_len)
        elif payload_len < 65536:
            header.append(0x80 | 126)
            header.extend(struct.pack("!H", payload_len))
        else:
            header.append(0x80 | 127)
            header.extend(struct.pack("!Q", payload_len))
            
        mask = bytes(random.getrandbits(8) for _ in range(4))
        header.extend(mask)
        masked_data = bytearray(data[i] ^ mask[i % 4] for i in range(payload_len))
        
        try:
            self.sock.sendall(header + masked_data)
        except:
            self.connected = False

    def _write_loop(self):
        while self.connected:
            try:
                msg = self.send_queue.get(timeout=0.1)
                self._send_frame(msg.encode("utf-8"), opcode=1)
            except queue.Empty:
                continue
            except:
                break

    def _read_loop(self):
        while self.connected:
            try:
                header = self._recv_exactly(2)
                if not header:
                    break
                opcode = header[0] & 0x0F
                masked = (header[1] & 0x80) != 0
                payload_len = header[1] & 0x7F
                
                if payload_len == 126:
                    len_bytes = self._recv_exactly(2)
                    payload_len = struct.unpack("!H", len_bytes)[0]
                elif payload_len == 127:
                    len_bytes = self._recv_exactly(8)
                    payload_len = struct.unpack("!Q", len_bytes)[0]
                    
                if masked:
                    mask = self._recv_exactly(4)
                
                payload = self._recv_exactly(payload_len)
                if not payload and payload_len > 0:
                    break
                    
                if masked:
                    payload = bytes(payload[i] ^ mask[i % 4] for i in range(payload_len))
                    
                if opcode == 1: # Text
                    if self.on_message:
                        self.on_message(payload.decode("utf-8"))
                elif opcode == 8: # Close
                    break
            except:
                break
        self.connected = False
        if self.on_close:
            self.on_close()

    def _recv_exactly(self, n):
        data = b""
        while len(data) < n:
            try:
                chunk = self.sock.recv(n - len(data))
                if not chunk:
                    return None
                data += chunk
            except:
                return None
        return data


def on_message_received(msg_str):
    try:
        data = json.loads(msg_str)
        _msg_queue.put(data)
    except Exception as e:
        print(f"[REACTOR] Error parsing message: {e}")

def on_connection_closed():
    _msg_queue.put({"type": "_InternalDisconnect"})

def on_connection_error(err):
    _msg_queue.put({"type": "_InternalError", "msg": str(err)})


class REACTOR_OT_live_connect(bpy.types.Operator):
    bl_idname = "reactor.live_connect"
    bl_label = "Conectar a REACTOR"
    bl_description = "Establece la conexion WebSocket con REACTOR runtime"

    def execute(self, context):
        global _client, _timer_handle
        
        # Get host and port from preferences
        prefs = context.preferences.addons[__package__.split('.')[0]].preferences
        host = prefs.host
        port = prefs.port
        
        context.scene.reactor_status = "Conectando..."
        context.scene.reactor_latency = "Calculando..."
        
        try:
            _client = RawWebSocketClient(
                host, port,
                on_message=on_message_received,
                on_close=on_connection_closed,
                on_error=on_connection_error
            )
            _client.connect()
            
            # Send Hello handshake
            hello = {
                "type": "Hello",
                "data": {
                    "version": 1,
                    "client": "blender_addon",
                    "capabilities": ["ping"]
                }
            }
            _client.send(json.dumps(hello))
            
            # Register polling timer if not registered
            if not bpy.app.timers.is_registered(poll_reactor_queue):
                bpy.app.timers.register(poll_reactor_queue, first_interval=0.05)
                
            self.report({'INFO'}, f"Conectando a {host}:{port}...")
        except Exception as e:
            self.report({'ERROR'}, f"Fallo al conectar: {e}")
            context.scene.reactor_status = "Desconectado"
            context.scene.reactor_latency = "N/A"
            context.scene.reactor_connected = False
            _client = None
            
        return {'FINISHED'}


class REACTOR_OT_live_disconnect(bpy.types.Operator):
    bl_idname = "reactor.live_disconnect"
    bl_label = "Desconectar"
    bl_description = "Cierra la conexion activa con el runtime de REACTOR"

    def execute(self, context):
        global _client
        if _client:
            # Send Goodbye
            goodbye = {
                "type": "Goodbye",
                "data": { "reason": "User disconnected addon" }
            }
            try:
                _client.send(json.dumps(goodbye))
            except:
                pass
            _client.close()
            _client = None
            
        context.scene.reactor_status = "Desconectado"
        context.scene.reactor_latency = "N/A"
        context.scene.reactor_connected = False
        self.report({'INFO'}, "Desconectado de REACTOR.")
        return {'FINISHED'}


_last_ping_time = 0
_ping_seq = 0

def poll_reactor_queue():
    global _client, _last_ping_time, _ping_seq
    
    if not _client or not _client.connected:
        # Update scene properties
        for window in bpy.context.window_manager.windows:
            for area in window.screen.areas:
                if area.type == 'VIEW_3D':
                    area.tag_redraw()
        return None # Stop timer
        
    now = time.time()
    
    # Send ping every 1.5 seconds
    if now - _last_ping_time > 1.5:
        _last_ping_time = now
        _ping_seq += 1
        ping_msg = {
            "type": "Ping",
            "data": {
                "seq": _ping_seq,
                "ts_micros": int(now * 1000000)
            }
        }
        _client.send(json.dumps(ping_msg))

    # Process all incoming messages
    while not _msg_queue.empty():
        try:
            msg = _msg_queue.get_nowait()
            msg_type = msg.get("type")
            data = msg.get("data", {})
            
            if msg_type == "HelloAck":
                if data.get("accepted"):
                    bpy.context.scene.reactor_status = "Conectado"
                    bpy.context.scene.reactor_connected = True
                else:
                    bpy.context.scene.reactor_status = f"Rechazado: {data.get('reason')}"
                    bpy.context.scene.reactor_connected = False
                    if _client:
                        _client.close()
                        _client = None
            elif msg_type == "Pong":
                # Compute latency
                client_ts = data.get("client_ts_micros", 0) / 1000000.0
                rtt = (time.time() - client_ts) * 1000.0
                bpy.context.scene.reactor_latency = f"{rtt:.1f} ms"
            elif msg_type == "_InternalDisconnect":
                bpy.context.scene.reactor_status = "Desconectado"
                bpy.context.scene.reactor_latency = "N/A"
                bpy.context.scene.reactor_connected = False
                _client = None
            elif msg_type == "_InternalError":
                bpy.context.scene.reactor_status = f"Error: {msg.get('msg')}"
                bpy.context.scene.reactor_latency = "N/A"
                bpy.context.scene.reactor_connected = False
                _client = None
        except queue.Empty:
            break
            
    # Redraw UI areas to show changes
    for window in bpy.context.window_manager.windows:
        for area in window.screen.areas:
            if area.type == 'VIEW_3D':
                area.tag_redraw()
                
    return 0.05 # Run again in 50ms


def register():
    bpy.utils.register_class(REACTOR_OT_live_connect)
    bpy.utils.register_class(REACTOR_OT_live_disconnect)
    
    # Register scene properties
    bpy.types.Scene.reactor_connected = bpy.props.BoolProperty(name="Connected", default=False)
    bpy.types.Scene.reactor_status = bpy.props.StringProperty(name="Status", default="Desconectado")
    bpy.types.Scene.reactor_latency = bpy.props.StringProperty(name="Latency", default="N/A")

def unregister():
    global _client
    if _client:
        _client.close()
        _client = None
        
    bpy.utils.unregister_class(REACTOR_OT_live_connect)
    bpy.utils.unregister_class(REACTOR_OT_live_disconnect)
    
    del bpy.types.Scene.reactor_connected
    del bpy.types.Scene.reactor_status
    del bpy.types.Scene.reactor_latency
