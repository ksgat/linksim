use crate::simcore::types::*;
use std::collections::HashMap;
use glam::Vec3;



pub fn apply_distance(
    sim: &mut Simulation,
    joint_name_to_id: &HashMap<String, JointId>,
    a: &str,
    b: &str,
    value: f32,
) -> Result<(), String> {
    let joint_a_id = joint_name_to_id.get(a)
        .ok_or_else(|| format!("Joint '{}' not found", a))?;
    let joint_b_id = joint_name_to_id.get(b)
        .ok_or_else(|| format!("Joint '{}' not found", b))?;
    
    sim.constraints.push(Box::new(DistanceConstraint {
        joint_a: *joint_a_id,
        joint_b: *joint_b_id,
        target_distance: value,
    }));

    Ok(())
}

pub fn apply_fixed(
    sim: &mut Simulation,
    joint_name_to_id: &HashMap<String, JointId>,
    joints: &[String],
) -> Result<(), String> {
    for joint_name in joints {
        let joint_id = joint_name_to_id.get(joint_name)
            .ok_or_else(|| format!("Joint '{}' not found", joint_name))?;

        let target_position = sim.joints.get(*joint_id).unwrap().position;

        sim.constraints.push(Box::new(FixedPositionConstraint {
            joint_id: *joint_id,
            target_position,
        }));
    }
    Ok(())
}

pub fn apply_plane(
    sim: &mut Simulation,
    joint_name_to_id: &HashMap<String, JointId>,
    joints: &[String],
    normal: Vec3,
    point: Option<Vec3>,
) -> Result<(), String> {
    let plane_point = point.unwrap_or(Vec3::ZERO);

    for joint_name in joints {
        let joint_id = joint_name_to_id.get(joint_name)
            .ok_or_else(|| format!("Joint '{}' not found", joint_name))?;

        sim.constraints.push(Box::new(PlaneConstraint {
            joint_id: *joint_id,
            normal,
            plane_point,
        }));
    }

    Ok(())
}
//fix this broken as hell
pub fn apply_prismatic_link(
    sim: &mut Simulation,
    joint_name_to_id: &HashMap<String, JointId>,
    link_name_to_id: &HashMap<String, LinkId>,
    joints: &[String],
    link_name: &str,
    origin: Vec3,
) -> Result<(), String> {
    let link_id = link_name_to_id.get(link_name)
        .ok_or_else(|| format!("Link '{}' not found", link_name))?;

    for joint_name in joints {
        let joint_id = joint_name_to_id.get(joint_name)
            .ok_or_else(|| format!("Joint '{}' not found", joint_name))?;

        sim.constraints.push(Box::new(PrismaticConstraintLink {
            joint_id: *joint_id,
            link_id: *link_id,
            origin: origin,
        }));
    }

    Ok(()) 


}


pub fn apply_prismatic_vector(
    sim: &mut Simulation,
    joint_name_to_id: &HashMap<String, JointId>,
    joints: &[String],
    axis: Vec3,
    origin: Vec3,
) -> Result<(), String> {
    for joint_name in joints {
        let joint_id = joint_name_to_id.get(joint_name)
            .ok_or_else(|| format!("Joint '{}' not found", joint_name))?;

        sim.constraints.push(Box::new(PrismaticConstraintVector {
            joint_id: *joint_id,
            axis: axis.normalize(),
            origin,
        }));
    }

    Ok(())
}