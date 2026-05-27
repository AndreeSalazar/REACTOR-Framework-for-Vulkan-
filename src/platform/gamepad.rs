//! Gamepad input — Fase 5.5
//!
//! Wrapper sobre `gilrs` (100 % Rust, sin SDL ni C) que expone una API simple
//! y consistente con el resto del módulo `Input`. Soporta XInput (Xbox),
//! DualShock/DualSense (PlayStation), Switch Pro, y mandos genéricos vía
//! HID en Windows, Linux y macOS.
//!
//! ## Uso típico
//!
//! ```ignore
//! if ctx.input().gamepad().is_connected() {
//!     let aim = ctx.input().gamepad().right_stick();
//!     if ctx.input().gamepad().is_button_just_pressed(GamepadButton::South) {
//!         // disparar (botón A en Xbox, X en PlayStation)
//!     }
//! }
//! ```

#![allow(clippy::collapsible_match)]

use glam::Vec2;
use std::collections::{HashMap, HashSet};

// Re-exportamos los enums de gilrs para que el usuario no necesite la
// dependencia directa en su Cargo.toml.
pub use gilrs::{Axis as GamepadAxis, Button as GamepadButton};

/// Subsistema de gamepad. Detecta conexión/desconexión en caliente y mantiene
/// el estado del primer mando activo (suficiente para single-player).
pub struct Gamepad {
    gilrs: Option<gilrs::Gilrs>,
    active: Option<gilrs::GamepadId>,
    active_name: Option<String>,
    button_state: HashMap<GamepadButton, bool>,
    just_pressed: HashSet<GamepadButton>,
    just_released: HashSet<GamepadButton>,
    left_stick: Vec2,
    right_stick: Vec2,
    left_trigger: f32,
    right_trigger: f32,
    /// Radio del *dead-zone* radial aplicado a los sticks (default 0.15).
    /// Valores con módulo menor se redondean a `Vec2::ZERO`.
    pub deadzone: f32,
}

impl Default for Gamepad {
    fn default() -> Self {
        Self::new()
    }
}

impl Gamepad {
    /// Inicializa el subsistema. Si gilrs falla al cargar drivers (raro), el
    /// gamepad permanece "desconectado" y todos los métodos devuelven valores
    /// neutros — el juego nunca casca por falta de mando.
    pub fn new() -> Self {
        let gilrs = match gilrs::Gilrs::new() {
            Ok(g) => {
                println!("🎮 Gamepad subsystem initialized (gilrs)");
                Some(g)
            }
            Err(e) => {
                eprintln!(
                    "⚠️ Gamepad subsystem failed to init: {} (continuando sin mando)",
                    e
                );
                None
            }
        };

        // Si ya hay un mando conectado al arrancar, lo activamos.
        let (active, active_name) = match gilrs.as_ref() {
            Some(g) => g
                .gamepads()
                .next()
                .map(|(id, pad)| {
                    println!("🎮 Mando detectado: {} (id {:?})", pad.name(), id);
                    (Some(id), Some(pad.name().to_string()))
                })
                .unwrap_or((None, None)),
            None => (None, None),
        };

        Self {
            gilrs,
            active,
            active_name,
            button_state: HashMap::new(),
            just_pressed: HashSet::new(),
            just_released: HashSet::new(),
            left_stick: Vec2::ZERO,
            right_stick: Vec2::ZERO,
            left_trigger: 0.0,
            right_trigger: 0.0,
            deadzone: 0.15,
        }
    }

