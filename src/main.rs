pub mod camera;
pub mod events;
pub mod material;
pub mod plugins;

use bevy::{
    asset::AssetServerSettings,
    prelude::*,
    window::{PresentMode, WindowDescriptor, WindowMode, WindowResizeConstraints},
};
use camera::pan_orbit_camera::{OrbitCamera, OrbitCameraPlugin};
use material::{
    animated_material::{setup_animated_cubes, AnimatedMaterialPlugin},
    game_of_life::GameOfLifePlugin,
    shader_material::CustomMaterial,
};
use plugins::mouse::MousePlugin;

fn main() {
    let mut app = App::new();

    // Add main world systems/resources
    app.insert_resource(WindowDescriptor {
        title: "bev_101".to_string(),
        width: 1280.0,
        height: 720.0,
        position: WindowPosition::Automatic,
        scale_factor_override: None, //Some(1.0), //Needed for some mobile devices, but disables scaling
        present_mode: PresentMode::Immediate,
        resizable: true,
        decorations: true,
        cursor_locked: false,
        cursor_visible: true,
        mode: WindowMode::Windowed,
        transparent: false,
        fit_canvas_to_parent: true,
        resize_constraints: WindowResizeConstraints::default(),
        canvas: Some("#bevy".to_string()),
        ..default()
    })
    .add_plugins(DefaultPlugins)
    .add_plugin(MousePlugin)
    .add_plugin(OrbitCameraPlugin)
    .add_plugin(AnimatedMaterialPlugin)
    .add_plugin(GameOfLifePlugin)
    .insert_resource(AssetServerSettings {
        watch_for_changes: true,
        ..default()
    })
    .add_startup_system(setup_animated_cubes)
    .add_startup_system(lights_camera_action);

    // .add_system(update_custom_material);

    // Add render world systems / resources
    app.run();
}

/// generic scene
/// set up a simple 3D scene
fn lights_camera_action(mut commands: Commands) {
    // load a texture and retrieve its aspect ratio

    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(OrbitCamera::default());

    // light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}

// fn update_custom_material(
//     // access entities that have `Health` and `Transform` components
//     // get read-only access to `Health` and mutable access to `Transform`
//     // optional component: get access to `Player` if it exists
//     mut query: Query<&mut CustomMaterial>,
// ) {
//     // get all matching entities
//     for mut custom_material in query.iter_mut() {
//         custom_material.color = Color::CRIMSON;
//     }
// }
