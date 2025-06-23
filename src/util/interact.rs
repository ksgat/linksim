
use bevy::{
    prelude::*, 
    ecs::event::EventWriter,

};

use crate::util::constants::*;
use crate::simcore::types::*;
use crate::util::camera::{Player, CameraController};

#[derive(Event)]
pub struct PickedJoint {
    entity: Entity,
}


#[derive(Component)]
pub struct Selected;




#[derive(Default, Resource)]
pub struct SelectedJoint(Option<Entity>);
#[derive(Event)]
pub struct MoveJoint {
    pub joint_id: JointId,
    pub new_position: Position, 
}



pub fn interact_system(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<Player>>,
    joints: Query<(Entity, &Transform), With<JointWrapper>>, 
    mut picked_joints: EventWriter<PickedJoint>,
) {
    // Only fire on click
    if mouse_buttons.just_pressed(MouseButton::Left) {
        let window = match windows.single() {
            Ok(w) => w,
            _ => return, // bail if no window
        };

        let cursor_pos = match window.cursor_position() {
            Some(pos) => pos,
            None => return, // bail if no cursor
        };

        let (camera, camera_transform) = match cameras.single() {
            Ok(pair) => pair,
            _ => return, // bail if no camera
        };

        let ray = match camera.viewport_to_world(camera_transform, cursor_pos) {
            Ok(ray) => ray,
            _ => return, // bail if ray failed
        };

        // Track closest joint
        let mut closest: Option<(Entity, f32)> = None;

        for (entity, transform) in joints.iter() {
            let joint_pos = transform.translation;

            // Compute shortest distance from point to ray in 3D space
            let to_point = joint_pos - ray.origin;
            let projected_length = to_point.dot(ray.direction.as_vec3().normalize());
            let closest_point_on_ray = ray.origin + projected_length * ray.direction;
            let distance = joint_pos.distance(closest_point_on_ray);

            // If within threshold and closer than current, record it
            if distance < 0.2 {
                match closest {
                    Some((_, prev_dist)) if distance < prev_dist => {
                        closest = Some((entity, distance));
                    }
                    None => {
                        closest = Some((entity, distance));
                    }
                    _ => {}
                }
            }
        }

        // If a joint was found, emit event
        if let Some((entity, _)) = closest {
            picked_joints.write(PickedJoint { entity });
        }
    }
}

pub fn highlight_system(
    mut events: EventReader<PickedJoint>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut joint_query: Query<(Entity, &mut MeshMaterial3d<StandardMaterial>), With<JointWrapper>>,
    mut selected_joint: ResMut<SelectedJoint>,
) {
    for event in events.read() {
        // Reset old selected joint to yellow by cloning its material
        if let Some(old_entity) = selected_joint.0 {
            if let Ok((_, mut material_handle)) = joint_query.get_mut(old_entity) {
                if let Some(old_mat) = materials.get(&material_handle.0) {
                    let mut new_mat = old_mat.clone();
                    new_mat.base_color = Color::srgb(1.0, 1.0, 0.0); // Yellow
                    let new_handle = materials.add(new_mat);
                    material_handle.0 = new_handle;
                }
                commands.entity(old_entity).remove::<Selected>();
            }
        }

        // Highlight new selected joint with a cloned red material
        if let Ok((_, mut material_handle)) = joint_query.get_mut(event.entity) {
            if let Some(old_mat) = materials.get(&material_handle.0) {
                let mut new_mat = old_mat.clone();
                new_mat.base_color = Color::srgb(1.0, 0.0, 0.0); // Red
                let new_handle = materials.add(new_mat);
                material_handle.0 = new_handle;
            }
            commands.entity(event.entity).insert(Selected);
            selected_joint.0 = Some(event.entity);
        }
    }
}

pub fn reset_on_release_system(
        mouse_buttons: Res<ButtonInput<MouseButton>>,
        mut commands: Commands,
        mut materials: ResMut<Assets<StandardMaterial>>,
        joint_query: Query<(Entity, &MeshMaterial3d<StandardMaterial>), With<Selected>>,
    ) {
        if mouse_buttons.just_released(MouseButton::Left) {
            for (entity, material) in joint_query.iter() {
                if let Some(mat) = materials.get_mut(&material.0) {
                    mat.base_color = Color::srgb(1.0, 1.0, 0.0); // Original yellow
                }
                commands.entity(entity).remove::<Selected>();
            }
        }
}


pub fn joint_drag_system(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform, &CameraController), With<Player>>,
    selected_joints: Query<&JointWrapper, With<Selected>>,
    mut sim_wrapper: ResMut<SimWrapper>,
    mut move_joint_events: EventWriter<MoveJoint>,
) {
    if !mouse_buttons.pressed(MouseButton::Left) {
        return;
    }

    let window = match windows.single() {
        Ok(w) => w,
        _ => return,
    };

    let cursor_pos = match window.cursor_position() {
        Some(pos) => pos,
        None => return,
    };

    let (camera, camera_transform, _) = match cameras.single() {
        Ok((cam, trans, _)) => (cam, trans, ()),
        _ => return,
    };

    let ray = match camera.viewport_to_world(camera_transform, cursor_pos) {
        Ok(ray) => ray,
        _ => return,
    };

    if selected_joints.is_empty() {
        return;
    }

    // Get camera's forward direction (negated because cameras look down -Z)
    let camera_forward = -camera_transform.forward();
    let plane_normal = camera_forward.normalize();

    for joint_wrapper in selected_joints.iter() {
        if let Some(joint) = sim_wrapper.sim.joints.get_mut(joint_wrapper.joint_id) {
            // Get current joint position
            let current_joint_pos = match &joint.position {
                Position::Vec3(pos) => *pos,
                // Handle other position types if needed
                _ => continue,
            };

            let plane_distance = current_joint_pos.dot(glam::Vec3::new(plane_normal.x, plane_normal.y, plane_normal.z));

            let denom = ray.direction.dot(plane_normal);
            if denom.abs() > 1e-6 {
                let t = (plane_distance - ray.origin.dot(plane_normal)) / denom;
                let intersection = ray.origin + ray.direction * t;
                
                let new_pos = glam::Vec3::new(intersection.x, intersection.y, intersection.z);
                joint.position = Position::Vec3(new_pos);
                move_joint_events.write(MoveJoint {
                    joint_id: joint_wrapper.joint_id,
                    new_position: Position::Vec3(new_pos),
                });
            }
        }
    }
}