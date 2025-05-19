use generational_arena::{Arena as GenArena, Index};
use glam::Vec2;
use std::any::Any;

pub type JointId = Index;
pub type LinkId = Index;

#[derive(Debug)]
pub struct Simulation {
    pub joints: GenArena<Joint>,
    pub links: GenArena<Link>,
    pub constraints: Vec<Box<dyn Constraint>>,
}

#[derive(Debug, Clone)]
pub struct Joint {
    pub position: Vec2,
    pub joint_type: JointType,
    pub connected_links: Vec<LinkId>,
}

#[derive(Debug, Clone)]
pub enum JointType {
    Fixed,
    Revolute,
    Slider { axis: Vec2 },
}

#[derive(Debug, Clone)]
pub struct Link {
    pub joints: Vec<JointId>,
    pub rigid: bool,
}


pub trait Constraint: std::fmt::Debug + Any + 'static {
    fn apply(&self, sim: &mut Simulation);
    fn is_satisfied(&self, sim: &Simulation) -> bool;
    fn as_any(&self) -> &dyn Any;


}


#[derive(Debug, Clone)]
pub struct FixedPositionConstraint {
    pub joint_id: JointId,
    pub target_position: Vec2,
}

#[derive(Debug, Clone)]
pub struct DistanceConstraint {
    pub joint_a: JointId,
    pub joint_b: JointId,
    pub target_distance: f32,
}