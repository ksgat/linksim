use crate::simcore::types::*; 
use glam::{Vec3, Vec2};
use std::any::Any;

use crate::simcore::bindings::apply_distance;
impl Simulation {
    pub fn step(&mut self, dt: f32, iterations: usize) {
        let iterations = iterations * 2;
        for _ in 0..iterations {
            self.solve_constraints();
        }
    }

    // Solve all constraints once
    pub fn solve_constraints(&mut self) {
        // Take constraints out temporarily to avoid borrow conflicts
        let constraints = std::mem::take(&mut self.constraints);

        for constraint in &constraints {
            constraint.apply(self);
        }

        // Put constraints back
        self.constraints = constraints;
    }
    pub fn get_two_joints_mut(
        &mut self,
        a: JointId,
        b: JointId,
    ) -> Option<(&mut Joint, &mut Joint)> {
        if a == b {
            None
        } else {
            match self.joints.get2_mut(a, b) {
                (Some(joint_a), Some(joint_b)) => Some((joint_a, joint_b)),
                _ => None,
            }
        }
    }
}

impl Constraint for FixedPositionConstraint {
    fn apply(&self, sim: &mut Simulation) {
        if let Some(joint) = sim.joints.get_mut(self.joint_id) {
            joint.position = self.target_position;
        }
    }

