use crate::ligne::Ligne;
use crate::response::Position;
use crate::{arret::*, ui};

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
                    canvas: Some("#canvas".into()),
                    fit_canvas_to_parent: true,
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

    let mut min_lat = f64::MAX;
    let mut max_lat = f64::MIN;
    let mut min_lon = f64::MAX;
    let mut max_lon = f64::MIN;

    for arret in data.lignes.iter().flat_map(|ligne| ligne.arrets.iter()) {
        if arret.lat < min_lat {
            min_lat = arret.lat;
        }
        if arret.lat > max_lat {
            max_lat = arret.lat;
        }
        if arret.lon < min_lon {
            min_lon = arret.lon;
        }
        if arret.lon > max_lon {
            max_lon = arret.lon;
        }
    }

    for ligne in data.lignes.iter() {
        for arret in ligne.arrets.iter() {
            let x = (arret.lon - min_lon) as f32 / (max_lon - min_lon) as f32 * windows.width()
                - windows.width() / 2.0;
            let y = (arret.lat - min_lat) as f32 / (max_lat - min_lat) as f32 * windows.height()
                - windows.height() / 2.0;
            arret.clone().spawn(
                &mut commands,
                Transform::from_xyz(x, y, 1.0), // z=1 → above the line
            );
        }
    }
}
