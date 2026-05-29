"""REACTOR Live Link — Blender addon (FASE 0 + arranque FASE 1).

Conecta Blender al runtime de REACTOR via WebSocket localhost para construir
juegos en tiempo real. Esta version inicial maneja el handshake y ping/pong;
las fases siguientes anaden sync de mesh, materiales, luces y animaciones.

Instalacion:
    Edit -> Preferences -> Add-ons -> Install...
    Seleccionar esta carpeta empaquetada como .zip
    Activar "REACTOR Live Link"

Panel:
    3D Viewport -> N-panel -> pestana "REACTOR"
"""

bl_info = {
    "name": "REACTOR Live Link",
    "author": "Salazar-interactive",
    "version": (0, 1, 0),
    "blender": (4, 2, 0),
    "location": "View3D > N-Panel > REACTOR",
    "description": "Live sync between Blender and REACTOR runtime (FASE 0 - ping/pong)",
    "category": "Development",
    "doc_url": "https://github.com/AndreeSalazar/REACTOR-Framework-for-Vulkan-",
}

# Importaciones diferidas para que el modulo se pueda inspeccionar sin Blender
# (util para tests y para que el linter no se queje).
try:
    import bpy  # noqa: F401
    _IN_BLENDER = True
except ImportError:
    _IN_BLENDER = False


def register():
    if not _IN_BLENDER:
        return
    from . import prefs, panel
    from .operators import connect as op_connect

    prefs.register()
    op_connect.register()
    panel.register()


def unregister():
    if not _IN_BLENDER:
        return
    from . import prefs, panel
    from .operators import connect as op_connect

    panel.unregister()
    op_connect.unregister()
    prefs.unregister()


if __name__ == "__main__":
    register()