    fn is_satisfied(&self, sim: &Simulation) -> bool {
        sim.joints.get(self.joint_id)
            .map(|joint| joint.position.abs_diff_eq(self.target_position, 1e-6))
            .unwrap_or(false)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Constraint for DistanceConstraint {
    fn apply(&self, sim: &mut Simulation) {
        if let Some((joint_a, joint_b)) = sim.get_two_joints_mut(self.joint_a, self.joint_b) {
            let delta = joint_b.position.sub(joint_a.position);
            let current_distance = delta.length();
            let error: f32 = current_distance - self.target_distance;

            if error.abs() > 1e-6 && current_distance > 0.0 {
                let correction = delta.normalize().scale(error * 0.5);
                joint_a.position = joint_a.position.add(correction);
                joint_b.position = joint_b.position.sub(correction);
            }
        }
    }

    fn is_satisfied(&self, sim: &Simulation) -> bool {
        match (sim.joints.get(self.joint_a), sim.joints.get(self.joint_b)) {
            (Some(a), Some(b)) => {
                let dist = a.position.distance(b.position);
                (dist - self.target_distance).abs() < 1e-6
            }
            _ => false,
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Constraint for PlaneConstraint {
    fn apply(&self, sim: &mut Simulation) {
        if let Some(joint) = sim.joints.get_mut(self.joint_id) {
            let position = joint.position.as_vec3();
            let normal = self.normal;
            let to_plane = position - self.plane_point;
            let distance_to_plane = to_plane.dot(normal);
            let projection = position - normal * distance_to_plane;

            joint.position = Position::Vec3(projection);
        }
    }

    fn is_satisfied(&self, sim: &Simulation) -> bool {
        if let Some(joint) = sim.joints.get(self.joint_id) {
            let position = joint.position.as_vec3();
            let to_plane = position - self.plane_point;
            (to_plane.dot(self.normal)).abs() < 1e-6
        } else {
            false
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Constraint for PrismaticConstraintVector {
    fn apply(&self, sim: &mut Simulation) {
        if let Some(joint) = sim.joints.get_mut(self.joint_id) {
            let axis_dir = self.axis.normalize();
            let joint_pos = joint.position.as_vec3();
            let to_joint = joint_pos - self.origin;
            let proj_length = to_joint.dot(axis_dir);
            let projected_pos = self.origin + axis_dir * proj_length;
            joint.position = Position::Vec3(projected_pos);
        }
    }

    fn is_satisfied(&self, sim: &Simulation) -> bool {
        if let Some(joint) = sim.joints.get(self.joint_id) {
            let axis_dir = self.axis.normalize();
            let joint_pos = joint.position.as_vec3();
            let to_joint = joint_pos - self.origin;
            let proj_length = to_joint.dot(axis_dir);
            let projected_pos = self.origin + axis_dir * proj_length;
            let dist = (joint_pos - projected_pos).length();
            dist < 1e-6
        } else {
            false
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Constraint for PrismaticConstraintLink {
    fn apply(&self, sim: &mut Simulation) {
        // Get the axis from the link
        let axis_vec = match self.get_link_axis(sim) {
            Some(axis) => axis,
            None => return,
        };
        
        // Apply the constraint using the calculated axis
        let prismatic_vec = PrismaticConstraintVector {
            joint_id: self.joint_id,
            origin: self.origin,
            axis: axis_vec,
        };
        prismatic_vec.apply(sim);
    }
    fn is_satisfied(&self, sim: &Simulation) -> bool {
        // Get the axis from the link (same logic as apply)
        let axis_vec = match self.get_link_axis(sim) {
            Some(axis) => axis,
            None => return false,
        };
        
        // Create the equivalent PrismaticConstraintVector and check if it's satisfied
        let prismatic_vec = PrismaticConstraintVector {
            joint_id: self.joint_id,
            origin: self.origin,
            axis: axis_vec,
        };
        prismatic_vec.is_satisfied(sim)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl PrismaticConstraintLink {
    fn get_link_axis(&self, sim: &Simulation) -> Option<Vec3> {
        let link = sim.links.get(self.link_id)?;
        if link.joints.len() != 2 {
            return None;
        }
        let joint_a = sim.joints.get(link.joints[0])?;
        let joint_b = sim.joints.get(link.joints[1])?;
        Some(joint_b.position.as_vec3() - joint_a.position.as_vec3())
    }
}
//broken as shit bro
impl Constraint for FixedAngleConstraint { 
    fn apply(&self, sim: &mut Simulation) {
        let pivot_pos = if let Some(pivot_joint) = sim.joints.get(self.pivot_joint_id) {
            pivot_joint.position
        } else {
            return;
        };
    
        if let Some((joint_a, joint_b)) = sim.get_two_joints_mut(self.joint_a_id, self.joint_b_id) {
            let r_a = (joint_a.position.sub(pivot_pos)).length();
            let r_b = (joint_b.position.sub(pivot_pos)).length();
        
            // Debug: Print current distances
            println!("r_a: {}, r_b: {}", r_a, r_b);
        
            let target_distance = (r_a.powi(2) + r_b.powi(2) - 2.0 * r_a * r_b * self.target_angle.cos()).sqrt();
            
            let distance_constraint = DistanceConstraint{
                joint_a: self.joint_a_id,
                joint_b: self.joint_b_id,
                target_distance:target_distance,
            };

            distance_constraint.apply(sim);
        }
    }
    fn is_satisfied(&self, sim: &Simulation) -> bool {
        let pivot: Option<&Joint> = sim.joints.get(self.pivot_joint_id);
        let joint_a = sim.joints.get(self.joint_a_id);
        let joint_b = sim.joints.get(self.joint_b_id);
    
        if let (Some(pivot), Some(joint_a), Some(joint_b)) = (pivot, joint_a, joint_b) {
            let pivot_pos = pivot.position.as_vec3();
            let a_pos = joint_a.position.as_vec3();
            let b_pos = joint_b.position.as_vec3();
    
            let r_a = (a_pos - pivot_pos).length();
            let r_b = (b_pos - pivot_pos).length();
            let target_dist = (r_a.powi(2) + r_b.powi(2) - 2.0 * r_a * r_b * self.target_angle.cos()).sqrt();
    
            let actual_dist = (b_pos - a_pos).length();
            let epsilon = 1e-4;
    
            (actual_dist - target_dist).abs() < epsilon
        } else {
            false
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

}


impl Constraint for RevoluteConstraint {
    fn apply(&self, sim: &mut Simulation) {
        let (pivot, moving) = match sim.get_two_joints_mut(self.pivot_joint_id, self.moving_joint_id) {
            Some((a, b)) => (a, b),
            None => return,
        };

        let pivot_pos = pivot.position.as_vec3();
        let moving_pos = moving.position.as_vec3();

        let current_vec = (moving_pos - pivot_pos).normalize();
        let rest_vec = self.rest_direction.normalize();

        // Rotation axis = plane normal
        let plane_normal = rest_vec.cross(current_vec).normalize_or_zero();
        if plane_normal.length_squared() == 0.0 {
            return; // degenerate case: no clear rotation plane
        }

        let dot = rest_vec.dot(current_vec).clamp(-1.0, 1.0);
        let angle = dot.acos();

        // Signed angle: cross product gives direction
        let cross = rest_vec.cross(current_vec);
        let signed_angle = if cross.dot(plane_normal) < 0.0 { -angle } else { angle };

        // Check bounds
        if signed_angle < self.min_angle || signed_angle > self.max_angle {
            let clamped_angle = signed_angle.clamp(self.min_angle, self.max_angle);

            // Rotate rest_vec to target angle in plane of plane_normal
            let rotated_vec = rotate_vec_in_plane(rest_vec, plane_normal, clamped_angle);

            // Maintain original length
            let dist = (moving_pos - pivot_pos).length();
            moving.position = Position::Vec3(pivot_pos + rotated_vec * dist);
        }
    }   

    fn is_satisfied(&self, sim: &Simulation) -> bool {
        
        let pivot = sim.joints.get(self.pivot_joint_id).unwrap();
        let moving = sim.joints.get(self.moving_joint_id).unwrap();
        
        let pivot_pos = pivot.position.as_vec3();
        let moving_pos = moving.position.as_vec3();

        let current_vec = (moving_pos - pivot_pos).normalize();
        let rest_vec = self.rest_direction.normalize();

        // Rotation axis = plane normal
        let plane_normal = rest_vec.cross(current_vec).normalize_or_zero();
        if plane_normal.length_squared() == 0.0 {
            return false; // degenerate case: no clear rotation plane
        }

        let dot = rest_vec.dot(current_vec).clamp(-1.0, 1.0);
        let angle = dot.acos();

        // Signed angle: cross product gives direction
        let cross = rest_vec.cross(current_vec);
        let signed_angle = if cross.dot(plane_normal) < 0.0 { -angle } else { angle };
        signed_angle >= self.min_angle - 1e-5 && signed_angle <= self.max_angle + 1e-5

        
    }
    fn as_any(&self) -> &dyn Any {
        self
    } 
    }
    
   
/// Rotate a vector by a given angle in the plane defined by a normal.
/// Uses Rodrigues' rotation formula (but no quats).
fn rotate_vec_in_plane(vec: Vec3, normal: Vec3, angle: f32) -> Vec3 {
    let cos = angle.cos();
    let sin = angle.sin();
    vec * cos + normal.cross(vec) * sin + normal * normal.dot(vec) * (1.0 - cos)
}
