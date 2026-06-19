//! One-call exit helpers.

use crate::app::ReactorContext;
use winit::keyboard::KeyCode;

pub fn check_escape(ctx: &mut ReactorContext) -> bool {
    if ctx.input().is_key_down(KeyCode::Escape) {
        ctx.reactor.exit_requested = true;
        true
    } else {
        false
    }
}
