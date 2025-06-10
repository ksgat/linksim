use crate::simcore::types::*; // Add Position

use std::any::Any;

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
            let error = current_distance - self.target_distance;

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

