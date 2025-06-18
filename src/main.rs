pub mod util;
pub mod simcore;
pub mod dsl;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlAnchorElement, Blob, BlobPropertyBag, Url};

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
use crate::util::keybindings::*;


#[derive(Resource, Default)]
pub struct ListeningState {
    current: Option<&'static str>,
}


#[derive(Resource, Default)]
pub struct FilePath {
    pub path: String,
}

#[derive(Resource, Default)]
pub struct TextState {
    pub content: String,
    pub other_speed_string: String,
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
        .insert_resource(ListeningState::default())
        .insert_resource(FilePath::default())
        .insert_resource(KeyBindings::default())
        .insert_resource(SimWrapper {
            sim: Simulation::default(),
        })
        .add_systems(Startup, set_fullscreen_canvas)
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
        .add_systems(EguiContextPass, keybindings_ui)
        .run();
}



fn setup_sim(    
    mut sim_wrapper: ResMut<SimWrapper>,
    joint_query: Query<Entity, With<JointWrapper>>,
    link_query: Query<Entity, With<LinkWrapper>>,

    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {

    let code = std::fs::read_to_string("src\\examples\\fourbar.ugoku");
    match code {
        Ok(ref s) => {
            match setup_sim_from_dsl(s) {
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
        },
        Err(e) => {
            eprintln!("Failed to read file: {}", e);
        }
    }
    

}



fn ui_example_system(
    mut contexts: EguiContexts,
    mut sim_wrapper: ResMut<SimWrapper>,
    joint_query: Query<Entity, With<JointWrapper>>,
    link_query: Query<Entity, With<LinkWrapper>>,

    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    mut text_state: ResMut<TextState>,
    mut file_path: ResMut<FilePath>,
    mut input_focus: ResMut<InputFocus>,

) { 
    let ctx = contexts.ctx_mut();
    input_focus.egui_focused = ctx.wants_pointer_input() || ctx.wants_keyboard_input();

    egui::Window::new("Ugoku!")
        .resizable(true)
        .collapsible(true)
        .default_size([400.0, 300.0])
        .min_size([200.0, 150.0])
        .max_size([800.0, 600.0])
        .show(contexts.ctx_mut(), |ui| {
            ui.label("world");
            // this could be like attach file idk, im not focusing on web builds anymoreq
            //i hate ui bro
            /*
            ui.add_sized(
                [200.0, 11.0],
                egui::TextEdit::singleline(&mut file_path.path)
                    .hint_text("Enter filepath here")
            );
             */
            ui.add_sized(
                [ui.available_width(), 200.0],
                egui::TextEdit::multiline(&mut text_state.content)
                    .code_editor()
                    .hint_text("Enter text here")
            );

            if ui.button("Save to file").clicked() {
                match save_string_to_file("simulation.ugoku", &text_state.content) {
                    Ok(_) => web_sys::console::log_1(&"File saved successfully!".into()),
                    Err(e) => web_sys::console::error_1(&format!("Error saving file: {:?}", e).into()),
                }
            }
            
            if ui.button("Compile").clicked() {

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
pub fn keybindings_ui(
    mut contexts: EguiContexts,
    keys: Res<ButtonInput<KeyCode>>,
    mut bindings: ResMut<KeyBindings>,
    mut listen: ResMut<ListeningState>,
    mut text_state: ResMut<TextState>,
) {
    if text_state.other_speed_string.is_empty() {
        text_state.other_speed_string = bindings.iterations_per_time_step.to_string();
    }

    egui::Window::new("Keybindings").show(contexts.ctx_mut(), |ui| {
        ui.label("Click a button, then press a new key.");

        macro_rules! editable_binding {
            ($label:expr, $field:ident) => {
                ui.horizontal(|ui| {
                    ui.label($label);
                    let btn = ui.button(format!("{:?}", bindings.$field));
                    if btn.clicked() {
                        listen.current = Some(stringify!($field));
                    }
                });
            };
        }

        editable_binding!("Pan Up", pan_up);
        editable_binding!("Pan Down", pan_down);
        editable_binding!("Pan Left", pan_left);
        editable_binding!("Pan Right", pan_right);
        editable_binding!("Orbit Left", orbit_left);
        editable_binding!("Orbit Right", orbit_right);
        editable_binding!("Zoom In", zoom_in);
        editable_binding!("Zoom Out", zoom_out);
        editable_binding!("Shift", shift);

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("iteration counter");

            let response = ui.text_edit_singleline(&mut text_state.other_speed_string);

            // On Enter or losing focus, try to parse and update binding
            if (response.lost_focus() && keys.pressed(KeyCode::Enter)) || response.changed() 
                        {
                if let Ok(val) = text_state.other_speed_string.trim().parse::<usize>() {
                    bindings.iterations_per_time_step = val;
                    text_state.other_speed_string = val.to_string();
                } else {
                }
            }
        });
    });

    if let Some(field) = listen.current {
        for key in keys.get_just_pressed() {
            match field {
                "pan_up" => bindings.pan_up = *key,
                "pan_down" => bindings.pan_down = *key,
                "pan_left" => bindings.pan_left = *key,
                "pan_right" => bindings.pan_right = *key,
                "orbit_left" => bindings.orbit_left = *key,
                "orbit_right" => bindings.orbit_right = *key,
                "zoom_in" => bindings.zoom_in = *key,
                "zoom_out" => bindings.zoom_out = *key,
                "shift" => bindings.shift = *key,
                _ => {}
            }
            listen.current = None;
        }
    }
}

fn setup_sim_from_dsl(dsl_code: &str) -> Result<Simulation, Box<dyn std::error::Error>> {
    // Parse DSL to AST
    let program = UgokuParser::parse_dsl(dsl_code)?;

    // Compile AST to simulation
    let sim = DslCompiler::compile_to_simulation(program)
        .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    Ok(sim)
}

#[wasm_bindgen]
pub fn save_string_to_file(filename: &str, contents: &str) -> Result<(), JsValue> {
    use web_sys::*;
    
    let window = window().ok_or("no global `window` exists")?;
    let document = window.document().ok_or("should have a document on window")?;

    // Create blob with proper array handling
    let blob_parts = js_sys::Array::new();
    blob_parts.push(&JsValue::from_str(contents));
    
    let blob_options = BlobPropertyBag::new();
    blob_options.set_type("text/plain");
    
    let blob = Blob::new_with_str_sequence_and_options(&blob_parts, &blob_options)?;
    let url = Url::create_object_url_with_blob(&blob)?;

    // Create and configure download link
    let a = document
        .create_element("a")?
        .dyn_into::<HtmlAnchorElement>()?;
    
    a.set_href(&url);
    a.set_download(filename);
 
    // Append, click, and cleanup
    let body = document.body().ok_or("document should have a body")?;
    body.append_child(&a)?;
    a.click();
    body.remove_child(&a)?;
    
    // Revoke the object URL to free memory
    Url::revoke_object_url(&url)?;

    Ok(())
}

fn set_fullscreen_canvas(mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut();

    // Use the actual browser window dimensions
    let width = web_sys::window().unwrap().inner_width().unwrap().as_f64().unwrap();
    let height = web_sys::window().unwrap().inner_height().unwrap().as_f64().unwrap();

    window.unwrap().resolution.set(width as f32, height as f32);
}
