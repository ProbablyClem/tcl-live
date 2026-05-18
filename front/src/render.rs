use crate::ligne::Ligne;
use crate::response::Position;
use crate::{arret::*, ui};
use web_sys::console;

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
#[derive(Resource)]
struct MapBounds {
    min_lat: f64,
    max_lat: f64,
    min_lon: f64,
    max_lon: f64,
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
        .add_systems(
            Startup,
            (
                setup_camera,
                compute_min_max,
                spawn_metro_lines,
                draw_metro_traces,
            )
                .chain(),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn compute_min_max(mut commands: Commands, data: Res<MetroData>) {
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
    commands.insert_resource(MapBounds {
        min_lat,
        max_lat,
        min_lon,
        max_lon,
    });
}

fn spawn_metro_lines(
    mut commands: Commands,
    data: Res<MetroData>,
    bounds: Res<MapBounds>,
    windows: Query<&Window>,
) {
    let windows = windows
        .single()
        .expect("Window resolution couldnt be found");

    for ligne in data.lignes.iter() {
        for arret in ligne.arrets.iter() {
            let x = (arret.lon - bounds.min_lon) as f32 / (bounds.max_lon - bounds.min_lon) as f32
                * windows.width()
                - windows.width() / 2.0;
            let y = (arret.lat - bounds.min_lat) as f32 / (bounds.max_lat - bounds.min_lat) as f32
                * windows.height()
                - windows.height() / 2.0;
            arret.clone().spawn(
                &mut commands,
                Transform::from_xyz(x, y, 1.0), // z=1 → above the line
            );
        }
    }
}

fn draw_segment(commands: &mut Commands, x0: f32, y0: f32, x1: f32, y1: f32, color: Color) {
    let mid_x = (x0 + x1) / 2.0;
    let mid_y = (y0 + y1) / 2.0;
    let length = Vec2::new(x1 - x0, y1 - y0).length();
    let angle = f32::atan2(y1 - y0, x1 - x0);
    commands.spawn((
        Sprite::from_color(color, Vec2::new(length, 3.0)),
        Transform::from_xyz(mid_x, mid_y, 0.0).with_rotation(Quat::from_rotation_z(angle)),
    ));
}

fn draw_metro_traces(
    mut commands: Commands,
    data: Res<MetroData>,
    bounds: Res<MapBounds>,
    windows: Query<&Window>,
) {
    let windows = windows
        .single()
        .expect("Window resolution couldnt be found");

    for ligne in data.lignes.iter() {
        for trace in ligne.traces.iter() {
            for segment in trace.geometry.coordinates.iter() {
                for pair in segment.windows(2) {
                    let x0 = (pair[0][0] - bounds.min_lon) as f32
                        / (bounds.max_lon - bounds.min_lon) as f32
                        * windows.width()
                        - windows.width() / 2.0;
                    let y0 = (pair[0][1] - bounds.min_lat) as f32
                        / (bounds.max_lat - bounds.min_lat) as f32
                        * windows.height()
                        - windows.height() / 2.0;
                    let x1 = (pair[1][0] - bounds.min_lon) as f32
                        / (bounds.max_lon - bounds.min_lon) as f32
                        * windows.width()
                        - windows.width() / 2.0;
                    let y1 = (pair[1][1] - bounds.min_lat) as f32
                        / (bounds.max_lat - bounds.min_lat) as f32
                        * windows.height()
                        - windows.height() / 2.0;
                    draw_segment(&mut commands, x0, y0, x1, y1, ligne.color());
                }
            }
        }
    }
}
