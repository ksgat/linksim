use bevy::{
    color::palettes::tailwind, 
    input::mouse::AccumulatedMouseMotion, 
    pbr::NotShadowCaster,
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
const ZOOM_IN_KEY: KeyCode = KeyCode::Equal;
const ZOOM_OUT_KEY: KeyCode = KeyCode::Minus;
const ORBIT_LEFT_KEY: KeyCode = KeyCode::KeyQ;
const ORBIT_RIGHT_KEY: KeyCode = KeyCode::KeyE;
const MOUSE_PAN_BUTTON: MouseButton = MouseButton::Right;
const MOUSE_ORBIT_BUTTON: MouseButton = MouseButton::Left;


// Camera constants
const DEFAULT_CAMERA_SENSITIVITY: Vec2 = Vec2::new(0.003, 0.002);
const DEFAULT_CAMERA_POSITION: Vec3 = Vec3::new(5.0, 5.0, 5.0);
const DEFAULT_ORBIT_RADIUS: f32 = 10.0;
const DEFAULT_ORBIT_YAW: f32 = 0.0;
const DEFAULT_ORBIT_PITCH: f32 = 0.3;
const PAN_SPEED: f32 = 0.1;
const ZOOM_SPEED: f32 = 0.05;
const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 10.0;

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

#[derive(Debug, Component, Deref, DerefMut)]
struct CameraSensitivity(Vec2);

impl Default for CameraSensitivity {
    fn default() -> Self {
        Self(DEFAULT_CAMERA_SENSITIVITY)
    }
}

#[derive(Component)]
struct OrbitControl {
    radius: f32,
    yaw: f32,
    pitch: f32,
}

#[derive(Component, Default, PartialEq)]
enum CameraMode {
    #[default]
    Perspective3D,
    Orthographic3D,
    Orthographic2D,
}

impl Default for OrbitControl {
    fn default() -> Self {
        Self {
            radius: DEFAULT_ORBIT_RADIUS,
            yaw: DEFAULT_ORBIT_YAW,
            pitch: DEFAULT_ORBIT_PITCH,
        }
    }
}
#[derive(Component, Debug, Default)]
struct PanOffset(Vec3);

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
        .add_systems(Update, pan_ortho_camera)
        .add_systems(Update, zoom_ortho_camera)
        .add_systems(Update, orbit_ortho_camera)
        .add_systems(Update, pan_perspective_camera)
        .add_systems(Update, zoom_perspective_camera)
        .add_systems(Update, orbit_perspective_camera)
        .add_systems(Update, toggle_camera_mode)
        .add_systems(Update, (update_camera_mode_indicator,update_camera_mode_text))
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
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 6.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_translation(DEFAULT_CAMERA_POSITION).looking_at(Vec3::ZERO, Vec3::Y),
        Player,
        CameraSensitivity::default(),
        OrbitControl::default(),
        PanOffset::default(),
        CameraMode::default(),
    ));
}


fn spawn_world_model(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let floor = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(10.0)));
    let cube = meshes.add(Cuboid::new(2.0, 0.5, 1.0));
    let cylinder = meshes.add(Cylinder::new(0.5, 1.0).mesh().resolution(50));

    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..Default::default()
    });

    commands.spawn((Mesh3d(floor), MeshMaterial3d(material.clone())));
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
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        Name::new("Instructions"),
    ))
    .with_children(|parent| {
        parent.spawn(Text::new(concat!(
            "Move the camera with your mouse.\n",
            "Right click + drag to pan\n",
            "Left click + drag to orbit\n",
            "WASD to pan with keyboard\n",
            "+/- to zoom\n",
            "Q/E to orbit with keyboard\n",
            "Space to toggle camera mode\n",
        )));
    });

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
            Text::new("{CameraModeIndicator}"),
            CameraModeText,  
        ));
    });

}


