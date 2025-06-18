use crate::dsl::ast::*;
use crate::simcore::types::*;
use crate::simcore::bindings::*;
use std::collections::HashMap;
use glam::Vec3;

pub struct DslCompiler;

impl DslCompiler {
    pub fn compile_to_simulation(program: Program) -> Result<Simulation, String> {
        let mut sim = Simulation::default();
        let mut joint_name_to_id = HashMap::new(); // This is the local HashMap we'll actually use
        
        // First pass: Create all joints
        for joint_decl in &program.joints {

            let position = Position::Vec3(Vec3::new(
                joint_decl.position[0],
                joint_decl.position[1],
                joint_decl.position[2],
            ));
            
            let joint_id = sim.joints.insert(Joint {
                position,
                joint_type: JointType::Revolute, // Default, could be specified in DSL
                connected_links: Vec::new(),
            });
            
            // Store in BOTH HashMaps
            joint_name_to_id.insert(joint_decl.name.clone(), joint_id);
        }
        
        // Second pass: Create links and update joint connections
        let mut link_name_to_id = HashMap::new();
        for link_decl in &program.links {
            let joint_a_id = joint_name_to_id.get(&link_decl.joint_a)
                .ok_or_else(|| format!("Joint '{}' not found", link_decl.joint_a))?;
            let joint_b_id = joint_name_to_id.get(&link_decl.joint_b)
                .ok_or_else(|| format!("Joint '{}' not found", link_decl.joint_b))?;
            
            let link_id = sim.links.insert(Link {
                joints: vec![*joint_a_id, *joint_b_id],
                rigid: true,
            });
            
            // Update joint connections
            sim.joints.get_mut(*joint_a_id).unwrap().connected_links.push(link_id);
            sim.joints.get_mut(*joint_b_id).unwrap().connected_links.push(link_id);
            
            link_name_to_id.insert(link_decl.name.clone(), link_id);

        }
        
        // Third pass: Create explicit constraints type shittt
        for constraint_decl in &program.constraints {
            match constraint_decl {
                ConstraintDecl::Distance { a, b, value } => {
                    apply_distance(&mut sim, &joint_name_to_id, a, b, *value)?;
                }
                ConstraintDecl::Fixed { joints } => {
                    apply_fixed(&mut sim, &joint_name_to_id, joints)?;
                }
                ConstraintDecl::Plane { joints, normal, point } => {
                    apply_plane(&mut sim, &joint_name_to_id, joints, *normal, *point)?;
                }
                ConstraintDecl::PrismaticVector { joints, axis, origin } => {
                    apply_prismatic_vector(&mut sim, &joint_name_to_id, joints, *axis, *origin)?;
                }
                ConstraintDecl::PrismaticLink { joints, link, origin } => {
                    apply_prismatic_link(&mut sim, &joint_name_to_id, &link_name_to_id, joints, link, *origin)?;
                }
                ConstraintDecl::FixedAngle { joint_a, pivot, joint_c, angle } => {
                    apply_fixed_angle(&mut sim, &joint_name_to_id, joint_a, pivot, joint_c, *angle)?;
                }
                ConstraintDecl::Revolute { joint_a, joint_b, axis, min_angle, max_angle } => {
                    apply_revolute(&mut sim, &joint_name_to_id, joint_a, joint_b, *axis, *min_angle, *max_angle);
                }
                
        }
        }
        
        println!("DSL Compilation complete:");
        println!("  - {} joints created", sim.joints.len());
        println!("  - {} links created", sim.links.len());
        println!("  - {} constraints created", sim.constraints.len());
        
        Ok(sim)
    }
}