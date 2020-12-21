use super::InputEvent;
use crate::{Comp, Real, SystemMessage};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseDown {
    pub pos: MousePos,
    pub button: MouseButton,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseScroll {
    pub pos: MousePos,
    pub delta: (f32, f32),
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct MouseController {
    last_pos: Option<MousePos>,
    last_offset: Option<MousePos>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct MousePos {
    pub x: Real,
    pub y: Real,
}

impl MouseController {
    pub fn new() -> Self {
        MouseController {
            last_pos: None,
            last_offset: None,
        }
    }

    pub fn update_pos(&mut self, x: Real, y: Real) {
        let offset = self
            .last_pos
            .map(|last| MousePos {
                x: x - last.x,
                y: last.y - y, // reversed since y-coordinates go from bottom to top
            })
            .unwrap_or_default();

        self.last_pos = Some(MousePos { x, y });
        self.last_offset = Some(offset);
    }

    pub fn last_pos(&self) -> MousePos {
        self.last_pos.unwrap_or_default()
    }

    pub fn pressed_comp(&self, comp: &mut Comp, button: MouseButton) {
        let pos = self.last_pos();
        comp.send_system_msg(SystemMessage::Input(InputEvent::mouse_down(pos, button)))
    }

    pub fn mouse_scroll(&self, comp: &mut Comp, delta: (f32, f32)) {
        let pos = self.last_pos();
        comp.send_system_msg(SystemMessage::Input(InputEvent::mouse_scroll(MouseScroll {
            pos,
            delta,
        })))
    }
}
