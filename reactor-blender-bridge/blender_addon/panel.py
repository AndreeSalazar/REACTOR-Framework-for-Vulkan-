import bpy

class REACTOR_PT_live_panel(bpy.types.Panel):
    bl_label = "REACTOR Live Link"
    bl_category = "REACTOR"
    bl_space_type = 'VIEW_3D'
    bl_region_type = 'UI'

    def draw(self, context):
        layout = self.layout
        scene = context.scene
        
        # Display settings info
        box = layout.box()
        box.label(text="Configuración de Red", icon='NETWORK_BACKGROUND')
        
        # Access preferences
        try:
            prefs = context.preferences.addons[__package__.split('.')[0]].preferences
            box.label(text=f"Servidor: {prefs.host}:{prefs.port}")
        except Exception:
            box.label(text="Error leyendo preferencias")
            
        # Display Status
        status_box = layout.box()
        status_box.label(text="Conexión", icon='CONSOLE')
        
        status = scene.reactor_status
        connected = scene.reactor_connected
        
        row = status_box.row()
        row.label(text="Estado:")
        if connected:
            row.label(text=status, icon='COLOR_GREEN')
            latency_row = status_box.row()
            latency_row.label(text="Latencia:")
            latency_row.label(text=scene.reactor_latency, icon='TIME')
        else:
            if "Conectando" in status:
                row.label(text=status, icon='COLOR_YELLOW')
            else:
                row.label(text=status, icon='COLOR_RED')
                
        # Connection button
        layout.separator()
        if not connected and "Conectando" not in status:
            layout.operator("reactor.live_connect", icon='PLAY', text="Conectar a REACTOR")
        else:
            layout.operator("reactor.live_disconnect", icon='PAUSE', text="Desconectar")

def register():
    bpy.utils.register_class(REACTOR_PT_live_panel)

def unregister():
    bpy.utils.unregister_class(REACTOR_PT_live_panel)
