mod event;

use bevy::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.5, 0.5, 0.9)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Draft".into(),
                name: Some("Draft".into()),
                canvas: Some("#main_canvas".into()),
                fit_canvas_to_parent: true,
                ..Default::default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .run();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.5, 0.5, 0.9)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Draft".into(),
                name: Some("Draft".into()),
                canvas: Some("#main_canvas".into()),
                fit_canvas_to_parent: true,
                ..Default::default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
