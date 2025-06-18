use bevy::{
    color::palettes::tailwind, 
    prelude::*, 
    render::view::RenderLayers,
};
use bevy_infinite_grid::*;
use std::f32::consts::FRAC_PI_2;

use crate::util::camera::CameraModeText;


/// Used implicitly by all entities without a `RenderLayers` component.
/// Our world model camera and all objects other than the player are on this layer.
/// The light source belongs to both layers.
const DEFAULT_RENDER_LAYER: usize = 0;

/// Used by the view model camera and the player's arm.
/// The light source belongs to both layers.
const VIEW_MODEL_RENDER_LAYER: usize = 1;


pub fn spawn_world_model(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    

    

    //XYZ arrows
    let arrow_length = 1.0;
    let arrow_thickness = 0.05;

    let x_arrow_material: Handle<StandardMaterial> = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),
        ..Default::default()
    });
    let y_arrow_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 1.0, 0.0),
        ..Default::default()
    });
    let z_arrow_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.0, 1.0),
        ..Default::default()
    });

    let arrow_mesh = meshes.add(Cylinder::new(arrow_thickness, arrow_length).mesh().resolution(20));

    // X-axis arrow
    commands.spawn((
        Mesh3d(arrow_mesh.clone()),
        MeshMaterial3d(x_arrow_material),
        Transform::from_translation(Vec3::new(arrow_length / 2.0, 0.0, 0.0))
            .with_rotation(Quat::from_rotation_z(-FRAC_PI_2)),
    ));

    // Y-axis arrow
    commands.spawn((
        Mesh3d(arrow_mesh.clone()),
        MeshMaterial3d(y_arrow_material),
        Transform::from_translation(Vec3::new(0.0, arrow_length / 2.0, 0.0)),
    ));

    // Z-axis arrow
    commands.spawn((
        Mesh3d(arrow_mesh),
        MeshMaterial3d(z_arrow_material),
        Transform::from_translation(Vec3::new(0.0, 0.0, arrow_length / 2.0))
            .with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
    ));

    commands.spawn(InfiniteGridBundle::default());


}

pub fn spawn_lights(mut commands: Commands) {
    commands.spawn((
        PointLight {
            color: Color::from(tailwind::ROSE_300),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-2.0, 4.0, -0.75),
        RenderLayers::from_layers(&[DEFAULT_RENDER_LAYER, VIEW_MODEL_RENDER_LAYER]),
    ));
}

pub fn spawn_text(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                right: Val::Px(12.0),
                ..default()
            },
            Name::new("CameraModeIndicator"),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Camera Mode: Perspective 3D"),
                CameraModeText,  
            ));
        });
}


