use bevy::{
    color::palettes::tailwind, 
    input::mouse::AccumulatedMouseMotion, 
    render::camera::ScalingMode,
    prelude::*, render::view::RenderLayers,
};

pub mod simcore;
use crate::simcore::types::*;
use std::f32::consts::FRAC_PI_2;

// Keybindings constants
const PAN_UP_KEY: KeyCode = KeyCode::KeyW;
const PAN_DOWN_KEY: KeyCode = KeyCode::KeyS;
const PAN_LEFT_KEY: KeyCode = KeyCode::KeyA;
const PAN_RIGHT_KEY: KeyCode = KeyCode::KeyD;

const ORBIT_LEFT_KEY: KeyCode = KeyCode::KeyQ;
const ORBIT_RIGHT_KEY: KeyCode = KeyCode::KeyE;
const MOUSE_ORBIT_BUTTON: MouseButton = MouseButton::Right;
const MOUSE_PAN_BUTTON: MouseButton = MouseButton::Left;

// Added missing zoom keys
const ZOOM_IN_KEY: KeyCode = KeyCode::Equal;
const ZOOM_OUT_KEY: KeyCode = KeyCode::Minus;

// Camera constants
const DEFAULT_CAMERA_SENSITIVITY: Vec2 = Vec2::new(0.003, 0.002);
const DEFAULT_CAMERA_POSITION: Vec3 = Vec3::new(5.0, 5.0, 5.0);
const DEFAULT_ORBIT_RADIUS: f32 = 10.0;
const DEFAULT_ORBIT_YAW: f32 = 0.0;
const DEFAULT_ORBIT_PITCH: f32 = 1.0;
const PAN_SPEED: f32 = 5.0;
const ZOOM_SPEED: f32 = 5.0;
const MIN_ZOOM: f32 = 1.0;
const MAX_ZOOM: f32 = 20.0;

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

/// Rendering components
#[derive(Debug, Component)]
struct WorldModelCamera;

#[derive(Debug, Component)]
struct Player;

#[derive(Component, Default, PartialEq, Clone)]
pub enum CameraMode {
    #[default]
    Perspective3D,
    Orthographic3D,
    Orthographic2D,
}

// Unified camera controller component
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

#[derive(Resource, Default)]
struct CameraModeIndicator(String);

#[derive(Component)]
struct CameraModeText;



fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(CameraModeIndicator::default())
        .add_systems(
            Startup,
            (
                spawn_view_model,
                spawn_world_model,
                spawn_lights,
                spawn_text,
                setup_sim,
            ),
        )
        .add_systems(Update, camera_control_system)
        .add_systems(Update, (update_camera_mode_indicator, update_camera_mode_text))
        .run();
}

/// Used implicitly by all entities without a `RenderLayers` component.
/// Our world model camera and all objects other than the player are on this layer.
/// The light source belongs to both layers.
const DEFAULT_RENDER_LAYER: usize = 0;

/// Used by the view model camera and the player's arm.
/// The light source belongs to both layers.
const VIEW_MODEL_RENDER_LAYER: usize = 1;

fn spawn_view_model(mut commands: Commands) {
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

fn spawn_world_model(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let floor = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(10.0)));
    
    //let cube: Handle<Mesh> = meshes.add(Cuboid::new(2.0, 0.5, 1.0));
    //let cylinder = meshes.add(Cylinder::new(0.5, 1.0).mesh().resolution(50));

    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..Default::default()
    });
    
    commands.spawn((
        Mesh3d(floor), 
        MeshMaterial3d(material.clone())
    ));
    /*
    commands.spawn((
        Mesh3d(cube.clone()),
        MeshMaterial3d(material.clone()),
        Transform::from_xyz(0.0, 0.25, -3.0),
    ));
    commands.spawn((
        Mesh3d(cube),
        MeshMaterial3d(material.clone()),
        Transform::from_xyz(0.75, 1.75, 0.0),
    ));
    commands.spawn((
        Mesh3d(cylinder),
        MeshMaterial3d(material),
        Transform::from_xyz(1.0, 1.75, 0.0),
    ));
     */
    
    // Add XYZ arrows
    let arrow_length = 1.0;
    let arrow_thickness = 0.05;

    let x_arrow_material: Handle<StandardMaterial> = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),
        ..Default::default()
    });
    let y_arrow_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 1.0, 0.0),
        ..Default::default()
    });
    let z_arrow_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.0, 1.0),
        ..Default::default()
    });

    let arrow_mesh = meshes.add(Cylinder::new(arrow_thickness, arrow_length).mesh().resolution(20));

    // X-axis arrow
    commands.spawn((
        Mesh3d(arrow_mesh.clone()),
        MeshMaterial3d(x_arrow_material),
        Transform::from_translation(Vec3::new(arrow_length / 2.0, 0.0, 0.0))
            .with_rotation(Quat::from_rotation_z(-FRAC_PI_2)),
    ));

    // Y-axis arrow
    commands.spawn((
        Mesh3d(arrow_mesh.clone()),
        MeshMaterial3d(y_arrow_material),
        Transform::from_translation(Vec3::new(0.0, arrow_length / 2.0, 0.0)),
    ));

    // Z-axis arrow
    commands.spawn((
        Mesh3d(arrow_mesh),
        MeshMaterial3d(z_arrow_material),
        Transform::from_translation(Vec3::new(0.0, 0.0, arrow_length / 2.0))
            .with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
    ));

}

