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


/// Sim core wrapper types
#[derive(Resource)]
pub struct SimWrapper{
    pub sim: Simulation,
}

#[derive(Component)]
pub struct JointWrapper{
    pub joint_id: JointId,
}
#[derive(Component)]
pub struct LinkWrapper{
    pub link_id: LinkId,
}
///rendering shit idrk what it does im just tweaking until it gives me that gah damn fourbar
#[derive(Debug, Component)]
struct WorldModelCamera;


#[derive(Debug, Component)]
struct Player;

#[derive(Debug, Component, Deref, DerefMut)]
struct CameraSensitivity(Vec2);

impl Default for CameraSensitivity {
    fn default() -> Self {
        Self(
            // These factors are just arbitrary mouse sensitivity values.
            // It's often nicer to have a faster horizontal sensitivity than vertical.
            // We use a component for them so that we can make them user-configurable at runtime
            // for accessibility reasons.
            // It also allows you to inspect them in an editor if you `Reflect` the component.
            Vec2::new(0.003, 0.002),
        )
    }
}


fn main(){
    App::new()
    .add_plugins(DefaultPlugins) 
    .add_systems(
        Startup,
        (
            spawn_view_model,
            spawn_world_model,
            spawn_lights,
            spawn_text,
            setup_sim, // Add the new system here
        ),
    )
    .add_systems(Update, (pan_ortho_camera, change_fov))
    .run();
}
// from bevy docs
/// Used implicitly by all entities without a `RenderLayers` component.
/// Our world model camera and all objects other than the player are on this layer.
/// The light source belongs to both layers.
const DEFAULT_RENDER_LAYER: usize = 0;

/// Used by the view model camera and the player's arm.
/// The light source belongs to both layers.
const VIEW_MODEL_RENDER_LAYER: usize = 1;






fn spawn_view_model(
    mut commands: Commands,
) {
    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 6.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        Player,
        CameraSensitivity::default(),
    ));
}
fn spawn_world_model(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let floor = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(10.0)));
    let cube = meshes.add(Cuboid::new(2.0, 0.5, 1.0));
    // i cant spell cynlider
    let tallextrudedcircle = meshes.add(Cylinder::new(0.5, 1.0).mesh().resolution(50));

    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..Default::default()
    });
    // The world model camera will render the floor and the cubes spawned in this system.
    // Assigning no `RenderLayers` component defaults to layer 0.

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
        Mesh3d(tallextrudedcircle),
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
        // The light source illuminates both the world model and the view model.
        RenderLayers::from_layers(&[DEFAULT_RENDER_LAYER, VIEW_MODEL_RENDER_LAYER]),
    ));
}

fn spawn_text(mut commands: Commands) {
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        })
        .with_child(Text::new(concat!(
            "Move the camera with your mouse.\n",
            "Press arrow up to decrease the FOV of the world model.\n",
            "Press arrow down to increase the FOV of the world model."
        )));
}

fn pan_ortho_camera(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &CameraSensitivity), With<Player>>,
) {
    let (mut transform, sensitivity) = match query.get_single_mut() {
        Ok(result) => result,
        Err(_) => return, // No camera found
    };

    // Handle mouse input for panning
    if mouse_buttons.pressed(MouseButton::Right) {
        let delta = accumulated_mouse_motion.delta;

        if delta != Vec2::ZERO {
            // Move in the camera's right and up directions
            let right = transform.rotation * Vec3::X;
            let up = transform.rotation * Vec3::Y;
            transform.translation -=
                (right * delta.x * sensitivity.x + up * delta.y * sensitivity.y) * 100.0;
        }
    }

    // Handle keyboard input for panning
    let mut movement = Vec3::ZERO;
    if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
        movement += Vec3::Y; // Move up
    }
    if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
        movement -= Vec3::Y; // Move down
    }
    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
        movement -= Vec3::X; // Move left
    }
    if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
        movement += Vec3::X; // Move right
    }

    if movement != Vec3::ZERO {
        transform.translation += movement * 0.1; // Adjust speed as needed
    }
}

/*
fn move_player(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    player: Single<(&mut Transform, &CameraSensitivity), With<Player>>,
) {
    let (mut transform, camera_sensitivity) = player.into_inner();

    let delta = accumulated_mouse_motion.delta;

    if delta != Vec2::ZERO {
        // Note that we are not multiplying by delta_time here.
        // The reason is that for mouse movement, we already get the full movement that happened since the last frame.
        // This means that if we multiply by delta_time, we will get a smaller rotation than intended by the user.
        // This situation is reversed when reading e.g. analog input from a gamepad however, where the same rules
        // as for keyboard input apply. Such an input should be multiplied by delta_time to get the intended rotation
        // independent of the framerate.
        let delta_yaw = -delta.x * camera_sensitivity.x;
        let delta_pitch = -delta.y * camera_sensitivity.y;

        let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
        let yaw = yaw + delta_yaw;

        // If the pitch was ±¹⁄₂ π, the camera would look straight up or down.
        // When the user wants to move the camera back to the horizon, which way should the camera face?
        // The camera has no way of knowing what direction was "forward" before landing in that extreme position,
        // so the direction picked will for all intents and purposes be arbitrary.
        // Another issue is that for mathematical reasons, the yaw will effectively be flipped when the pitch is at the extremes.
        // To not run into these issues, we clamp the pitch to a safe range.
        const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;
        let pitch = (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);

        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }
}
*/
fn change_fov(
    input: Res<ButtonInput<KeyCode>>,
    mut world_model_projection: Single<&mut Projection, With<WorldModelCamera>>,
) {
    let Projection::Perspective(perspective) = world_model_projection.as_mut() else {
        unreachable!(
            "The `Projection` component was explicitly built with `Projection::Perspective`"
        );
    };

    if input.pressed(KeyCode::ArrowUp) {
        perspective.fov -= 1.0_f32.to_radians();
        perspective.fov = perspective.fov.max(20.0_f32.to_radians());
    }
    if input.pressed(KeyCode::ArrowDown) {
        perspective.fov += 1.0_f32.to_radians();
        perspective.fov = perspective.fov.min(160.0_f32.to_radians());
    }
}


fn setup_sim(mut commands: Commands){
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
    
    //insert the sim into the ECS
    commands.insert_resource(SimWrapper{sim});
}


