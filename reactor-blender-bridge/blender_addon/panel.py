import bpy
import traceback

class REACTOR_PT_live_panel(bpy.types.Panel):
    bl_label = "REACTOR Live Link"
    bl_category = "REACTOR"
    bl_space_type = 'VIEW_3D'
    bl_region_type = 'UI'

    def draw(self, context):
        try:
            self.draw_safe(context)
        except Exception as e:
            layout = self.layout
            box = layout.box()
            box.alert = True
            box.label(text="Error en Panel REACTOR", icon='ERROR')
            # Mostrar la excepción en la UI para diagnóstico rápido
            box.label(text=str(e))
            # Imprimir traceback completo en la consola de Blender
            traceback.print_exc()

    def draw_safe(self, context):
        layout = self.layout
        scene = context.scene
        
        # Display settings info
        box = layout.box()
        box.label(text="Configuración de Red", icon='WORLD')
        
        # Access preferences safely
        try:
            prefs = context.preferences.addons[__package__.split('.')[0]].preferences
            box.label(text=f"Servidor: {prefs.host}:{prefs.port}", icon='LINK')
        except Exception:
            box.label(text="Error leyendo preferencias", icon='ERROR')
            
        # Display Status safely using getattr to prevent AttributeError
        status_box = layout.box()
        status_box.label(text="Conexión", icon='CONSOLE')
        
        status = getattr(scene, "reactor_status", "Desconectado")
        connected = getattr(scene, "reactor_connected", False)
        latency = getattr(scene, "reactor_latency", "N/A")
        
        row = status_box.row()
        row.label(text="Estado:")
        
        if connected:
            row.label(text=status, icon='STATUS_YES')
            latency_row = status_box.row()
            latency_row.label(text="Latencia:")
            latency_row.label(text=latency, icon='TIMER')
        else:
            if "Conectando" in status:
                row.label(text=status, icon='STATUS_ALERT')
            else:
                row.label(text=status, icon='STATUS_NO')
                
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
