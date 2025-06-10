use bevy::{
    input::mouse::AccumulatedMouseMotion, 
    render::camera::ScalingMode,
    prelude::*,
};
use std::f32::consts::FRAC_PI_2;


use crate::util::constants::*;

/// Rendering components
#[derive(Debug, Component)]
pub struct WorldModelCamera;

#[derive(Debug, Component)]
pub struct Player;

#[derive(Component, Default, PartialEq, Clone)]
pub enum CameraMode {
    #[default]
    Perspective3D,
    Orthographic3D,
    Orthographic2D,
}


#[derive(Resource, Default)]
pub struct CameraModeIndicator(String);

#[derive(Component)]
pub struct CameraModeText;


// camera controller component
#[derive(Component)]
pub struct CameraController {
    pub orbit_radius: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub sensitivity: Vec2,
    pub pan_offset: Vec3,
    pub mode: CameraMode,
} 

impl Default for CameraController {
    fn default() -> Self {
        Self {
            orbit_radius: DEFAULT_ORBIT_RADIUS,
            yaw: DEFAULT_ORBIT_YAW,
            pitch: DEFAULT_ORBIT_PITCH,
            sensitivity: DEFAULT_CAMERA_SENSITIVITY,
            pan_offset: Vec3::ZERO,
            mode: CameraMode::Perspective3D,
        }
    }
}



pub fn spawn_view_model(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Projection::from(PerspectiveProjection {
            fov: FRAC_PI_2,
            ..default()
        }),
        Transform::from_translation(DEFAULT_CAMERA_POSITION).looking_at(Vec3::ZERO, Vec3::Y),
        Player,
        CameraController::default(),
    ));
}

pub fn update_camera_mode_indicator(
    camera_query: Query<&CameraController, With<Player>>,
    mut camera_mode_indicator: ResMut<CameraModeIndicator>,
) {
    if let Ok(controller) = camera_query.single() {
        camera_mode_indicator.0 = match controller.mode {
            CameraMode::Perspective3D => "Camera Mode: Perspective 3D".to_string(),
            CameraMode::Orthographic3D => "Camera Mode: Orthographic 3D".to_string(),
            CameraMode::Orthographic2D => "Camera Mode: Orthographic 2D".to_string(),
        };
    }
}
pub fn update_camera_mode_text(
    camera_mode_indicator: Res<CameraModeIndicator>,
    mut query: Query<&mut Text, With<CameraModeText>>,
) {
    for mut text in query.iter_mut() {
        *text = Text::new(camera_mode_indicator.0.clone());
    }
}


