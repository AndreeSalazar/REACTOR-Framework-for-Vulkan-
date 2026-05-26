use glam::Vec2;
use std::collections::HashSet;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::platform::gamepad::Gamepad;

pub struct Input {
    pressed_keys: HashSet<KeyCode>,
    just_pressed_keys: HashSet<KeyCode>,
    just_released_keys: HashSet<KeyCode>,
    pressed_mouse_buttons: HashSet<MouseButton>,
    mouse_position: Vec2,
    mouse_delta: Vec2,
    scroll_delta: f32,
    /// Subsistema de gamepad (Fase 5.5 — siempre presente, "desconectado"
    /// hasta que se enchufe un mando).
    gamepad: Gamepad,
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Input {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            just_pressed_keys: HashSet::new(),
            just_released_keys: HashSet::new(),
            pressed_mouse_buttons: HashSet::new(),
            mouse_position: Vec2::ZERO,
            mouse_delta: Vec2::ZERO,
            scroll_delta: 0.0,
            gamepad: Gamepad::new(),
        }
    }

    pub fn begin_frame(&mut self) {
        self.just_pressed_keys.clear();
        self.just_released_keys.clear();
        self.mouse_delta = Vec2::ZERO;
        self.scroll_delta = 0.0;
        // Drenar eventos del gamepad y actualizar estado.
        self.gamepad.begin_frame();
        self.gamepad.poll();
    }

    pub fn process_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => match state {
                ElementState::Pressed => {
                    if !self.pressed_keys.contains(keycode) {
                        self.just_pressed_keys.insert(*keycode);
                    }
                    self.pressed_keys.insert(*keycode);
                }
                ElementState::Released => {
                    self.pressed_keys.remove(keycode);
                    self.just_released_keys.insert(*keycode);
                }
            },
            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => {
                    self.pressed_mouse_buttons.insert(*button);
                }
                ElementState::Released => {
                    self.pressed_mouse_buttons.remove(button);
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                let new_pos = Vec2::new(position.x as f32, position.y as f32);
                self.mouse_delta = new_pos - self.mouse_position;
                self.mouse_position = new_pos;
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                winit::event::MouseScrollDelta::LineDelta(_, y) => {
                    self.scroll_delta = *y;
                }
                winit::event::MouseScrollDelta::PixelDelta(pos) => {
                    self.scroll_delta = pos.y as f32 / 100.0;
                }
            },
            _ => {}
        }
    }

    // Keyboard
    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed_keys.contains(&key)
    }

    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        self.just_released_keys.contains(&key)
    }

    // Mouse
    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.pressed_mouse_buttons.contains(&button)
    }

    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse_delta
    }

    pub fn scroll_delta(&self) -> f32 {
        self.scroll_delta
    }

    // Movement helpers
    pub fn get_movement_vector(&self) -> Vec2 {
        let mut movement = Vec2::ZERO;

        if self.is_key_down(KeyCode::KeyW) || self.is_key_down(KeyCode::ArrowUp) {
            movement.y -= 1.0;
        }
        if self.is_key_down(KeyCode::KeyS) || self.is_key_down(KeyCode::ArrowDown) {
            movement.y += 1.0;
        }
        if self.is_key_down(KeyCode::KeyA) || self.is_key_down(KeyCode::ArrowLeft) {
            movement.x -= 1.0;
        }
        if self.is_key_down(KeyCode::KeyD) || self.is_key_down(KeyCode::ArrowRight) {
            movement.x += 1.0;
        }

        if movement.length_squared() > 0.0 {
            movement = movement.normalize();
        }

        movement
    }

    // =========================================================================
    // 🎮 Gamepad (Fase 5.5)
    // =========================================================================

    /// Acceso al subsistema de gamepad (sticks, gatillos, botones).
    pub fn gamepad(&self) -> &Gamepad {
        &self.gamepad
    }

    /// Acceso mutable (raramente necesario — sólo para ajustar `deadzone`).
    pub fn gamepad_mut(&mut self) -> &mut Gamepad {
        &mut self.gamepad
    }
}
