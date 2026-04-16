use serde::Serialize;

#[derive(Serialize)]
pub struct Positions {
    positions: Vec<Position>,
}

#[derive(Serialize)]
pub struct Position {}
