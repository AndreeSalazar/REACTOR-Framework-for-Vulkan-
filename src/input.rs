use winit::event::{ElementState, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};
use std::collections::HashSet;

#[derive(Default)]
pub struct Input {
    pressed_keys: HashSet<KeyCode>,
}

impl Input {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
        }
    }

    pub fn process_event(&mut self, event: &WindowEvent) {
        if let WindowEvent::KeyboardInput {
            event:
                winit::event::KeyEvent {
                    state,
                    physical_key: PhysicalKey::Code(keycode),
                    ..
                },
            ..
        } = event
        {
            match state {
                ElementState::Pressed => {
                    self.pressed_keys.insert(*keycode);
                }
                ElementState::Released => {
                    self.pressed_keys.remove(keycode);
                }
            }
        }
    }

    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }
}
