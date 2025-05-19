mod simcore;

use crate::simcore::types::{Simulation, Joint, Link, FixedPositionConstraint, DistanceConstraint};
use glam::Vec2;

fn sanity_check(sim: &Simulation) {
    for link in sim.links.iter() {
        for &joint_id in &link.1.joints {
            if sim.joints.get(joint_id).is_none() {
                panic!("Link {:?} references non-existent joint {:?}", link, joint_id);
            }
        }
    }

    for constraint in &sim.constraints {
        if let Some(fixed) = constraint.as_any().downcast_ref::<FixedPositionConstraint>() {
            if sim.joints.get(fixed.joint_id).is_none() {
                panic!(
                    "Constraint references non-existent joint: {:?}",
                    fixed.joint_id
                );
            }
        } else if let Some(dist) = constraint.as_any().downcast_ref::<DistanceConstraint>() {
            if sim.joints.get(dist.joint_a).is_none()
                || sim.joints.get(dist.joint_b).is_none()
            {
                panic!(
                    "Constraint references non-existent joints: {:?}, {:?}",
                    dist.joint_a, dist.joint_b
                );
            }
        }
    }
    

    println!("Sanity check passed!");
}

fn main() {
    // Create a new simulation
    let mut sim = Simulation {
        joints: Default::default(),
        links: Default::default(),
        constraints: Vec::new(),
    };

    // Add joints
    let joint_a = Joint {
        position: Vec2::new(0.0, 0.0),
        joint_type: crate::simcore::types::JointType::Fixed,
        connected_links: Vec::new(),
    };
    let joint_b = Joint {
        position: Vec2::new(1.0, 1.0),
        joint_type: crate::simcore::types::JointType::Fixed,
        connected_links: Vec::new(),
    };
    let joint_c = Joint {
        position: Vec2::new(2.0, 2.0),
        joint_type: crate::simcore::types::JointType::Fixed,
        connected_links: Vec::new(),
    };

    let joint_a_id = sim.joints.insert(joint_a);
    let joint_b_id = sim.joints.insert(joint_b);
    let joint_c_id = sim.joints.insert(joint_c);

    // Add links
    let link_a = Link {
        joints: vec![joint_a_id],
        rigid: true,
    };
    let link_b = Link {
        joints: vec![joint_b_id],
        rigid: true,
    };

    sim.links.insert(link_a);
    sim.links.insert(link_b);

    // Debug: Print initial simulation state
    println!("Initial simulation state: {:#?}", sim);

    // Perform sanity check
    sanity_check(&sim);

    // Add constraints
    let fixed_constraint = FixedPositionConstraint {
        joint_id: joint_a_id,
        target_position: Vec2::new(1.0, 2.0),
    };

    let distance_constraint = DistanceConstraint {
        joint_a: joint_b_id,
        joint_b: joint_c_id,
        target_distance: 1.5,
    };

    sim.constraints.push(Box::new(fixed_constraint));
    sim.constraints.push(Box::new(distance_constraint));

    // Debug: Print simulation state before solving constraints
    println!("Simulation state before solving constraints: {:#?}", sim);

    // Solve constraints
    sim.step(0.016, 10);

    // Debug: Print final simulation state
    println!("Final simulation state after solving constraints: {:#?}", sim);

    // Additional tests
    println!("Testing constraints satisfaction:");
    for constraint in &sim.constraints {
        println!(
            "Constraint {:?} satisfied: {}",
            constraint,
            constraint.is_satisfied(&sim)
        );
    }
}