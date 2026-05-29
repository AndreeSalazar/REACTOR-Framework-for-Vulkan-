"""Encoder de transformaciones Blender → REACTOR.

Convierte matrices de transformación del sistema de coordenadas
de Blender (Z-Up, Right-Handed) al de REACTOR/Vulkan (Y-Up, Right-Handed).

La matriz de cambio de base es una rotación de -90° sobre el eje X:

    M_B→R = | 1   0   0   0 |
            | 0   0   1   0 |
            | 0  -1   0   0 |
            | 0   0   0   1 |

Para una posición:
    X_R =  X_B
    Y_R =  Z_B
    Z_R = -Y_B

Para una matriz completa:
    T_Reactor = M_B→R · T_Blender · M_B→R⁻¹
"""

import mathutils  # type: ignore  (Blender's built-in math library)


# Basis change matrix: Blender Z-Up RH → REACTOR Y-Up RH
# Rotation of -90° around X axis
_SWAP = mathutils.Matrix((
    (1.0,  0.0,  0.0, 0.0),
    (0.0,  0.0,  1.0, 0.0),
    (0.0, -1.0,  0.0, 0.0),
    (0.0,  0.0,  0.0, 1.0),
))

_SWAP_INV = _SWAP.inverted()


def blender_to_reactor_matrix(matrix_world):
    """Convierte una matrix_world de Blender (4x4, Z-Up RH)
    a una matriz REACTOR (4x4, Y-Up RH) en formato row-major flat list [16 floats].

    Args:
        matrix_world: bpy.types.Object.matrix_world (mathutils.Matrix 4x4)

    Returns:
        list[float]: 16 floats en row-major order para enviar por WebSocket.
    """
    # T_R = M · T_B · M⁻¹
    converted = _SWAP @ matrix_world @ _SWAP_INV

    # Blender stores matrices column-major internally.
    # REACTOR expects row-major, so we transpose before flattening.
    row_major = converted.transposed()
    flat = []
    for row in range(4):
        for col in range(4):
            flat.append(row_major[row][col])
    return flat


def blender_to_reactor_position(pos):
    """Convierte una posición (x, y, z) de Blender a REACTOR.

    Args:
        pos: Vector or tuple (x_b, y_b, z_b) in Blender space.

    Returns:
        tuple: (x_r, y_r, z_r) in REACTOR space.
    """
    return (pos[0], pos[2], -pos[1])
