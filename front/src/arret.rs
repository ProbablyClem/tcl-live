use crate::ui::tooltip::Tooltip;
use bevy::picking::prelude::*;
use bevy::prelude::Pickable;
use bevy::prelude::*;
use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
    prelude::*,
    text::FontSmoothing,
};
use bevy::{
    ecs::{component::Component, system::Command},
    transform,
};
use bevy::{
    prelude::*,
    render::{
        RenderPlugin,
        settings::{Backends, RenderCreation, WgpuSettings},
    },
};
use serde::Deserialize;
use web_sys::console;

#[derive(Deserialize, Debug, Clone, Component)]
pub struct Arret {
    pub id: u64,
    pub nom: String,
    pub lat: f64,
    pub lon: f64,
    pub lignes: Vec<String>,
}

impl Arret {
    pub fn spawn(self, commands: &mut Commands, transform: Transform) {
        commands.spawn((
            Sprite::from_color(Color::WHITE, Vec2::splat(8.0)),
            transform,
            self,
            Pickable::default(),
        ));
    }
}