fn spawn_lights(mut commands: Commands) {
    commands.spawn((
        PointLight {
            color: Color::from(tailwind::ROSE_300),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-2.0, 4.0, -0.75),
        RenderLayers::from_layers(&[DEFAULT_RENDER_LAYER, VIEW_MODEL_RENDER_LAYER]),
    ));
}

fn spawn_text(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                right: Val::Px(12.0),
                ..default()
            },
            Name::new("CameraModeIndicator"),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Camera Mode: Perspective 3D"),
                CameraModeText,  
            ));
        });
}

fn update_camera_mode_indicator(
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

fn update_camera_mode_text(
    camera_mode_indicator: Res<CameraModeIndicator>,
    mut query: Query<&mut Text, With<CameraModeText>>,
) {
    for mut text in query.iter_mut() {
        *text = Text::new(camera_mode_indicator.0.clone());
    }
}
fn camera_control_system(
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
    
    // Pan with WASD
    if controller.mode == CameraMode::Orthographic2D {
        // 2D movement - simple XZ plane movement
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
        // 3D movement - camera-relative directions
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
    
    if mouse_buttons.pressed(MOUSE_PAN_BUTTON) {
        let total_delta = mouse_motion.delta;
        match controller.mode {
            CameraMode::Orthographic3D => {
                let sensitivity_x = controller.sensitivity.x;
                let right = transform.right();
                let up = transform.forward();
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

    if mouse_buttons.pressed(MOUSE_ORBIT_BUTTON) {
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

    // Mode switching with Space key
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

    // Final position and look-at calculation
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
fn setup_sim(mut commands: Commands) {
    let mut sim = Simulation::default();

    // Define joint positions
    let joint_pos_a = Position::Vec2(glam::Vec2::new(0.0, 0.0));
    let joint_pos_b = Position::Vec2(glam::Vec2::new(2.0, 0.0));
    let joint_pos_c = Position::Vec2(glam::Vec2::new(4.0, 2.0));
    let joint_pos_d = Position::Vec2(glam::Vec2::new(1.0, 2.0));

    // Create joints
    let joint_a = sim.joints.insert(Joint {
        position: joint_pos_a,
        joint_type: JointType::Fixed,
        connected_links: Vec::new(),
    });
    let joint_b = sim.joints.insert(Joint {
        position: joint_pos_b,
        joint_type: JointType::Revolute,
        connected_links: Vec::new(),
    });
    let joint_c = sim.joints.insert(Joint {
        position: joint_pos_c,
        joint_type: JointType::Revolute,
        connected_links: Vec::new(),
    });
    let joint_d = sim.joints.insert(Joint {
        position: joint_pos_d,
        joint_type: JointType::Revolute,
        connected_links: Vec::new(),
    });

    // Define link properties
    let link_ab_len = joint_pos_a.distance(joint_pos_b);
    let link_bc_len = joint_pos_b.distance(joint_pos_c);
    let link_cd_len = joint_pos_c.distance(joint_pos_d);
    let link_da_len = joint_pos_d.distance(joint_pos_a);

    // Create links
    let link_ab = sim.links.insert(Link {
        joints: vec![joint_a, joint_b],
        rigid: true,
    });
    let link_bc = sim.links.insert(Link {
        joints: vec![joint_b, joint_c],
        rigid: true,
    });
    let link_cd = sim.links.insert(Link {
        joints: vec![joint_c, joint_d],
        rigid: true,
    });
    let link_da = sim.links.insert(Link {
        joints: vec![joint_d, joint_a],
        rigid: true,
    });

    // Add links to joints
    sim.joints.get_mut(joint_a).unwrap().connected_links.push(link_ab);
    sim.joints.get_mut(joint_a).unwrap().connected_links.push(link_da);
    sim.joints.get_mut(joint_b).unwrap().connected_links.push(link_ab);
    sim.joints.get_mut(joint_b).unwrap().connected_links.push(link_bc);
    sim.joints.get_mut(joint_c).unwrap().connected_links.push(link_bc);
    sim.joints.get_mut(joint_c).unwrap().connected_links.push(link_cd);
    sim.joints.get_mut(joint_d).unwrap().connected_links.push(link_cd);
    sim.joints.get_mut(joint_d).unwrap().connected_links.push(link_da);

    // Fix joints A and C
    sim.constraints.push(Box::new(FixedPositionConstraint {
        joint_id: joint_a,
        target_position: joint_pos_a,
    }));
    sim.constraints.push(Box::new(FixedPositionConstraint {
        joint_id: joint_c,
        target_position: joint_pos_c,
    }));

    // Add distance constraints
    sim.constraints.push(Box::new(DistanceConstraint {
        joint_a: joint_a,
        joint_b: joint_b,
        target_distance: link_ab_len,
    }));
    sim.constraints.push(Box::new(DistanceConstraint {
        joint_a: joint_b,
        joint_b: joint_c,
        target_distance: link_bc_len,
    }));
    sim.constraints.push(Box::new(DistanceConstraint {
        joint_a: joint_c,
        joint_b: joint_d,
        target_distance: link_cd_len,
    }));
    sim.constraints.push(Box::new(DistanceConstraint {
        joint_a: joint_d,
        joint_b: joint_a,
        target_distance: link_da_len,
    }));

    // Insert the sim into the ECS
    commands.insert_resource(SimWrapper { sim });
}

