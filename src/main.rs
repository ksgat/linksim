pub mod util;
pub mod simcore;
pub mod dsl;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContextPass, EguiContexts, EguiPlugin};
use bevy_infinite_grid::InfiniteGridPlugin;

use crate::util::constants::*;
use crate::util::camera::*;
use crate::util::world::*;
use crate::util::interact::*;
use crate::util::simulation::*;
use crate::simcore::types::*;
use crate::dsl::*;

#[derive(Resource, Default)]
struct TextState {
    content: String,
}
#[derive(Resource, Default)]
struct JointState {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Event)]
pub struct ReplaceSim(pub Simulation);


fn main() {
    App::new()
        
        .add_event::<PickedJoint>()
        .add_event::<MoveJoint>()        
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(InfiniteGridPlugin)
        .insert_resource(CameraModeIndicator::default())
        .insert_resource(SelectedJoint::default())
        .insert_resource(TextState::default())
        .insert_resource(InputFocus::default())
        .insert_resource(JointState::default())
        .add_systems(
            Startup,
            (
                spawn_view_model,
                spawn_world_model,
                spawn_lights,
                spawn_text,
                setup_sim,
                render_sim
            ).chain(),
        )
        .add_systems(Update, camera_control_system)
        .add_systems(Update, (update_camera_mode_indicator, update_camera_mode_text))
        .add_systems(Update, (
            interact_system, 
            highlight_system,  
            reset_on_release_system,
            joint_drag_system,
            sim_step_system,
            update_joint_visuals.after(sim_step_system),
            update_link_visuals.after(sim_step_system),
        ))
        .add_systems(EguiContextPass, ui_example_system)
        .run();
}



fn setup_sim(mut commands: Commands) {
    let mut sim = Simulation::default();
    let joint_pos_a = Position::Vec3(glam::Vec3::new(0.0, 0.0, 0.0));
    let joint_pos_b = Position::Vec3(glam::Vec3::new(2.0, 0.0, 0.0));
    let joint_pos_c = Position::Vec3(glam::Vec3::new(2.0, 0.0, 2.0));
    let joint_pos_d = Position::Vec3(glam::Vec3::new(0.0, 0.0, 2.0));
    
    // Create joints
    let joint_a = sim.joints.insert(Joint {
        position: joint_pos_a,
        joint_type: JointType::Fixed,
        connected_links: Vec::new(),
    });
    let joint_b = sim.joints.insert(Joint {
        position: joint_pos_b,
        joint_type: JointType::Revolute,
        connected_links: Vec::new(),
    });
    let joint_c = sim.joints.insert(Joint {
        position: joint_pos_c,
        joint_type: JointType::Revolute,
        connected_links: Vec::new(),
    });
    let joint_d = sim.joints.insert(Joint {
        position: joint_pos_d,
        joint_type: JointType::Revolute,
        connected_links: Vec::new(),
    });



    // Define link properties
    let link_ab_len = joint_pos_a.distance(joint_pos_b);
    let link_bc_len = joint_pos_b.distance(joint_pos_c);
    let link_cd_len = joint_pos_c.distance(joint_pos_d);
    let link_da_len = joint_pos_d.distance(joint_pos_a);

    // Create links
    let link_ab = sim.links.insert(Link {
        joints: vec![joint_a, joint_b],
        rigid: true,
    });
    let link_bc = sim.links.insert(Link {
        joints: vec![joint_b, joint_c],
        rigid: true,
    });
    let link_cd = sim.links.insert(Link {
        joints: vec![joint_c, joint_d],
        rigid: true,
    });
    let link_da = sim.links.insert(Link {
        joints: vec![joint_d, joint_a],
        rigid: true,
    });

    // Add links to joints
    sim.joints.get_mut(joint_a).unwrap().connected_links.push(link_ab);
    sim.joints.get_mut(joint_a).unwrap().connected_links.push(link_da);
    sim.joints.get_mut(joint_b).unwrap().connected_links.push(link_ab);
    sim.joints.get_mut(joint_b).unwrap().connected_links.push(link_bc);
    sim.joints.get_mut(joint_c).unwrap().connected_links.push(link_bc);
    sim.joints.get_mut(joint_c).unwrap().connected_links.push(link_cd);
    sim.joints.get_mut(joint_d).unwrap().connected_links.push(link_cd);
    sim.joints.get_mut(joint_d).unwrap().connected_links.push(link_da);

    // Fix joints A and C
    sim.constraints.push(Box::new(FixedPositionConstraint {
        joint_id: joint_a,
        target_position: joint_pos_a,
    }));
    sim.constraints.push(Box::new(FixedPositionConstraint {
        joint_id: joint_b,
        target_position: joint_pos_b,
    }));

    // Add distance constraints
    sim.constraints.push(Box::new(DistanceConstraint {
        joint_a: joint_a,
        joint_b: joint_b,
        target_distance: link_ab_len,
    }));
    sim.constraints.push(Box::new(DistanceConstraint {
        joint_a: joint_b,
        joint_b: joint_c,
        target_distance: link_bc_len,
    }));
    sim.constraints.push(Box::new(DistanceConstraint {
        joint_a: joint_c,
        joint_b: joint_d,
        target_distance: link_cd_len,
    }));
    sim.constraints.push(Box::new(DistanceConstraint {
        joint_a: joint_d,
        joint_b: joint_a,
        target_distance: link_da_len,
    }));
    let plane = glam::Vec3::Y;
    sim.constraints.push(Box::new(PlaneConstraint {
        joint_id: joint_a,
        normal: plane,
        plane_point: glam::Vec3::ZERO,
    }));
    sim.constraints.push(Box::new(PlaneConstraint {
        joint_id: joint_b,
        normal: plane,
        plane_point: glam::Vec3::ZERO,
    }));
    sim.constraints.push(Box::new(PlaneConstraint {
        joint_id: joint_c,
        normal: plane,
        plane_point: glam::Vec3::ZERO,
    }));
    sim.constraints.push(Box::new(PlaneConstraint {
        joint_id: joint_d,
        normal: plane,
        plane_point: glam::Vec3::ZERO,
    }));

    commands.insert_resource(SimWrapper { sim });
}



