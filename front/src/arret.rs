use bevy::ecs::component::Component;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Component)]
pub struct Arret {
    pub id: u64,
    pub nom: String,
    pub lat: f64,
    pub lon: f64,
    pub lignes: Vec<String>,
}
