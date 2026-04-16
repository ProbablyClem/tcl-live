use chrono::NaiveDateTime;
use serde::Serialize;
use std::collections::HashMap;

use crate::tcl::Passage;

const METRO_LINES: &[&str] = &["A", "B", "C", "D"];

#[derive(Serialize)]
pub struct MetroPosition {
    pub trip_id: String,
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
    pub snapshot_time: String,
    pub positions: Vec<MetroPosition>,
}

fn parse_dt(s: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok()
}

pub fn compute_positions(passages: Vec<Passage>) -> Positions {
    let snapshot_time = passages
        .first()
        .map(|e| e.last_update_fme.clone())
        .unwrap_or_default();

    // Use local time so it matches the naive timestamps in the API (Paris local time)
    let now = chrono::Local::now().naive_local();

    let mut trips: HashMap<String, Vec<Passage>> = HashMap::new();
    for entry in passages {
        if METRO_LINES.contains(&entry.ligne.as_str()) {
            trips
                .entry(entry.coursetheorique.clone())
                .or_default()
                .push(entry);
        }
    }

    let mut positions: Vec<MetroPosition> = trips
        .into_iter()
        .filter_map(|(trip_id, mut stops)| {
            stops.sort_by(|a, b| a.heurepassage.cmp(&b.heurepassage));

            let line = stops[0].ligne.clone();
            let direction = stops[0].direction.clone();

            // Split into past and future stops relative to now
            let pivot =
                stops.partition_point(|s| parse_dt(&s.heurepassage).map_or(false, |t| t <= now));

            if pivot == 0 || pivot == stops.len() {
                return None; // Train hasn't started or already finished
            }

            let prev = &stops[pivot - 1];
            let next = &stops[pivot];

            let prev_dt = parse_dt(&prev.heurepassage)?;
            let next_dt = parse_dt(&next.heurepassage)?;

            let elapsed = (now - prev_dt).num_seconds();
            let interval = (next_dt - prev_dt).num_seconds();
            let next_stop_in_secs = (next_dt - now).num_seconds();

            let progress = if interval > 0 {
                (elapsed as f64 / interval as f64).clamp(0.0, 1.0)
            } else {
                0.0
            };

            Some(MetroPosition {
                trip_id,
                line,
                direction,
                prev_stop_id: prev.id,
                next_stop_id: next.id,
                progress,
                next_stop_in_secs,
            })
        })
        .collect();

    positions.sort_by(|a, b| a.line.cmp(&b.line).then(a.trip_id.cmp(&b.trip_id)));

    Positions {
        snapshot_time,
        positions,
    }
}