    /// Limpia los flags "just_pressed" / "just_released" — llamar al inicio de cada frame.
    pub(crate) fn begin_frame(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    /// Drena los eventos pendientes de gilrs. Debe llamarse una vez por frame
    /// **después** de `begin_frame()`.
    pub(crate) fn poll(&mut self) {
        // Drenar primero a un Vec local para soltar el préstamo de `self.gilrs`
        // antes de mutar el resto de campos (sticks, botones, conexión).
        let events: Vec<(gilrs::GamepadId, gilrs::EventType)> = match self.gilrs.as_mut() {
            Some(g) => {
                let mut buf = Vec::new();
                while let Some(gilrs::Event { id, event, .. }) = g.next_event() {
                    buf.push((id, event));
                }
                buf
            }
            None => return,
        };

        for (id, event) in events {
            match event {
                gilrs::EventType::Connected => {
                    if self.active.is_none() {
                        self.active = Some(id);
                        if let Some(pad) = self.gilrs.as_ref().and_then(|g| g.connected_gamepad(id))
                        {
                            let name = pad.name().to_string();
                            println!("🎮 Mando conectado: {}", name);
                            self.active_name = Some(name);
                        }
                    }
                }
                gilrs::EventType::Disconnected => {
                    if self.active == Some(id) {
                        println!("🎮 Mando desconectado");
                        self.active = None;
                        self.active_name = None;
                        self.reset_state();
                    }
                }
                _ if Some(id) != self.active => {
                    // Eventos de mandos secundarios: ignorar (single-player).
                }
                gilrs::EventType::ButtonPressed(btn, _) => {
                    if !*self.button_state.get(&btn).unwrap_or(&false) {
                        self.just_pressed.insert(btn);
                    }
                    self.button_state.insert(btn, true);
                }
                gilrs::EventType::ButtonReleased(btn, _) => {
                    self.button_state.insert(btn, false);
                    self.just_released.insert(btn);
                }
                gilrs::EventType::AxisChanged(axis, value, _) => match axis {
                    GamepadAxis::LeftStickX => self.left_stick.x = value,
                    GamepadAxis::LeftStickY => self.left_stick.y = value,
                    GamepadAxis::RightStickX => self.right_stick.x = value,
                    GamepadAxis::RightStickY => self.right_stick.y = value,
                    _ => {}
                },
                gilrs::EventType::ButtonChanged(btn, value, _) => match btn {
                    GamepadButton::LeftTrigger2 => self.left_trigger = value,
                    GamepadButton::RightTrigger2 => self.right_trigger = value,
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn reset_state(&mut self) {
        self.button_state.clear();
        self.left_stick = Vec2::ZERO;
        self.right_stick = Vec2::ZERO;
        self.left_trigger = 0.0;
        self.right_trigger = 0.0;
    }

    // ── Consultas ───────────────────────────────────────────────────────────

    /// `true` si hay al menos un mando conectado y activo.
    pub fn is_connected(&self) -> bool {
        self.active.is_some()
    }

    /// Nombre del mando activo (ej. `"Xbox Wireless Controller"`).
    pub fn name(&self) -> Option<&str> {
        self.active_name.as_deref()
    }

    /// `true` mientras el botón esté pulsado.
    pub fn is_button_down(&self, button: GamepadButton) -> bool {
        *self.button_state.get(&button).unwrap_or(&false)
    }

    /// `true` el frame en que el botón fue pulsado.
    pub fn is_button_just_pressed(&self, button: GamepadButton) -> bool {
        self.just_pressed.contains(&button)
    }

    /// `true` el frame en que el botón fue soltado.
    pub fn is_button_just_released(&self, button: GamepadButton) -> bool {
        self.just_released.contains(&button)
    }

    /// Posición del stick izquierdo (-1.0 .. 1.0 por eje), con deadzone radial.
    pub fn left_stick(&self) -> Vec2 {
        self.apply_deadzone(self.left_stick)
    }

    /// Posición del stick derecho (-1.0 .. 1.0 por eje), con deadzone radial.
    pub fn right_stick(&self) -> Vec2 {
        self.apply_deadzone(self.right_stick)
    }

    /// Valor analógico del gatillo izquierdo (0.0 .. 1.0).
    pub fn left_trigger(&self) -> f32 {
        self.left_trigger
    }

    /// Valor analógico del gatillo derecho (0.0 .. 1.0).
    pub fn right_trigger(&self) -> f32 {
        self.right_trigger
    }

    fn apply_deadzone(&self, stick: Vec2) -> Vec2 {
        let mag = stick.length();
        if mag < self.deadzone {
            Vec2::ZERO
        } else {
            // Re-mapeo radial: el borde del deadzone se convierte en 0,
            // el extremo del stick sigue siendo 1.
            let scaled = (mag - self.deadzone) / (1.0 - self.deadzone);
            stick.normalize() * scaled.clamp(0.0, 1.0)
        }
    }
}
