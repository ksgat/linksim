
pub use bevy::prelude::*;
use crate::simcore::types::*;

// Keybindings pub constants
pub const PAN_UP_KEY: KeyCode = KeyCode::KeyW;
pub const PAN_DOWN_KEY: KeyCode = KeyCode::KeyS;
pub const PAN_LEFT_KEY: KeyCode = KeyCode::KeyA;
pub const PAN_RIGHT_KEY: KeyCode = KeyCode::KeyD;
pub const SHIFT: KeyCode = KeyCode::ShiftLeft;
pub const ORBIT_LEFT_KEY: KeyCode = KeyCode::KeyQ;
pub const ORBIT_RIGHT_KEY: KeyCode = KeyCode::KeyE;
pub const MOUSE_ORBIT_BUTTON: MouseButton = MouseButton::Right;
pub const MOUSE_PAN_BUTTON: MouseButton = MouseButton::Left;

// Added missing zoom keys
pub const ZOOM_IN_KEY: KeyCode = KeyCode::Equal;
pub const ZOOM_OUT_KEY: KeyCode = KeyCode::Minus;

// Camera pub constants
pub const DEFAULT_CAMERA_SENSITIVITY: Vec2 = Vec2::new(0.003, 0.002);
pub const DEFAULT_CAMERA_POSITION: Vec3 = Vec3::new(5.0, 5.0, 5.0);
pub const DEFAULT_ORBIT_RADIUS: f32 = 10.0;
pub const DEFAULT_ORBIT_YAW: f32 = 0.0;
pub const DEFAULT_ORBIT_PITCH: f32 = 1.0;
pub const PAN_SPEED: f32 = 5.0;
pub const ZOOM_SPEED: f32 = 5.0;
pub const MIN_ZOOM: f32 = 1.0;
pub const MAX_ZOOM: f32 = 20.0;

/// Sim core wrapper types
#[derive(Resource)]
pub struct SimWrapper {
    pub sim: Simulation,
}

#[derive(Component)]
pub struct JointWrapper {
    pub joint_id: JointId,
}

#[derive(Component)]
pub struct LinkWrapper {
    pub link_id: LinkId,
}



