use crate::simcore::types::{Simulation, JointId, Joint, Constraint, 
    FixedPositionConstraint, DistanceConstraint};
use generational_arena::Arena;

use std::any::Any;

impl Simulation {
    // Step simulation forward by dt using a fixed number of solver iterations
    pub fn step(&mut self, dt: f32, iterations: usize) {
        for _ in 0..iterations {
            self.solve_constraints();
            // TODO: integrate velocities, forces, etc., if needed
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

    // Safely get two distinct mutable joints
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

// Implement Constraint trait for FixedPositionConstraint
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

// Implement Constraint trait for DistanceConstraint
impl Constraint for DistanceConstraint {
    fn apply(&self, sim: &mut Simulation) {
        if let Some((joint_a, joint_b)) = sim.get_two_joints_mut(self.joint_a, self.joint_b) {
            let delta = joint_b.position - joint_a.position;
            let current_distance = delta.length();
            let error = current_distance - self.target_distance;

            if error.abs() > 1e-6 && current_distance > 0.0 {
                let correction = (error * 0.5) * delta.normalize();
                joint_a.position += correction;
                joint_b.position -= correction;
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
