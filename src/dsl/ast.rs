use glam::Vec3;
pub struct Program {
    pub sim_name: String,
    pub joints: Vec<JointDecl>,
    pub links: Vec<LinkDecl>,
    pub constraints: Vec<ConstraintDecl>,
}
#[derive(Debug)]
pub struct JointDecl {
    pub name: String,
    pub position: [f32; 3],
}
#[derive(Debug)]
pub struct LinkDecl {
    pub name: String,
    pub joint_a: String,
    pub joint_b: String,
}
#[derive(Debug)]
pub enum ConstraintDecl {
    Distance { a: String, b: String, value: f32 },
    Fixed { joints: Vec<String> },
    Plane { joints: Vec<String>, normal: Vec3, point: Option<Vec3> },
    PrismaticVector { joints: Vec<String>, axis: Vec3, origin: Vec3 },
    PrismaticLink { joints: Vec<String>, link: String, origin: Vec3 },
}
impl ConstraintDecl {
    pub fn constraint_type(&self) -> &str {
        match self {
            ConstraintDecl::Distance { .. } => "Distance",
            ConstraintDecl::Fixed { .. } => "Fixed",
            ConstraintDecl::Plane { .. } => "Plane",
            ConstraintDecl::PrismaticVector { .. } => "PrismaticVector",
            ConstraintDecl::PrismaticLink { .. } => "PrismaticLink",
        }
    }
}