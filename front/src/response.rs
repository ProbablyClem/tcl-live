#[derive(serde::Deserialize, Debug)]
pub struct Position {
    pub voyage_id: String,
    pub ligne: String,
    pub direction: String,
    pub prev_arret_id: u64,
    pub next_arret_id: u64,
    pub progress: f64,
    pub next_arret_in_secs: i64,
}

#[derive(serde::Deserialize)]
pub struct Positions {
    pub positions: Vec<Position>,
}
