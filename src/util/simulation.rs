
use bevy::prelude::*;
use crate::util::interact::MoveJoint;
use crate::util::constants::*;
use crate::util::keybindings::KeyBindings;
use crate::simcore::types::*;

//render
pub fn render_sim(
    sim_wrapper: ResMut<SimWrapper>,
    joint_query: Query<Entity, With<JointWrapper>>,
    link_query: Query<Entity, With<LinkWrapper>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    
) {

    for entity in joint_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in link_query.iter() {
        commands.entity(entity).despawn();
    }

    let sim = &sim_wrapper.sim;

    let joint_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 0.0),
        ..Default::default()
    });

    let link_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.0, 1.0),
        ..Default::default()
    });

    let joint_mesh = meshes.add(Sphere::new(0.1).mesh());


    let link_mesh = meshes.add(Cuboid::new(0.05, 0.05, 1.0));

    for (joint_id, joint) in &sim.joints {
        let joint_position = match joint.position {
            Position::Vec2(v) => Vec3::new(v.x, 0.0, v.y),
            Position::Vec3(v) => Vec3::new(v.x, v.y, v.z),
        };
        commands.spawn((
            Mesh3d(joint_mesh.clone()),
            MeshMaterial3d(joint_material.clone()),
            Transform::from_translation(joint_position),
            JointWrapper { joint_id: joint_id },
        ));
    }

    for link in sim.links.iter() {
        let (_, link_data) = link;
        if link_data.joints.len() == 2 {
            let joint_a = sim.joints.get(link_data.joints[0]).unwrap();
            let joint_b = sim.joints.get(link_data.joints[1]).unwrap();

            let start = match joint_a.position {
                Position::Vec2(v) => Vec3::new(v.x, 0.0, v.y),
                Position::Vec3(v) => Vec3::new(v.x, v.y, v.z),
            };
            let end = match joint_b.position {
                Position::Vec2(v) => Vec3::new(v.x, 0.0, v.y),
                Position::Vec3(v) => Vec3::new(v.x, v.y, v.z),
            };
            let mid = (start + end) / 2.0;

            let direction = end - start;
            let length = direction.length();
            let rotation = Quat::from_rotation_arc(Vec3::Z, direction.normalize());

            commands.spawn((
                Mesh3d(link_mesh.clone()),
                MeshMaterial3d(link_material.clone()),
                Transform {
                    translation: mid,
                    rotation,
                    scale: Vec3::new(0.05, 0.05, length),
                },
                LinkWrapper {
                    link_id: link.0,
                },
            ));
        }
    }
}


pub fn update_link_visuals(
    sim_wrapper: Res<SimWrapper>,
    mut link_query: Query<(&mut Transform, &LinkWrapper)>,
) {
    let sim = &sim_wrapper.sim;
    
    for (mut transform, link_wrapper) in link_query.iter_mut() {
        if let Some(link) = sim.links.get(link_wrapper.link_id) {
            if link.joints.len() == 2 {
                let joint_a = sim.joints.get(link.joints[0]).unwrap();
                let joint_b = sim.joints.get(link.joints[1]).unwrap();

                let start = joint_a.position.as_vec3();
                let end = joint_b.position.as_vec3();
                let mid = (start + end) / 2.0;

                let direction = end - start;
                let length = direction.length();
                
                // Only update if we have a valid length
                if length > 0.001 {
                    let rotation = glam::Quat::from_rotation_arc(glam::Vec3::Z, direction.normalize());
                    
                    transform.translation = bevy::prelude::Vec3::new(mid.x, mid.y, mid.z);
                    transform.rotation = bevy::prelude::Quat::from_xyzw(rotation.x, rotation.y, rotation.z, rotation.w);
                    transform.scale = Vec3::new(0.05, 0.05, length);
                }
            }
        }
    }
}

pub fn update_joint_visuals(
    sim_wrapper: Res<SimWrapper>,
    mut joint_query: Query<(&mut Transform, &JointWrapper)>,
) {
    let sim = &sim_wrapper.sim;
    
    for (mut transform, joint_wrapper) in joint_query.iter_mut() {
        if let Some(joint) = sim.joints.get(joint_wrapper.joint_id) {
            let pos = joint.position.as_vec3();
            transform.translation = bevy::prelude::Vec3::new(pos.x, pos.y, pos.z);
        }
    }
}



pub fn sim_step_system(
    mut wrapper: ResMut<SimWrapper>,
    bindings: Res<KeyBindings>,
    move_events: EventReader<MoveJoint>,


) {
    // Only run simulation step if there were joint movements
    if !move_events.is_empty() {
        wrapper.sim.step(0.0, bindings.iterations_per_time_step);
    }
}

