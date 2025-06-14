use bevy::prelude::*;
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton;

#[derive(Resource, Debug, Clone)]
pub struct KeyBindings {
    pub pan_up: KeyCode,
    pub pan_down: KeyCode,
    pub pan_left: KeyCode,
    pub pan_right: KeyCode,
    pub shift: KeyCode,
    pub orbit_left: KeyCode,
    pub orbit_right: KeyCode,
    pub mouse_orbit: MouseButton,
    pub mouse_pan: MouseButton,
    pub zoom_in: KeyCode,
    pub zoom_out: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            pan_up: KeyCode::KeyW,
            pan_down: KeyCode::KeyS,
            pan_left: KeyCode::KeyA,
            pan_right: KeyCode::KeyD,
            shift: KeyCode::ShiftLeft,
            orbit_left: KeyCode::KeyQ,
            orbit_right: KeyCode::KeyE,
            mouse_orbit: MouseButton::Right,
            mouse_pan: MouseButton::Left,
            zoom_in: KeyCode::Equal,
            zoom_out: KeyCode::Minus,
        }
    }
}