//f

fn ui_example_system(
    mut contexts: EguiContexts,
    mut sim_wrapper: ResMut<SimWrapper>,
    joint_query: Query<Entity, With<JointWrapper>>,
    link_query: Query<Entity, With<LinkWrapper>>,

    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    mut text_state: ResMut<TextState>,
    mut input_focus: ResMut<InputFocus>,
    mut joint_state: ResMut<JointState>,

) {
    let ctx = contexts.ctx_mut();
    input_focus.egui_focused = ctx.wants_pointer_input() || ctx.wants_keyboard_input();

    egui::SidePanel::left("my_panel").show(ctx, |ui| {
        ui.label("world");

        ui.add(egui::TextEdit::multiline(&mut text_state.content).code_editor());
        
        ui.horizontal(|ui| {
            ui.label("X:");
            ui.add(egui::DragValue::new(&mut joint_state.x));
            ui.label("Y:");
            ui.add(egui::DragValue::new(&mut joint_state.y));
            ui.label("Z:");
            ui.add(egui::DragValue::new(&mut joint_state.z));
        });
        
        if ui.button("Do Something").clicked() {
            match setup_sim_from_dsl(text_state.content.as_str()) {
                Ok(new_sim) => {
                    println!("Successfully created simulation with {} joints", new_sim.joints.len());
                    sim_wrapper.sim = new_sim;
                    render_sim(
                        sim_wrapper,
                        joint_query,
                        link_query,
                        commands,
                        meshes,
                        materials,
                        
                    );

                },
                Err(e) => {
                    eprintln!("Error parsing DSL: {}", e);
                }
            }
        }
    });
}


fn setup_sim_from_dsl(dsl_code: &str) -> Result<Simulation, Box<dyn std::error::Error>> {
    // Parse DSL to AST
    let program = UgokuParser::parse_dsl(dsl_code)?;
    
    // Compile AST to simulation
    let sim = DslCompiler::compile_to_simulation(program)
        .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    
    Ok(sim)
}
