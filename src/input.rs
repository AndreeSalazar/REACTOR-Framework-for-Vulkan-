use winit::event::{ElementState, WindowEvent, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};
use std::collections::HashSet;
use glam::Vec2;

#[derive(Default)]
pub struct Input {
    pressed_keys: HashSet<KeyCode>,
    pressed_mouse: HashSet<MouseButton>,
    mouse_pos: Vec2,
    mouse_delta: Vec2,
    scroll_delta: f32,
}

impl Input {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            pressed_mouse: HashSet::new(),
            mouse_pos: Vec2::ZERO,
            mouse_delta: Vec2::ZERO,
            scroll_delta: 0.0,
        }
    }

    pub fn begin_frame(&mut self) {
        self.mouse_delta = Vec2::ZERO;
        self.scroll_delta = 0.0;
    }

    pub fn process_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                event: winit::event::KeyEvent {
                    state,
                    physical_key: PhysicalKey::Code(keycode),
                    ..
                },
                ..
            } => {
                match state {
                    ElementState::Pressed => { self.pressed_keys.insert(*keycode); }
                    ElementState::Released => { self.pressed_keys.remove(keycode); }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                match state {
                    ElementState::Pressed => { self.pressed_mouse.insert(*button); }
                    ElementState::Released => { self.pressed_mouse.remove(button); }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let new_pos = Vec2::new(position.x as f32, position.y as f32);
                self.mouse_delta = new_pos - self.mouse_pos;
                self.mouse_pos = new_pos;
            }
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => {
                        self.scroll_delta = *y;
                    }
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        self.scroll_delta = pos.y as f32 / 100.0;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn is_mouse_down(&self, button: MouseButton) -> bool {
        self.pressed_mouse.contains(&button)
    }

    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_pos
    }

    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse_delta
    }

    pub fn scroll_delta(&self) -> f32 {
        self.scroll_delta
    }
}