pub fn camera_control_system(
    mut cameras: Query<(&mut Transform, &mut Projection, &mut CameraController), With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
    windows: Query<&Window>,
) {
    let Ok((mut transform, mut projection, mut controller)) = cameras.single_mut() else {
        return;
    };
    
    let dt = time.delta_secs();
    
    if controller.mode == CameraMode::Orthographic2D {
        let mut pan_move = Vec3::ZERO;
        if keys.pressed(PAN_UP_KEY) {
            pan_move.z -= PAN_SPEED * dt; 
        }
        if keys.pressed(PAN_DOWN_KEY) {
            pan_move.z += PAN_SPEED * dt;
        }
        if keys.pressed(PAN_RIGHT_KEY) {
            pan_move.x += PAN_SPEED * dt;
        }
        if keys.pressed(PAN_LEFT_KEY) {
            pan_move.x -= PAN_SPEED * dt;
        }
        controller.pan_offset += pan_move;
    } else {
        let forward = transform.forward();
        let right = transform.right();
        let mut pan_move = Vec3::ZERO;
    
        if keys.pressed(PAN_UP_KEY) {
            pan_move += forward * PAN_SPEED * dt;
        }
        if keys.pressed(PAN_DOWN_KEY) {
            pan_move -= forward * PAN_SPEED * dt;
        }
        if keys.pressed(PAN_RIGHT_KEY) {
            pan_move += right * PAN_SPEED * dt;
        }
        if keys.pressed(PAN_LEFT_KEY) {
            pan_move -= right * PAN_SPEED * dt;
        }
        
        pan_move.y = 0.0;
        controller.pan_offset += pan_move;
    }
    
    if mouse_buttons.pressed(MOUSE_PAN_BUTTON) && keys.pressed(SHIFT){
        let total_delta = mouse_motion.delta;
        match controller.mode {
            CameraMode::Orthographic3D => {
                let sensitivity_x = controller.sensitivity.x;
                let right = transform.right();
                let up = transform.up();
                controller.pan_offset += (right * -total_delta.x + up * total_delta.y) * sensitivity_x;
            }
            CameraMode::Perspective3D | CameraMode::Orthographic2D => {
                let sensitivity_x = controller.sensitivity.x;
                let right = transform.right();
                let up = transform.up();
                controller.pan_offset += (right * -total_delta.x + up * total_delta.y) * sensitivity_x;
            }
        }
    }
    let mut yaw_delta = 0.0;
    let mut pitch_delta = 0.0;

    if mouse_buttons.pressed(MOUSE_ORBIT_BUTTON) && keys.pressed(SHIFT){
        yaw_delta += -mouse_motion.delta.x * controller.sensitivity.x;
        if controller.mode != CameraMode::Orthographic2D {
            pitch_delta += -mouse_motion.delta.y * controller.sensitivity.y;
        }
    }
    if keys.pressed(ORBIT_LEFT_KEY) {
        yaw_delta += 1.0 * dt;
    }
    if keys.pressed(ORBIT_RIGHT_KEY) {
        yaw_delta -= 1.0 * dt;
    }

    controller.yaw += yaw_delta;
    
    // Only apply pitch for 3D modes
    if controller.mode != CameraMode::Orthographic2D {
        controller.pitch = (controller.pitch + pitch_delta).clamp(-FRAC_PI_2 + 0.05, FRAC_PI_2 - 0.05);
    }

    // Zoom with +/- keys
    match controller.mode {
        CameraMode::Perspective3D => {
            if keys.pressed(ZOOM_IN_KEY) {
                controller.orbit_radius -= ZOOM_SPEED * dt;
            }
            if keys.pressed(ZOOM_OUT_KEY) {
                controller.orbit_radius += ZOOM_SPEED * dt;
            }
            controller.orbit_radius = controller.orbit_radius.clamp(MIN_ZOOM, MAX_ZOOM);
        }
        CameraMode::Orthographic3D | CameraMode::Orthographic2D => {
            if let Projection::Orthographic(ref mut ortho) = *projection {
                if keys.pressed(ZOOM_IN_KEY) {
                    ortho.scale -= ZOOM_SPEED * dt;
                }
                if keys.pressed(ZOOM_OUT_KEY) {
                    ortho.scale += ZOOM_SPEED * dt;
                }
                ortho.scale = ortho.scale.clamp(MIN_ZOOM, MAX_ZOOM);
            }
        }
    }

    if keys.just_pressed(KeyCode::Space) {
        controller.mode = match controller.mode {
            CameraMode::Perspective3D => {
                *projection = Projection::from(OrthographicProjection {
                    scaling_mode: ScalingMode::AutoMin { min_width: 6.0, min_height: 6.0 },
                    ..OrthographicProjection::default_2d()
                });
                CameraMode::Orthographic3D
            }
            CameraMode::Orthographic3D => {
                *projection = Projection::from(OrthographicProjection {
                    scaling_mode: ScalingMode::AutoMin { min_width: 6.0, min_height: 6.0 },
                    ..OrthographicProjection::default_2d()
                });
                CameraMode::Orthographic2D
            }
            CameraMode::Orthographic2D => {
                *projection = Projection::Perspective(PerspectiveProjection {
                    fov: FRAC_PI_2,
                    aspect_ratio: {
                        let window = windows.single().unwrap();
                        window.width() / window.height()
                    },
                    near: 0.1,
                    far: 1000.0,
                });
                CameraMode::Perspective3D
            }
        };
    }

    match controller.mode {
            CameraMode::Orthographic2D => {

                transform.translation = Vec3::new(
                    controller.pan_offset.x,
                    10.0,
                    controller.pan_offset.z,
                );

                // Set rotation: yaw around Y, then -90deg around X to look down
                transform.rotation = Quat::from_axis_angle(Vec3::Y, controller.yaw)
                    * Quat::from_axis_angle(Vec3::X, -std::f32::consts::FRAC_PI_2);
            }       
             _ => {
            let (sin_yaw, cos_yaw) = controller.yaw.sin_cos();
            let (sin_pitch, cos_pitch) = controller.pitch.sin_cos();

            let offset = Vec3::new(
                controller.orbit_radius * cos_pitch * sin_yaw,
                controller.orbit_radius * sin_pitch,
                controller.orbit_radius * cos_pitch * cos_yaw,
            );

            let target = controller.pan_offset;
            transform.translation = target + offset;
            transform.look_at(target, Vec3::Y);
        }
    }
}



