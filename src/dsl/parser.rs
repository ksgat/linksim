use pest_derive::Parser;
use pest::Parser;
use crate::dsl::ast::*;
use pest::iterators::{Pair, Pairs};
use glam::Vec3;
#[derive(Parser)]
#[grammar = "dsl/grammar.pest"]
pub struct UgokuParser;

impl UgokuParser {
    pub fn parse_dsl(input: &str) -> Result<Program, Box<dyn std::error::Error>> {
        let pairs = Self::parse(Rule::file, input)?;
        parse_program(pairs)
    }
}

pub fn parse_program(mut pairs: Pairs<Rule>) -> Result<Program, Box<dyn std::error::Error>> {
    let file = pairs.next().unwrap();
    let program = file.into_inner().next().unwrap();
    
    let mut inner = program.into_inner();
    let sim_name = inner.next().unwrap().as_str().to_string();
    
    let mut joints = Vec::new();
    let mut links = Vec::new();
    let mut constraints = Vec::new();
    for statement in inner {
        // Get the inner Pair (joint_decl, link_decl, or constraint_decl)
        let inner_pair = statement.into_inner().next().unwrap();
        match inner_pair.as_rule() {
            Rule::joint_decl => {
                let joint = parse_joint_decl(inner_pair)?;
                println!(
                    "Parsed joint: name={}, position={:?}",
                    joint.name, joint.position
                );
                joints.push(joint);
            }
            Rule::link_decl => {
                let link = parse_link_decl(inner_pair)?;
                println!(
                    "Parsed link: name={}, joint_a={}, joint_b={}",
                    link.name, link.joint_a, link.joint_b
                );
                links.push(link);
            }
            Rule::constraint_decl => {
                let constraint = parse_constraint_decl(inner_pair)?;
                println!(
                    "Parsed constraint: type={}, value={:?}",
                    constraint.constraint_type(), constraint
                );
                constraints.push(constraint);
            }
            _ => println!("Unexpected inner rule: {:?}", inner_pair.as_rule()),
        }
    }
    
    println!(
        "Parsed program: sim_name={}, joints_count={}, links_count={}, constraints_count={}",
        sim_name,
        joints.len(),
        links.len(),
        constraints.len()
    );
    println!("Joints: {:?}", joints);
    println!("Links: {:?}", links);
    println!("Constraints: {:?}", constraints);
    
    Ok(Program {
        sim_name,
        joints,
        links,
        constraints,
    })
}
fn parse_joint_decl(pair: Pair<Rule>) -> Result<JointDecl, Box<dyn std::error::Error>> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    
    let x: f32 = inner.next().unwrap().as_str().parse()?;
    let y: f32 = inner.next().unwrap().as_str().parse()?;
    let z: f32 = inner.next().map(|p| p.as_str().parse()).transpose()?.unwrap_or(0.0);
    
    Ok(JointDecl {
        name,
        position: [x, y, z],
    })
}

fn parse_link_decl(pair: Pair<Rule>) -> Result<LinkDecl, Box<dyn std::error::Error>> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let joint_a = inner.next().unwrap().as_str().to_string();
    let joint_b = inner.next().unwrap().as_str().to_string();
    
    Ok(LinkDecl {
        name,
        joint_a,
        joint_b,
    })
}

