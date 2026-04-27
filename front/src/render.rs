use crate::ligne::Ligne;
use crate::response::Position;
use crate::{arret::Arret, ui};

use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
    prelude::*,
    text::FontSmoothing,
};
use bevy::{
    prelude::*,
    render::{
        RenderPlugin,
        settings::{Backends, RenderCreation, WgpuSettings},
    },
};
use web_sys::window;

// Singleton
#[derive(Resource)]
pub struct MetroData {
    pub lignes: Vec<Ligne>,
    pub positions: Vec<Position>,
}

#[derive(Component)]
pub struct LigneName(pub String); // marks entities that represent a metro line

pub fn run(lignes: Vec<Ligne>, positions: Vec<Position>) {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    prevent_default_event_handling: false,
                    title: "TCL Live".into(),
                    canvas: Some("#canvas".into()), // ← attach to your existing <canvas>
                    mode: bevy::window::WindowMode::Windowed,
                    ..default()
                }),
                ..default()
            }),
            FpsOverlayPlugin { ..default() },
        ))
        .add_plugins(ui::plugin)
        .insert_resource(MetroData { lignes, positions })
        // Startup systems run exactly once
        .add_systems(Startup, (setup_camera, spawn_metro_lines))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_metro_lines(mut commands: Commands, data: Res<MetroData>, windows: Query<&mut Window>) {
    let windows = windows
        .single()
        .expect("Window resolution couldnt be found");

    let lignes_number = data.lignes.len();

    let vertical_spacing = windows.height() / (lignes_number + 1) as f32;

    for (ligne_idx, ligne) in data.lignes.iter().enumerate() {
        // Y position for this line, centered around 0
        let y = (windows.height() / 2.0) - (ligne_idx as f32 + 1.0) * vertical_spacing;

        let arrets_number = ligne.arrets.len();

        let horizontal_margin = 200.0;
        let x_start = ((windows.width() / 2.0) * -1.0) + horizontal_margin;
        let h_spacing = (windows.width() - (horizontal_margin * 2.0))
            / (arrets_number.saturating_sub(1).max(1)) as f32;

        // Draw line between arrets
        for i in 0..arrets_number.saturating_sub(1) {
            let x0 = x_start + i as f32 * h_spacing;
            let x1 = x_start + (i + 1) as f32 * h_spacing;

            let length = x1 - x0;
            commands.spawn((
                Sprite::from_color(ligne.color(), Vec2::new(length, 3.0)),
                Transform::from_xyz(x0 + length / 2.0, y, 0.0),
            ));
        }

        // Draw a circle for each arret
        for (arret_idx, arret) in ligne.arrets.iter().enumerate() {
            let x = x_start + arret_idx as f32 * h_spacing;
            commands.spawn((
                Sprite::from_color(Color::WHITE, Vec2::splat(8.0)),
                Transform::from_xyz(x, y, 1.0), // z=1 → above the line
                arret.clone(),
                Pickable::default(),
            ));
        }
    }
}