fn update_camera_mode_indicator(
    camera_mode_query: Query<&CameraMode>,
    mut camera_mode_indicator: ResMut<CameraModeIndicator>,
) {
    if let Ok(camera_mode) = camera_mode_query.single() {
        camera_mode_indicator.0 = match camera_mode {
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


fn pan_ortho_camera(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut query: Query<(
        &mut PanOffset,
        &Transform,
        &CameraSensitivity,
        &CameraMode,
    ), With<Player>>,
) {
    let (mut pan_offset, transform, sensitivity, camera_mode) = match query.single_mut() {
        Ok(values) => values,
        Err(_) => return,
    };

    // Keyboard panning
    let mut movement = Vec3::ZERO;
    if keys.pressed(PAN_UP_KEY) {
        movement += Vec3::new(transform.forward().x, 0.0, transform.forward().z);
    }
    if keys.pressed(PAN_DOWN_KEY) {
        movement -= Vec3::new(transform.forward().x, 0.0, transform.forward().z);
    }
    if keys.pressed(PAN_RIGHT_KEY) {
        movement += Vec3::new(transform.right().x, 0.0, transform.right().z);
    }
    if keys.pressed(PAN_LEFT_KEY) {
        movement -= Vec3::new(transform.right().x, 0.0, transform.right().z);
    }

    if movement != Vec3::ZERO {
        let delta = movement.normalize_or_zero() * PAN_SPEED;
        pan_offset.0 += Vec3::new(delta.x, 0.0, delta.z);
    }

    // Mouse panning
    if mouse_buttons.pressed(MOUSE_PAN_BUTTON) {
        let total_delta = mouse_motion.delta;
        match camera_mode {
            CameraMode::Orthographic2D => {
                // Simple 2D panning (XZ plane)
                pan_offset.0 += Vec3::new(-total_delta.x, 0.0, total_delta.y) * sensitivity.x;
            }
            CameraMode::Perspective3D | CameraMode::Orthographic3D => {
                // 3D panning - use camera relative directions
                let right = transform.right();
                let forward = transform.forward();
                pan_offset.0 += (right * -total_delta.x + forward * total_delta.y) * sensitivity.x;
            }
        }
    }
}
fn pan_perspective_camera(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut query: Query<(
        &mut PanOffset,
        &Transform,
        &CameraSensitivity,
        &CameraMode,
    ), With<Player>>,
) {
    let (mut pan_offset, transform, sensitivity, camera_mode) = match query.single_mut() {
        Ok(values) => values,
        Err(_) => return,
    };

    if *camera_mode != CameraMode::Orthographic2D {
        let mut movement = Vec3::ZERO;
        if keys.pressed(PAN_UP_KEY) {
            movement += Vec3::new(transform.forward().x, transform.forward().y, transform.forward().z);
        }
        if keys.pressed(PAN_DOWN_KEY) {
            movement -= Vec3::new(transform.forward().x, transform.forward().y, transform.forward().z);
        }
        if keys.pressed(PAN_RIGHT_KEY) {
            movement += Vec3::new(transform.right().x, transform.right().y, transform.right().z);
        }
        if keys.pressed(PAN_LEFT_KEY) {
            movement -= Vec3::new(transform.right().x, transform.right().y, transform.right().z);
        }

        if movement != Vec3::ZERO {
            let delta = movement.normalize_or_zero() * PAN_SPEED;
            pan_offset.0 += Vec3::new(delta.x, 0.0, delta.z);
        }

        if mouse_buttons.pressed(MOUSE_PAN_BUTTON) {
            let total_delta = mouse_motion.delta;
            let right = transform.right();
            let forward = transform.forward();
            pan_offset.0 += (right * -total_delta.x + forward * total_delta.y) * sensitivity.x;
        }
    }
}

fn orbit_ortho_camera(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(
        &mut Transform,
        &CameraSensitivity,
        &mut OrbitControl,
        &PanOffset,
        &CameraMode,
    ), With<Player>>,
) {
    let Ok((mut transform, sensitivity, mut orbit, pan_offset, camera_mode)) = query.single_mut() else {
        return;
    };

    // Update angles only in 3D modes
    match camera_mode {
        CameraMode::Perspective3D | CameraMode::Orthographic3D => {
            if mouse_buttons.pressed(MOUSE_ORBIT_BUTTON) {
                let delta = accumulated_mouse_motion.delta;
                orbit.yaw -= delta.x * sensitivity.x;
                orbit.pitch -= delta.y * sensitivity.y;
            }

            if keyboard_input.pressed(ORBIT_LEFT_KEY) {
                orbit.yaw += 0.02;
            }
            if keyboard_input.pressed(ORBIT_RIGHT_KEY) {
                orbit.yaw -= 0.02;
            }

            const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;
            orbit.pitch = orbit.pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);
        }
        CameraMode::Orthographic2D => {
            // Reset angles for 2D mode to ensure top-down view
            orbit.yaw = 0.0;
            orbit.pitch = 0.0;
        }
    }

    // Update camera position based on current mode
    let target = pan_offset.0;
    match camera_mode {
        CameraMode::Orthographic2D => {
            // Strict top-down view in 2D mode
            transform.translation = target + Vec3::Y * orbit.radius;
            transform.look_at(target, Vec3::Y);
        }
        CameraMode::Perspective3D | CameraMode::Orthographic3D => {
            let x = orbit.radius * orbit.pitch.cos() * orbit.yaw.cos();
            let y = orbit.radius * orbit.pitch.sin();
            let z = orbit.radius * orbit.pitch.cos() * orbit.yaw.sin();
            transform.translation = target + Vec3::new(x, y, z);
            transform.look_at(target, Vec3::Y);
        }
    }
}

fn orbit_perspective_camera(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(
        &mut Transform,
        &CameraSensitivity,
        &mut OrbitControl,
        &PanOffset,
        &CameraMode,
    ), With<Player>>,
) {
    let Ok((mut transform, sensitivity, mut orbit, pan_offset, camera_mode)) = query.single_mut() else {
        return;
    };

    if *camera_mode != CameraMode::Orthographic2D {
        if mouse_buttons.pressed(MOUSE_ORBIT_BUTTON) {
            let delta = accumulated_mouse_motion.delta;
            orbit.yaw -= delta.x * sensitivity.x;
            orbit.pitch -= delta.y * sensitivity.y;
        }

        if keyboard_input.pressed(ORBIT_LEFT_KEY) {
            orbit.yaw += 0.02;
        }
        if keyboard_input.pressed(ORBIT_RIGHT_KEY) {
            orbit.yaw -= 0.02;
        }

        const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;
        orbit.pitch = orbit.pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);

        let target = pan_offset.0;
        let x = orbit.radius * orbit.pitch.cos() * orbit.yaw.cos();
        let y = orbit.radius * orbit.pitch.sin();
        let z = orbit.radius * orbit.pitch.cos() * orbit.yaw.sin();
        transform.translation = target + Vec3::new(x, y, z);
        transform.look_at(target, Vec3::Y);
    }
}

fn zoom_ortho_camera(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Projection, With<Player>>,
) {
    let mut projection = match query.single_mut() {
        Ok(p) => p,
        Err(_) => return,
    };

    let Projection::Orthographic(ortho) = &mut *projection else {
        return;
    };

    // Zoom in
    if keyboard_input.pressed(ZOOM_IN_KEY) {
        ortho.scale *= 1.0 - ZOOM_SPEED;
        ortho.scale = ortho.scale.max(MIN_ZOOM);
    }

    // Zoom out
    if keyboard_input.pressed(ZOOM_OUT_KEY) {
        ortho.scale *= 1.0 + ZOOM_SPEED;
        ortho.scale = ortho.scale.min(MAX_ZOOM);
    }
}

fn zoom_perspective_camera(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Projection, With<Player>>,
) {
    let mut projection = match query.single_mut() {
        Ok(p) => p,
        Err(_) => return,
    };

    let Projection::Perspective(perspective) = &mut *projection else {
        return;
    };

    if keyboard_input.pressed(ZOOM_IN_KEY) {
        perspective.fov -= ZOOM_SPEED;
        perspective.fov = perspective.fov.max(MIN_ZOOM);
    }

    if keyboard_input.pressed(ZOOM_OUT_KEY) {
        perspective.fov += ZOOM_SPEED;
        perspective.fov = perspective.fov.min(MAX_ZOOM);
    }
}

fn toggle_camera_mode(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut CameraMode, &mut Projection), With<Player>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        if let Ok((mut mode, mut projection)) = query.single_mut() {
            *mode = match *mode {
                CameraMode::Perspective3D => CameraMode::Orthographic3D,
                CameraMode::Orthographic3D => CameraMode::Orthographic2D,
                CameraMode::Orthographic2D => CameraMode::Perspective3D,
            };
            *projection = match *mode {
                CameraMode::Perspective3D => Projection::Perspective(PerspectiveProjection {
                    fov: FRAC_PI_2,
                    ..default()
                }),
                CameraMode::Orthographic3D => Projection::Orthographic(OrthographicProjection {
                    scaling_mode: ScalingMode::FixedVertical {
                        viewport_height: 6.0,
                    },
                    ..OrthographicProjection::default_3d()
                }),
                CameraMode::Orthographic2D => Projection::Orthographic(OrthographicProjection {
                    scaling_mode: ScalingMode::FixedVertical {
                        viewport_height: 6.0,
                    },
                    ..OrthographicProjection::default_3d()
                }),
            };
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