import bpy
import json
import os
import queue
import time

# Transport layer — use bundled or system python.transport
try:
    from ..transport import WebSocketClient, make_message, parse_message, MessageType, PROTOCOL_VERSION
except ImportError:
    from transport.websocket_client import WebSocketClient
    from transport.protocol import make_message, parse_message, MessageType, PROTOCOL_VERSION

# Global reference to the client instance
_client = None
_timer_handle = None
_msg_queue = queue.Queue()


def on_message_received(msg_str):
    try:
        _, data = parse_message(msg_str)
        _msg_queue.put(json.loads(msg_str))
    except Exception as e:
        print(f"[REACTOR] Error parsing message: {e}")

def on_connection_closed():
    _msg_queue.put({"type": "_InternalDisconnect"})

def on_connection_error(err):
    _msg_queue.put({"type": "_InternalError", "msg": str(err)})


def load_live_config():
    import os
    import json
    
    # Valores por defecto
    config = {
        "host": "127.0.0.1",
        "port": 19840,
        "auto_connect": True,
        "sync_transforms": True,
        "sync_cameras": True,
        "sync_lights": True,
        "log_level": "info"
    }
    
    # Buscar hacia arriba desde connect.py
    current_dir = os.path.dirname(os.path.abspath(__file__))
    for _ in range(5):
        config_path = os.path.join(current_dir, "reactor_live_config.json")
        if os.path.exists(config_path):
            try:
                with open(config_path, "r", encoding="utf-8") as f:
                    file_config = json.load(f)
                    config.update(file_config)
                print(f"[REACTOR] Cargada configuracion de {config_path}")
                break
            except Exception as e:
                print(f"[REACTOR] Error leyendo archivo de config JSON: {e}")
        current_dir = os.path.dirname(current_dir)
        
    return config


class REACTOR_OT_live_connect(bpy.types.Operator):
    bl_idname = "reactor.live_connect"
    bl_label = "Conectar a REACTOR"
    bl_description = "Establece la conexion WebSocket con REACTOR runtime"

    def execute(self, context):
        global _client, _timer_handle
        
        print("[REACTOR] ── Iniciando conexión Live Link ──")
        
        # Cargar configuración desde el JSON compartido
        config = load_live_config()
        host = config.get("host", "127.0.0.1")
        port = config.get("port", 19840)
        
        # Intentar fallback a preferencias del addon si no hay JSON
        try:
            addon_key = __package__.split('.')[0]
            prefs = context.preferences.addons[addon_key].preferences
            # Solo usar preferencias si load_live_config no encontró el JSON
            if not config.get("_found_file", False):
                host = prefs.host
                port = prefs.port
                print(f"[REACTOR] Usando preferencias del addon: {host}:{port}")
        except Exception:
            print(f"[REACTOR] Usando configuración por defecto: {host}:{port}")
        
        context.scene.reactor_status = "Conectando..."
        context.scene.reactor_latency = "Calculando..."
        
        print(f"[REACTOR] Conectando WebSocket a {host}:{port}...")
        
        try:
            _client = WebSocketClient(
                host=host,
                port=port,
                on_message=on_message_received,
                on_close=on_connection_closed,
                on_error=on_connection_error,
            )
            _client.connect()
            print(f"[REACTOR] ✓ WebSocket conectado a {host}:{port}")
            
            # Enviar Hello handshake
            hello = make_message(MessageType.HELLO, {
                "version": PROTOCOL_VERSION,
                "client": "blender_addon",
                "capabilities": ["ping", "scene_sync"],
            })
            _client.send(hello)
            print("[REACTOR] → Enviado Hello handshake")
            
            # Registrar timer de polling si no está registrado
            if not bpy.app.timers.is_registered(poll_reactor_queue):
                bpy.app.timers.register(poll_reactor_queue, first_interval=0.05)
                print("[REACTOR] ✓ Timer de polling registrado (50ms)")
                
            self.report({'INFO'}, f"Conectando a {host}:{port}...")
        except Exception as e:
            print(f"[REACTOR] ✗ Error de conexión: {e}")
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
            try:
                goodbye = make_message(MessageType.GOODBYE, {"reason": "User disconnected addon"})
                _client.send(goodbye)
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
        ping_msg = make_message(MessageType.PING, {
            "seq": _ping_seq,
            "ts_micros": int(now * 1_000_000),
        })
        _client.send(ping_msg)

    # Process all incoming messages
    while not _msg_queue.empty():
        try:
            msg = _msg_queue.get_nowait()
            msg_type = msg.get("type")
            data = msg.get("data", {})
            
            if msg_type == "HelloAck":
                if data.get("accepted"):
                    print("[REACTOR] ✓ HelloAck aceptado — ¡Conexión establecida!")
                    bpy.context.scene.reactor_status = "Conectado"
                    bpy.context.scene.reactor_connected = True
                    # Sincronizar toda la escena al conectar
                    try:
                        from ..handlers import depsgraph
                        depsgraph.sync_full_scene()
                    except Exception as e:
                        import traceback
                        print(f"[REACTOR] ✗ Error al sincronizar escena completa al conectar: {e}")
                        traceback.print_exc()
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


class REACTOR_OT_live_sync(bpy.types.Operator):
    bl_idname = "reactor.live_sync"
    bl_label = "Sincronizar Escena"
    bl_description = "Sincroniza todos los objetos geometricos actuales con REACTOR de forma manual"

    def execute(self, context):
        global _client
        if not _client or not _client.connected:
            self.report({'ERROR'}, "REACTOR no esta conectado.")
            return {'CANCELLED'}
        try:
            from ..handlers import depsgraph
            depsgraph.sync_full_scene()
            self.report({'INFO'}, "Escena sincronizada de forma manual con éxito.")
        except Exception as e:
            self.report({'ERROR'}, f"Fallo al sincronizar: {e}")
        return {'FINISHED'}


def register():
    bpy.utils.register_class(REACTOR_OT_live_connect)
    bpy.utils.register_class(REACTOR_OT_live_disconnect)
    bpy.utils.register_class(REACTOR_OT_live_sync)
    
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
    bpy.utils.unregister_class(REACTOR_OT_live_sync)
    
    del bpy.types.Scene.reactor_connected
    del bpy.types.Scene.reactor_status
    del bpy.types.Scene.reactor_latency

