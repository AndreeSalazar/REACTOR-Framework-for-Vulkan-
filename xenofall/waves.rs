// =============================================================================
// XENOFALL — Wave Design
// =============================================================================
// Encounter pacing belongs outside the runtime loop. Keeping waves here makes it
// easier to tune XENOFALL as a repeatable rendering/gameplay stress test.
// =============================================================================

use crate::xenofall::types::WaveDef;

pub fn build_waves() -> Vec<WaveDef> {
    vec![
        // Oleada 1: Casa abandonada — pocos zombies lentos
        WaveDef {
            trigger_z: 8.0,
            count: 3,
            spread: 3.0,
            depth: 4.0,
            _height_range: (0.8, 0.8),
            speed_mult: 0.7,
            enemy_hp: 1,
        },
        // Oleada 2: Más infectados emergen
        WaveDef {
            trigger_z: 18.0,
            count: 5,
            spread: 4.0,
            depth: 5.0,
            _height_range: (0.8, 0.8),
            speed_mult: 0.8,
            enemy_hp: 1,
        },
        // Oleada 3: Mezcla — más enemigos, mayor spread
        WaveDef {
            trigger_z: 28.0,
            count: 5,
            spread: 4.5,
            depth: 4.0,
            _height_range: (0.8, 0.8),
            speed_mult: 0.9,
            enemy_hp: 2,
        },
        // Oleada 4: Emboscada lateral
        WaveDef {
            trigger_z: 38.0,
            count: 7,
            spread: 5.0,
            depth: 6.0,
            _height_range: (0.8, 0.8),
            speed_mult: 1.0,
            enemy_hp: 2,
        },
        // Oleada 5: Horda densa — Laboratorio
        WaveDef {
            trigger_z: 48.0,
            count: 6,
            spread: 5.0,
            depth: 5.0,
            _height_range: (0.8, 0.8),
            speed_mult: 1.1,
            enemy_hp: 2,
        },
        // Oleada 6: Mutantes rápidos
        WaveDef {
            trigger_z: 58.0,
            count: 8,
            spread: 5.0,
            depth: 6.0,
            _height_range: (0.8, 0.8),
            speed_mult: 1.2,
            enemy_hp: 3,
        },
        // Oleada 7: Caos total — pasillos de contención rotos
        WaveDef {
            trigger_z: 68.0,
            count: 8,
            spread: 5.5,
            depth: 6.0,
            _height_range: (0.8, 0.8),
            speed_mult: 1.3,
            enemy_hp: 3,
        },
        // Oleada 8: JEFE FINAL — Sujeto Omega
        WaveDef {
            trigger_z: 78.0,
            count: 12,
            spread: 5.5,
            depth: 8.0,
            _height_range: (0.8, 0.8),
            speed_mult: 1.0,
            enemy_hp: 4,
        },
    ]
}
