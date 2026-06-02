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


def get_texture_path(input_socket):
    """Obtiene la ruta absoluta de una textura conectada a un socket."""
    if input_socket and input_socket.is_linked:
        link = input_socket.links[0]
        from_node = link.from_node
        if from_node.type == 'TEX_IMAGE' and from_node.image:
            return bpy.path.abspath(from_node.image.filepath)
    return None


def get_normal_texture_path(node):
    """Obtiene la ruta absoluta del normal map conectado al nodo Principled BSDF."""
    normal_input = node.inputs.get('Normal')
    if normal_input and normal_input.is_linked:
        normal_node = normal_input.links[0].from_node
        if normal_node.type == 'NORMAL_MAP':
            color_input = normal_node.inputs.get('Color')
            if color_input and color_input.is_linked:
                tex_node = color_input.links[0].from_node
                if tex_node.type == 'TEX_IMAGE' and tex_node.image:
                    return bpy.path.abspath(tex_node.image.filepath)
    return None


def get_material_properties(mat):
    """Extrae las propiedades PBR principales de un material de Blender."""
    props = {
        "color": list(mat.diffuse_color),
        "metallic": 0.0,
        "roughness": 0.5,
        "albedo_path": None,
        "normal_path": None,
        "emission_color": [0.0, 0.0, 0.0],
        "emission_strength": 0.0,
    }
    
    if mat.use_nodes and mat.node_tree:
        node = next((n for n in mat.node_tree.nodes if n.type == 'BSDF_PRINCIPLED'), None)
        if node:
            # Color base
            color_input = node.inputs.get('Base Color')
            if color_input:
                if color_input.is_linked:
                    props["albedo_path"] = get_texture_path(color_input)
                else:
                    props["color"] = list(color_input.default_value)
            
            # Metalicidad
            metallic_input = node.inputs.get('Metallic')
            if metallic_input and not metallic_input.is_linked:
                props["metallic"] = metallic_input.default_value
            
            # Rugosidad
            roughness_input = node.inputs.get('Roughness')
            if roughness_input and not roughness_input.is_linked:
                props["roughness"] = roughness_input.default_value
            
            # Mapa de normales
            props["normal_path"] = get_normal_texture_path(node)
            
            # Emisión
            emission_input = node.inputs.get('Emission Color')
            if emission_input and not emission_input.is_linked:
                props["emission_color"] = list(emission_input.default_value)[:3]
            
            emission_str_input = node.inputs.get('Emission Strength')
            if emission_str_input and not emission_str_input.is_linked:
                props["emission_strength"] = emission_str_input.default_value
            
    return props


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
            props = get_material_properties(mat)
            
            # Buscar qué objetos en la escena usan este material y enviar su actualización
            for scene_obj in scene.objects:
                if scene_obj.type == 'MESH' and len(scene_obj.data.materials) > 0:
                    if scene_obj.data.materials[0] == mat:
                        matrix_flat = blender_to_reactor_matrix(scene_obj.matrix_world)
                        msg = {
                            "type": "TransformUpdated",
                            "data": {
                                "id": scene_obj.name,
                                "matrix": matrix_flat,
                                "color": props["color"],
                                "metallic": props["metallic"],
                                "roughness": props["roughness"],
                                "albedo_path": props["albedo_path"],
                                "normal_path": props["normal_path"],
                                "emission_color": props["emission_color"],
                                "emission_strength": props["emission_strength"],
                            }
                        }
                        try:
                            client.send(json.dumps(msg))
                            print(f"[REACTOR] 🎨 Material actualizado para '{scene_obj.name}': color={props['color'][:3]} metallic={props['metallic']} roughness={props['roughness']}")
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

        # Obtener las propiedades del material si es una malla
        props = None
        if obj.type == 'MESH' and len(obj.data.materials) > 0:
            mat = obj.data.materials[0]
            if mat is not None:
                props = get_material_properties(mat)

        obj_name = obj.name
        current_matrix = tuple(tuple(row) for row in obj.matrix_world)
        
        # El estado incluye la matriz y todas las propiedades del material para detectar cambios
        props_tuple = (
            tuple(props["color"]),
            props["metallic"],
            props["roughness"],
            props["albedo_path"],
            props["normal_path"],
            tuple(props["emission_color"]),
            props["emission_strength"]
        ) if props else None
        
        current_state = (current_matrix, props_tuple)

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
        if props:
            msg["data"].update({
                "color": props["color"],
                "metallic": props["metallic"],
                "roughness": props["roughness"],
                "albedo_path": props["albedo_path"],
                "normal_path": props["normal_path"],
                "emission_color": props["emission_color"],
                "emission_strength": props["emission_strength"],
            })

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
        
        props = None
        if obj.type == 'MESH' and len(obj.data.materials) > 0:
            mat = obj.data.materials[0]
            if mat is not None:
                props = get_material_properties(mat)

        current_matrix = tuple(tuple(row) for row in obj.matrix_world)
        
        props_tuple = (
            tuple(props["color"]),
            props["metallic"],
            props["roughness"],
            props["albedo_path"],
            props["normal_path"],
            tuple(props["emission_color"]),
            props["emission_strength"]
        ) if props else None
        
        _last_transforms[obj_name] = (current_matrix, props_tuple)

        matrix_flat = blender_to_reactor_matrix(obj.matrix_world)

        msg = {
            "type": "TransformUpdated",
            "data": {
                "id": obj_name,
                "matrix": matrix_flat
            }
        }
        if props:
            msg["data"].update({
                "color": props["color"],
                "metallic": props["metallic"],
                "roughness": props["roughness"],
                "albedo_path": props["albedo_path"],
                "normal_path": props["normal_path"],
                "emission_color": props["emission_color"],
                "emission_strength": props["emission_strength"],
            })

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
