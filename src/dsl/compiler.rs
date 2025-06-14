use crate::dsl::ast::*;
use crate::simcore::types::*;
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
        
        // Third pass: Create explicit constraints
        for constraint_decl in &program.constraints {
            match constraint_decl {
                ConstraintDecl::Distance { a, b, value } => {
                    let joint_a_id = joint_name_to_id.get(a)
                        .ok_or_else(|| format!("Joint '{}' not found", a))?;
                    let joint_b_id = joint_name_to_id.get(b)
                        .ok_or_else(|| format!("Joint '{}' not found", b))?;
                    
                    sim.constraints.push(Box::new(DistanceConstraint {
                        joint_a: *joint_a_id,
                        joint_b: *joint_b_id,
                        target_distance: *value,
                    }));
                }
                ConstraintDecl::Fixed { joints } => {
                    for joint_name in joints {
                        let joint_id = joint_name_to_id.get(joint_name)
                            .ok_or_else(|| format!("Joint '{}' not found", joint_name))?;
                        
                        let target_position = sim.joints.get(*joint_id).unwrap().position;
                        sim.constraints.push(Box::new(FixedPositionConstraint {
                            joint_id: *joint_id,
                            target_position,
                        }));
                    }
                }
                ConstraintDecl::Plane { joints, normal, point } => {
                    let plane_point = point.unwrap_or(Vec3::ZERO);
                    
                    for joint_name in joints {
                        let joint_id = joint_name_to_id.get(joint_name)
                            .ok_or_else(|| format!("Joint '{}' not found", joint_name))?;
                        
                        sim.constraints.push(Box::new(PlaneConstraint {
                            joint_id: *joint_id,
                            normal: *normal,
                            plane_point,
                        }));
                    }
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