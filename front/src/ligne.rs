use std::cmp::Ordering;

use super::response::Arret;
use bevy::color::Color;
use bevy::prelude::Srgba;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Ligne {
    pub name: LigneName,
    pub arrets: Vec<Arret>,
}

#[derive(Deserialize, Debug, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub enum LigneName {
    A,
    B,
    C,
    D,
}

impl Ligne {
    pub fn color(&self) -> Color {
        Color::Srgba(Srgba::hex(self.color_hex()).unwrap())
    }

    fn color_hex(&self) -> &'static str {
        match self.name {
            LigneName::A => "e82825",
            LigneName::B => "00a4de",
            LigneName::C => "f5a300",
            LigneName::D => "009b47",
        }
    }
}
