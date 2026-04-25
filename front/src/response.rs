#[derive(serde::Deserialize, Debug)]
pub struct Arret {
    pub id: u64,
    pub nom: String,
    pub lat: f64,
    pub lon: f64,
    pub lignes: Vec<String>,
}

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
