pub mod util;
pub mod simcore;


use bevy::prelude::*;
use crate::util::constants::SimWrapper;
use crate::util::camera::*;
use crate::util::world::*;
use crate::util::interact::*;
use crate::util::simulation::*;

use crate::simcore::types::*;



fn main() {
    App::new()
        .add_event::<PickedJoint>()
        .add_event::<MoveJoint>()
        .add_plugins(DefaultPlugins)
        .insert_resource(CameraModeIndicator::default())
        .insert_resource(SelectedJoint::default())
        .add_systems(
            Startup,
            (
                spawn_view_model,
                spawn_world_model,
                spawn_lights,
                spawn_text,
                setup_sim,
                render_sim
            ).chain(),
        )
        .add_systems(Update, camera_control_system)
        .add_systems(Update, (update_camera_mode_indicator, update_camera_mode_text))
        .add_systems(Update, (
            interact_system, 
            highlight_system,  
            reset_on_release_system,
            joint_drag_system,
            sim_step_system,
            update_joint_visuals.after(sim_step_system),
            update_link_visuals.after(sim_step_system),
        ))
        .run();
}



fn setup_sim(mut commands: Commands) {
    let mut sim = Simulation::default();
    let joint_pos_a = Position::Vec3(glam::Vec3::new(0.0, 0.0, 0.0));
    let joint_pos_b = Position::Vec3(glam::Vec3::new(2.0, 0.0, 0.0));
    let joint_pos_c = Position::Vec3(glam::Vec3::new(2.0, 0.0, 2.0));
    let joint_pos_d = Position::Vec3(glam::Vec3::new(0.0, 0.0, 2.0));
    
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
        joint_id: joint_b,
        target_position: joint_pos_b,
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
    let plane = glam::Vec3::Y;
    sim.constraints.push(Box::new(PlaneConstraint {
        joint_id: joint_a,
        normal: plane,
        plane_point: glam::Vec3::ZERO,
    }));
    sim.constraints.push(Box::new(PlaneConstraint {
        joint_id: joint_b,
        normal: plane,
        plane_point: glam::Vec3::ZERO,
    }));
    sim.constraints.push(Box::new(PlaneConstraint {
        joint_id: joint_c,
        normal: plane,
        plane_point: glam::Vec3::ZERO,
    }));
    sim.constraints.push(Box::new(PlaneConstraint {
        joint_id: joint_d,
        normal: plane,
        plane_point: glam::Vec3::ZERO,
    }));

    commands.insert_resource(SimWrapper { sim });
}