fn parse_constraint_decl(pair: Pair<Rule>) -> Result<ConstraintDecl, Box<dyn std::error::Error>> {
    let constraint = pair.into_inner().next().unwrap();
    
    match constraint.as_rule() {
        Rule::distance_constraint => {
            let mut inner = constraint.into_inner();
            let a = inner.next().unwrap().as_str    ().to_string();
            let b = inner.next().unwrap().as_str().to_string();
            let value: f32 = inner.next().unwrap().as_str().parse()?;
            
            Ok(ConstraintDecl::Distance { a, b, value })
        }
        Rule::fixed_constraint => {
            let mut inner = constraint.into_inner();
            let identifier_list = inner.next().unwrap();
            let joints = parse_identifier_list(identifier_list);
            
            Ok(ConstraintDecl::Fixed { joints })
        }
        Rule::plane_constraint => {
            let mut inner = constraint.into_inner();
            let identifier_list = inner.next().unwrap();
            let joints = parse_identifier_list(identifier_list);
            
            // Parse the normal (either axis or Vec3)
            let normal_param = inner.next().unwrap();
            let normal = match normal_param.as_str() {
                "X" => Vec3::X,
                "Y" => Vec3::Y,
                "Z" => Vec3::Z,
                _ => {
                    // Parse as Vec3 tuple
                    let mut vec_inner = normal_param.into_inner();
                    let x: f32 = vec_inner.next().unwrap().as_str().parse()?;
                    let y: f32 = vec_inner.next().unwrap().as_str().parse()?;
                    let z: f32 = vec_inner.next().unwrap().as_str().parse()?;
                    Vec3::new(x, y, z)
                }
            };
                    
            // Optional point parameter
            let point = if let Some(point_param) = inner.next() {
                let mut vec_inner = point_param.into_inner();
                let x: f32 = vec_inner.next().unwrap().as_str().parse()?;
                let y: f32 = vec_inner.next().unwrap().as_str().parse()?;
                let z: f32 = vec_inner.next().unwrap().as_str().parse()?;
                Some(Vec3::new(x, y, z))
            } else {
                None
            };
            
            Ok(ConstraintDecl::Plane { joints, normal, point })
        }
        Rule::prismatic_constraint_vector => {
            let mut inner = constraint.into_inner();
            let identifier_list = inner.next().unwrap();
            let joints = parse_identifier_list(identifier_list);
            
            // Parse the axis (either axis or Vec3)
            let axis_param = inner.next().unwrap();
            let axis = match axis_param.as_str() {
                "X" => Vec3::X,
                "Y" => Vec3::Y,
                "Z" => Vec3::Z,
                _ => {
                    // Parse as Vec3 tuple
                    let mut vec_inner = axis_param.into_inner();
                    let x: f32 = vec_inner.next().unwrap().as_str().parse()?;
                    let y: f32 = vec_inner.next().unwrap().as_str().parse()?;
                    let z: f32 = vec_inner.next().unwrap().as_str().parse()?;
                    Vec3::new(x, y, z)
                }
            };
            
            // Parse the origin point
            let origin_param = inner.next().unwrap();
            let mut vec_inner = origin_param.into_inner();
            let x: f32 = vec_inner.next().unwrap().as_str().parse()?;
            let y: f32 = vec_inner.next().unwrap().as_str().parse()?;
            let z: f32 = vec_inner.next().unwrap().as_str().parse()?;
            let origin = Vec3::new(x, y, z);
            
            Ok(ConstraintDecl::PrismaticVector { joints, axis, origin })
        }
        Rule::prismatic_constraint_link => {
            let mut inner = constraint.into_inner();
            let identifier_list = inner.next().unwrap();
            let joints = parse_identifier_list(identifier_list);
            
            // Parse the link name
            let link_name = inner.next().unwrap().as_str().to_string();
            
            // Parse the origin point
            let origin_param = inner.next().unwrap();
            let mut vec_inner = origin_param.into_inner();
            let x: f32 = vec_inner.next().unwrap().as_str().parse()?;
            let y: f32 = vec_inner.next().unwrap().as_str().parse()?;
            let z: f32 = vec_inner.next().unwrap().as_str().parse()?;
            let origin = Vec3::new(x, y, z);
            
            Ok(ConstraintDecl::PrismaticLink { joints, link: link_name, origin })
        }
        Rule::fixed_constraint_angle => {
            let mut inner = constraint.into_inner();
            
            let joint_a = inner.next().unwrap().as_str().to_string();
            let pivot   = inner.next().unwrap().as_str().to_string();
            let joint_c = inner.next().unwrap().as_str().to_string();
            
            let angle_pair = inner.next().unwrap();
            let mut number_value = 0.0f32;
            let mut is_degrees = false;
        
            for inner_pair in angle_pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::number => {
                        number_value = inner_pair.as_str().parse()?;
                    }
                    Rule::angle_unit => {
                        match inner_pair.as_str() {
                            "deg" | "degrees" => is_degrees = true,
                            "rad" | "radians" => is_degrees = false,
                            _ => {} // fallback is radians
                        }
                    }
                    _ => {}
                }
            }
        
            let angle = if is_degrees {
                number_value.to_radians()
            } else {
                number_value
            };
        
            Ok(ConstraintDecl::FixedAngle { joint_a, pivot, joint_c, angle })
        }
        Rule::revolute_constraint => {
            let mut inner = constraint.into_inner();
            let joint_pivot = inner.next().unwrap().as_str().to_string();
            let joint_moving = inner.next().unwrap().as_str().to_string();
            
            let axis_param = inner.next().unwrap();
            let axis = match axis_param.as_str() {
                "X" => Vec3::X,
                "Y" => Vec3::Y,
                "Z" => Vec3::Z,
                _ => {
                    // Parse as Vec3 tuple
                    let mut vec_inner = axis_param.into_inner();
                    let x: f32 = vec_inner.next().unwrap().as_str().parse()?;
                    let y: f32 = vec_inner.next().unwrap().as_str().parse()?;
                    let z: f32 = vec_inner.next().unwrap().as_str().parse()?;
                    Vec3::new(x, y, z)
                }
            };
            let min_angle = inner.next().unwrap().as_str().parse()?;
            let max_angle = inner.next().unwrap().as_str().parse()?;
           
            Ok(ConstraintDecl::Revolute {
                joint_a: joint_pivot,
                joint_b: joint_moving,
                axis,
                min_angle,
                max_angle,
            })
        }
        _ => Err("Unknown constraint type".into())

    }
}

fn parse_identifier_list(pair: Pair<Rule>) -> Vec<String> {
    pair.into_inner()
        .map(|p| p.as_str().to_string())
        .collect()
}