use crate::voyages::Voyage;
use serde::Serialize;

#[derive(Serialize)]
pub struct Position {
    pub line: String,
    pub direction: String,
    pub prev_stop_id: u64,
    pub next_stop_id: u64,
    /// Progression between prev and next stop, from 0.0 to 1.0
    pub progress: f64,
    pub next_stop_in_secs: i64,
}

#[derive(Serialize)]
pub struct Positions {
    pub positions: Vec<Position>,
}

pub fn compute_positions(voyages: Vec<Voyage>) -> Positions {
    // Use local time so it matches the naive timestamps in the API (Paris local time)
    let mut positions: Vec<Position> = voyages
        .into_iter()
        .flat_map(compute_voyage_positions)
        .collect();

    positions.sort_by(|a, b| a.line.cmp(&b.line));

    Positions { positions }
}

fn compute_voyage_positions(mut voyage: Voyage) -> Option<Position> {
    let now = chrono::Local::now().naive_local();

    voyage
        .passages
        .sort_by(|a, b| a.heurepassage.cmp(&b.heurepassage));
    let passages = voyage.passages;

    let line = voyage.ligne.clone();
    let direction = voyage.direction.clone();

    // Split into past and future stops relative to now
    let pivot = passages.partition_point(|s| s.heurepassage <= now);

    if pivot == 0 || pivot == passages.len() {
        return None;
    }

    let prev = &passages[pivot - 1];
    let next = &passages[pivot];

    let prev_dt = prev.heurepassage;
    let next_dt = next.heurepassage;

    let elapsed = (now - prev_dt).num_seconds();
    let interval = (next_dt - prev_dt).num_seconds();
    let next_stop_in_secs = (next_dt - now).num_seconds();

    let progress = if interval > 0 {
        (elapsed as f64 / interval as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };

    Some(Position {
        line,
        direction,
        prev_stop_id: prev.id,
        next_stop_id: next.id,
        progress,
        next_stop_in_secs,
    })
}
