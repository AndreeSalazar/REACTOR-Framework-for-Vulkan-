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
        # CASO A: Se actualizó un material directamente en Blender
        if isinstance(update.id, bpy.types.Material):
            mat = update.id
            color_flat = list(mat.diffuse_color)
            
            # Buscar qué objetos en la escena usan este material y enviar su color
            for scene_obj in scene.objects:
                if scene_obj.type == 'MESH' and len(scene_obj.data.materials) > 0:
                    if scene_obj.data.materials[0] == mat:
                        matrix_flat = blender_to_reactor_matrix(scene_obj.matrix_world)
                        msg = {
                            "type": "TransformUpdated",
                            "data": {
                                "id": scene_obj.name,
                                "matrix": matrix_flat,
                                "color": color_flat
                            }
                        }
                        try:
                            client.send(json.dumps(msg))
                            print(f"[REACTOR] 🎨 Material color actualizado para '{scene_obj.name}' → {color_flat[:3]}")
                        except Exception:
                            pass
            continue

        # CASO B: Se actualizó la transformación o geometría de un objeto
        obj = update.id
        # Only process objects (not meshes, scenes, etc.)
        if not isinstance(obj, bpy.types.Object):
            continue

        # Skip non-geometric objects
        if obj.type not in {'MESH', 'EMPTY', 'LIGHT', 'CAMERA', 'ARMATURE',
                            'CURVE', 'SURFACE', 'FONT', 'LATTICE'}:
            continue

        # Obtener el color del material si está disponible
        color_flat = None
        if obj.type == 'MESH' and len(obj.data.materials) > 0:
            mat = obj.data.materials[0]
            if mat is not None:
                color_flat = list(mat.diffuse_color)

        obj_name = obj.name
        current_matrix = tuple(tuple(row) for row in obj.matrix_world)
        
        # El estado incluye la matriz y el color para forzar envío si el color cambia
        current_state = (current_matrix, tuple(color_flat) if color_flat else None)

        if obj_name in _last_transforms and _last_transforms[obj_name] == current_state:
            continue

        _last_transforms[obj_name] = current_state

        # Convert and send
        matrix_flat = blender_to_reactor_matrix(obj.matrix_world)

        msg = {
            "type": "TransformUpdated",
            "data": {
                "id": obj_name,
                "matrix": matrix_flat
            }
        }
        if color_flat is not None:
            msg["data"]["color"] = color_flat

        try:
            client.send(json.dumps(msg))
            print(f"[REACTOR] → Enviado TransformUpdated para '{obj_name}'")
        except Exception as e:
            print(f"[REACTOR] ✗ Error al enviar TransformUpdated para '{obj_name}': {e}")


def sync_full_scene():
    """Sincroniza todos los objetos geométricos de la escena actual con REACTOR."""
    client = _get_client()
    if client is None or not client.connected:
        return

    # Import encoder lazily to avoid import errors outside Blender
    from ..encoders.transform import blender_to_reactor_matrix

    print("[REACTOR] Sincronizando escena completa con REACTOR...")
    count = 0
    # Iterar por todos los objetos de la escena
    for obj in bpy.context.scene.objects:
        # Saltar objetos no geométricos o que no nos interesen
        if obj.type not in {'MESH', 'EMPTY', 'LIGHT', 'CAMERA', 'ARMATURE',
                            'CURVE', 'SURFACE', 'FONT', 'LATTICE'}:
            continue

        obj_name = obj.name
        
        # Obtener el color del material
        color_flat = None
        if obj.type == 'MESH' and len(obj.data.materials) > 0:
            mat = obj.data.materials[0]
            if mat is not None:
                color_flat = list(mat.diffuse_color)

        current_matrix = tuple(tuple(row) for row in obj.matrix_world)
        _last_transforms[obj_name] = (current_matrix, tuple(color_flat) if color_flat else None)

        matrix_flat = blender_to_reactor_matrix(obj.matrix_world)

        msg = {
            "type": "TransformUpdated",
            "data": {
                "id": obj_name,
                "matrix": matrix_flat
            }
        }
        if color_flat is not None:
            msg["data"]["color"] = color_flat

        try:
            client.send(json.dumps(msg))
            count += 1
        except Exception as e:
            print(f"[REACTOR] Error enviando sync inicial para '{obj_name}': {e}")

    print(f"[REACTOR] Sincronizados {count} objetos iniciales con el motor.")


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
