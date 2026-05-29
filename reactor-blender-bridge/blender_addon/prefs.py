import bpy

class REACTOR_AddonPreferences(bpy.types.AddonPreferences):
    bl_idname = __package__

    host: bpy.props.StringProperty(
        name="Host IP",
        description="IP of the REACTOR runtime bridge server",
        default="127.0.0.1"
    )
    
    port: bpy.props.IntProperty(
        name="Port",
        description="Port of the REACTOR runtime bridge server",
        default=19840,
        min=1024,
        max=65535
    )

    def draw(self, context):
        layout = self.layout
        layout.prop(self, "host")
        layout.prop(self, "port")

def register():
    bpy.utils.register_class(REACTOR_AddonPreferences)

def unregister():
    bpy.utils.unregister_class(REACTOR_AddonPreferences)
