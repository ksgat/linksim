use generational_arena::{Arena as GenArena, Index};
use glam::{Vec2, Vec3}; // Add Vec3
use std::any::Any;

pub type JointId = Index;
pub type LinkId = Index;

#[derive(Debug, Default)]
pub struct Simulation {
    pub joints: GenArena<Joint>,
    pub links: GenArena<Link>,
    pub constraints: Vec<Box<dyn Constraint>>,
}

#[derive(Debug, Clone)]
pub struct Joint {
    pub position: Position, // Changed from Vec2 to Position
    pub joint_type: JointType,
    pub connected_links: Vec<LinkId>,
}

#[derive(Debug, Clone)]
pub enum JointType {
    Fixed,
    Revolute,
    Slider { axis: Vec2 },
}
#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    pub joints: Vec<JointId>,
    pub rigid: bool,
}


pub trait Constraint: std::fmt::Debug + Any + Send + Sync + 'static {
    fn apply(&self, sim: &mut Simulation);
    fn is_satisfied(&self, sim: &Simulation) -> bool;
    fn as_any(&self) -> &dyn Any;


}

// Add Position enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Position {
    Vec2(Vec2),
    Vec3(Vec3),
}

// Math helpers for Position
impl Position {
    pub fn as_vec3(&self) -> Vec3 {
        match *self {
            Position::Vec2(v) => v.extend(0.0),
            Position::Vec3(v) => v,
        }
    }
    pub fn as_vec2(&self) -> Vec2 {
        match *self {
            Position::Vec2(v) => v,
            Position::Vec3(v) => v.truncate(),
        }
    }
    pub fn add(self, rhs: Position) -> Position {
        Position::Vec3(self.as_vec3() + rhs.as_vec3())
    }
    pub fn sub(self, rhs: Position) -> Position {
        Position::Vec3(self.as_vec3() - rhs.as_vec3())
    }
    pub fn scale(self, s: f32) -> Position {
        Position::Vec3(self.as_vec3() * s)
    }
    pub fn length(&self) -> f32 {
        self.as_vec3().length()
    }
    pub fn normalize(&self) -> Position {
        Position::Vec3(self.as_vec3().normalize())
    }
    pub fn distance(&self, other: Position) -> f32 {
        (self.as_vec3() - other.as_vec3()).length()
    }
    pub fn abs_diff_eq(&self, other: Position, epsilon: f32) -> bool {
        self.as_vec3().abs_diff_eq(other.as_vec3(), epsilon)
    }
}

#[derive(Debug, Clone)]
pub struct FixedPositionConstraint {
    pub joint_id: JointId,
    pub target_position: Position, 
}

#[derive(Debug, Clone)]
pub struct DistanceConstraint {
    pub joint_a: JointId,
    pub joint_b: JointId,
    pub target_distance: f32,
}

#[derive(Debug, Clone)]
pub struct PlaneConstraint {
    pub joint_id: JointId,
    pub normal: Vec3,
    pub plane_point: Vec3,
}



#[derive(Debug, Clone)]
pub struct PrismaticConstraintVector {
    pub joint_id: JointId,
    pub axis: Vec3, //normalize the jawn
    pub origin: Vec3,
}

#[derive(Debug, Clone)]
pub struct PrismaticConstraintLink {
    pub joint_id: JointId,
    pub link_id: LinkId,
    pub origin: Vec3,

}

#[derive(Debug, Clone)]
pub struct FixedAngleConstraint {
    pub joint_a_id: JointId,    // First joint of link A (not the pivot)
    pub joint_b_id: JointId,    // First joint of link B (not the pivot)
    pub pivot_joint_id: JointId, // The shared pivot joint
    pub target_angle: f32,   // in radians
}