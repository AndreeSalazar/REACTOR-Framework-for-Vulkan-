"""Handler de depsgraph — sincronización automática Blender → REACTOR.

Registra un callback en `bpy.app.handlers.depsgraph_update_post` que se
ejecuta cada vez que Blender actualiza su grafo de dependencias (es decir,
cada vez que el usuario mueve, escala, rota, o modifica CUALQUIER objeto).

El handler detecta qué objetos cambiaron, convierte sus matrices de
transformación del mundo de Z-Up a Y-Up, y envía mensajes
`TransformUpdated` al servidor REACTOR via el WebSocket activo.
"""

import bpy
import json

# Avoid circular import issues — we access the client lazily
_last_transforms = {}


def _get_client():
    """Obtiene la instancia global del WebSocket client."""
    from ..operators import connect
    return connect._client


def _on_depsgraph_update(scene, depsgraph):
    """Callback ejecutado tras cada actualización del depsgraph."""
    client = _get_client()
    if client is None or not client.connected:
        return

    # Import encoder lazily to avoid import errors outside Blender
    from ..encoders.transform import blender_to_reactor_matrix

    # Iterate over updates
    for update in depsgraph.updates:
        obj = update.id
        # Only process objects (not materials, meshes, etc.)
        if not isinstance(obj, bpy.types.Object):
            continue

        # Skip non-geometric objects
        if obj.type not in {'MESH', 'EMPTY', 'LIGHT', 'CAMERA', 'ARMATURE',
                            'CURVE', 'SURFACE', 'FONT', 'LATTICE'}:
            continue

        # Check if transform actually changed (avoid redundant sends)
        obj_name = obj.name
        current_matrix = tuple(tuple(row) for row in obj.matrix_world)

        if obj_name in _last_transforms and _last_transforms[obj_name] == current_matrix:
            continue

        _last_transforms[obj_name] = current_matrix

        # Convert and send
        matrix_flat = blender_to_reactor_matrix(obj.matrix_world)

        msg = {
            "type": "TransformUpdated",
            "data": {
                "id": obj_name,
                "matrix": matrix_flat
            }
        }

        try:
            client.send(json.dumps(msg))
        except Exception as e:
            print(f"[REACTOR] Error sending TransformUpdated for '{obj_name}': {e}")


def register():
    """Registra el handler de depsgraph en Blender."""
    bpy.app.handlers.depsgraph_update_post.append(_on_depsgraph_update)
    print("[REACTOR] depsgraph handler registered")


def unregister():
    """Desregistra el handler de depsgraph."""
    if _on_depsgraph_update in bpy.app.handlers.depsgraph_update_post:
        bpy.app.handlers.depsgraph_update_post.remove(_on_depsgraph_update)
    _last_transforms.clear()
    print("[REACTOR] depsgraph handler unregistered")
