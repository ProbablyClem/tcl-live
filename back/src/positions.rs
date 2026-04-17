use crate::{tcl::VoyageId, voyages::Voyage};
use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Serialize)]
pub struct Position {
    pub voyage_id: VoyageId,
    pub ligne: String,
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
    let now = chrono::Local::now().naive_local();
    compute_positions_at(voyages, now)
}

fn compute_positions_at(voyages: Vec<Voyage>, now: NaiveDateTime) -> Positions {
    let mut positions: Vec<Position> = voyages
        .into_iter()
        .flat_map(|v| compute_voyage_position_at(v, now))
        .collect();

    positions.sort_by(|a, b| a.ligne.cmp(&b.ligne));

    Positions { positions }
}

fn compute_voyage_position_at(mut voyage: Voyage, now: NaiveDateTime) -> Option<Position> {
    voyage
        .passages
        .sort_by(|a, b| a.heurepassage.cmp(&b.heurepassage));
    let passages = voyage.passages;

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
        voyage_id: voyage.voyage_id,
        ligne: voyage.ligne,
        direction: voyage.direction,
        prev_stop_id: prev.id,
        next_stop_id: next.id,
        progress,
        next_stop_in_secs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tcl::{Passage, VoyageId};
    use fixture_rs::Fixture;

    fn t(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").unwrap()
    }

    fn make_passage(id: u64, time_str: &str) -> Passage {
        Passage {
            id,
            ligne: "A".to_string(),
            direction: "Perrache".to_string(),
            voyage_id: VoyageId::fixture(),
            heurepassage: t(time_str),
        }
    }

    fn make_voyage(passages: Vec<Passage>) -> Voyage {
        make_voyage_on_ligne("A", passages)
    }

    fn make_voyage_on_ligne(ligne: &str, passages: Vec<Passage>) -> Voyage {
        Voyage {
            voyage_id: VoyageId::fixture(),
            ligne: ligne.to_string(),
            direction: "Perrache".to_string(),
            passages,
        }
    }

    // ── compute_voyage_position_at ──────────────────────────────────────────

    #[test]
    fn test_halfway_progress() {
        let v = make_voyage(vec![
            make_passage(1, "2026-01-01 10:00:00"),
            make_passage(2, "2026-01-01 10:02:00"),
        ]);
        let pos = compute_voyage_position_at(v, t("2026-01-01 10:01:00")).unwrap();
        assert_eq!(pos.prev_stop_id, 1);
        assert_eq!(pos.next_stop_id, 2);
        assert!((pos.progress - 0.5).abs() < 1e-9);
        assert_eq!(pos.next_stop_in_secs, 60);
    }

    #[test]
    fn test_progress_at_prev_stop_is_zero() {
        let v = make_voyage(vec![
            make_passage(1, "2026-01-01 10:00:00"),
            make_passage(2, "2026-01-01 10:02:00"),
        ]);
        let pos = compute_voyage_position_at(v, t("2026-01-01 10:00:00")).unwrap();
        assert!((pos.progress - 0.0).abs() < 1e-9);
        assert_eq!(pos.next_stop_in_secs, 120);
    }

    #[test]
    fn test_progress_one_quarter() {
        let v = make_voyage(vec![
            make_passage(1, "2026-01-01 10:00:00"),
            make_passage(2, "2026-01-01 10:04:00"),
        ]);
        let pos = compute_voyage_position_at(v, t("2026-01-01 10:01:00")).unwrap();
        assert!((pos.progress - 0.25).abs() < 1e-9);
    }

    #[test]
    fn test_progress_three_quarters() {
        let v = make_voyage(vec![
            make_passage(1, "2026-01-01 10:00:00"),
            make_passage(2, "2026-01-01 10:04:00"),
        ]);
        let pos = compute_voyage_position_at(v, t("2026-01-01 10:03:00")).unwrap();
        assert!((pos.progress - 0.75).abs() < 1e-9);
    }

    #[test]
    fn test_one_second_before_next_stop() {
        let v = make_voyage(vec![
            make_passage(1, "2026-01-01 10:00:00"),
            make_passage(2, "2026-01-01 10:02:00"),
        ]);
        let pos = compute_voyage_position_at(v, t("2026-01-01 10:01:59")).unwrap();
        assert_eq!(pos.next_stop_in_secs, 1);
        assert!((pos.progress - 119.0 / 120.0).abs() < 1e-9);
    }

    #[test]
    fn test_before_all_stops_returns_none() {
        let v = make_voyage(vec![
            make_passage(1, "2026-01-01 10:00:00"),
            make_passage(2, "2026-01-01 10:02:00"),
        ]);
        assert!(compute_voyage_position_at(v, t("2026-01-01 09:59:59")).is_none());
    }

    #[test]
    fn test_exactly_at_last_stop_returns_none() {
        // partition_point returns len when all elements satisfy the predicate (<=)
        let v = make_voyage(vec![
            make_passage(1, "2026-01-01 10:00:00"),
            make_passage(2, "2026-01-01 10:02:00"),
        ]);
        assert!(compute_voyage_position_at(v, t("2026-01-01 10:02:00")).is_none());
    }

    #[test]
    fn test_after_last_stop_returns_none() {
        let v = make_voyage(vec![
            make_passage(1, "2026-01-01 10:00:00"),
            make_passage(2, "2026-01-01 10:02:00"),
        ]);
        assert!(compute_voyage_position_at(v, t("2026-01-01 10:05:00")).is_none());
    }

    #[test]
    fn test_single_stop_returns_none() {
        let v = make_voyage(vec![make_passage(1, "2026-01-01 10:00:00")]);
        assert!(compute_voyage_position_at(v, t("2026-01-01 10:00:30")).is_none());
    }

    #[test]
    fn test_unsorted_passages_get_sorted() {
        // Passages fed in reverse order; function must sort by heurepassage before pivoting
        let v = make_voyage(vec![
            make_passage(3, "2026-01-01 10:04:00"),
            make_passage(1, "2026-01-01 10:00:00"),
            make_passage(2, "2026-01-01 10:02:00"),
        ]);
        let pos = compute_voyage_position_at(v, t("2026-01-01 10:01:00")).unwrap();
        assert_eq!(pos.prev_stop_id, 1);
        assert_eq!(pos.next_stop_id, 2);
    }

    #[test]
    fn test_three_stops_first_segment() {
        let v = make_voyage(vec![
            make_passage(1, "2026-01-01 10:00:00"),
            make_passage(2, "2026-01-01 10:02:00"),
            make_passage(3, "2026-01-01 10:04:00"),
        ]);
        let pos = compute_voyage_position_at(v, t("2026-01-01 10:01:00")).unwrap();
        assert_eq!(pos.prev_stop_id, 1);
        assert_eq!(pos.next_stop_id, 2);
        assert!((pos.progress - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_three_stops_second_segment() {
        let v = make_voyage(vec![
            make_passage(1, "2026-01-01 10:00:00"),
            make_passage(2, "2026-01-01 10:02:00"),
            make_passage(3, "2026-01-01 10:04:00"),
        ]);
        let pos = compute_voyage_position_at(v, t("2026-01-01 10:03:00")).unwrap();
        assert_eq!(pos.prev_stop_id, 2);
        assert_eq!(pos.next_stop_id, 3);
        assert!((pos.progress - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_three_stops_exactly_at_middle_stop() {
        // now == passages[1].heurepassage: pivot = 2, prev = stop 2, next = stop 3
        let v = make_voyage(vec![
            make_passage(1, "2026-01-01 10:00:00"),
            make_passage(2, "2026-01-01 10:02:00"),
            make_passage(3, "2026-01-01 10:04:00"),
        ]);
        let pos = compute_voyage_position_at(v, t("2026-01-01 10:02:00")).unwrap();
        assert_eq!(pos.prev_stop_id, 2);
        assert_eq!(pos.next_stop_id, 3);
        assert!((pos.progress - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_metadata_propagated_to_position() {
        let mut v = make_voyage(vec![
            make_passage(1, "2026-01-01 10:00:00"),
            make_passage(2, "2026-01-01 10:02:00"),
        ]);
        v.ligne = "D".to_string();
        v.direction = "Vaise".to_string();
        let pos = compute_voyage_position_at(v, t("2026-01-01 10:01:00")).unwrap();
        assert_eq!(pos.ligne, "D");
        assert_eq!(pos.direction, "Vaise");
    }

    #[test]
    fn test_next_stop_in_secs_three_minutes() {
        let v = make_voyage(vec![
            make_passage(1, "2026-01-01 10:00:00"),
            make_passage(2, "2026-01-01 10:05:00"),
        ]);
        let pos = compute_voyage_position_at(v, t("2026-01-01 10:02:00")).unwrap();
        assert_eq!(pos.next_stop_in_secs, 180);
    }

    #[test]
    fn test_next_stop_in_secs_decreases_over_time() {
        let passages = || {
            vec![
                make_passage(1, "2026-01-01 10:00:00"),
                make_passage(2, "2026-01-01 10:02:00"),
            ]
        };
        let early = compute_voyage_position_at(make_voyage(passages()), t("2026-01-01 10:00:30"))
            .unwrap();
        let late = compute_voyage_position_at(make_voyage(passages()), t("2026-01-01 10:01:30"))
            .unwrap();
        assert_eq!(early.next_stop_in_secs, 90);
        assert_eq!(late.next_stop_in_secs, 30);
        assert!(early.next_stop_in_secs > late.next_stop_in_secs);
    }

    #[test]
    fn test_progress_increases_over_time() {
        let passages = || {
            vec![
                make_passage(1, "2026-01-01 10:00:00"),
                make_passage(2, "2026-01-01 10:04:00"),
            ]
        };
        let early =
            compute_voyage_position_at(make_voyage(passages()), t("2026-01-01 10:01:00")).unwrap();
        let late =
            compute_voyage_position_at(make_voyage(passages()), t("2026-01-01 10:03:00")).unwrap();
        assert!(early.progress < late.progress);
    }

    // ── compute_positions_at ────────────────────────────────────────────────

    #[test]
    fn test_empty_voyages_produces_empty_positions() {
        let result = compute_positions_at(vec![], t("2026-01-01 10:00:00"));
        assert!(result.positions.is_empty());
    }

    #[test]
    fn test_positions_sorted_by_ligne() {
        let now = t("2026-01-01 10:01:00");
        let passages = || {
            vec![
                make_passage(1, "2026-01-01 10:00:00"),
                make_passage(2, "2026-01-01 10:02:00"),
            ]
        };
        let voyages = vec![
            make_voyage_on_ligne("C", passages()),
            make_voyage_on_ligne("A", passages()),
            make_voyage_on_ligne("B", passages()),
        ];
        let result = compute_positions_at(voyages, now);
        let lignes: Vec<&str> = result.positions.iter().map(|p| p.ligne.as_str()).collect();
        assert_eq!(lignes, ["A", "B", "C"]);
    }

    #[test]
    fn test_future_voyage_excluded() {
        let now = t("2026-01-01 10:01:00");
        let voyages = vec![
            make_voyage_on_ligne(
                "A",
                vec![
                    make_passage(1, "2026-01-01 10:00:00"),
                    make_passage(2, "2026-01-01 10:02:00"),
                ],
            ),
            make_voyage_on_ligne(
                "B",
                vec![
                    make_passage(3, "2026-01-01 11:00:00"),
                    make_passage(4, "2026-01-01 11:02:00"),
                ],
            ),
        ];
        let result = compute_positions_at(voyages, now);
        assert_eq!(result.positions.len(), 1);
        assert_eq!(result.positions[0].ligne, "A");
    }

    #[test]
    fn test_completed_voyage_excluded() {
        let now = t("2026-01-01 10:05:00");
        let voyages = vec![
            make_voyage_on_ligne(
                "A",
                vec![
                    make_passage(1, "2026-01-01 10:00:00"),
                    make_passage(2, "2026-01-01 10:02:00"),
                ],
            ),
            make_voyage_on_ligne(
                "B",
                vec![
                    make_passage(3, "2026-01-01 10:03:00"),
                    make_passage(4, "2026-01-01 10:07:00"),
                ],
            ),
        ];
        let result = compute_positions_at(voyages, now);
        assert_eq!(result.positions.len(), 1);
        assert_eq!(result.positions[0].ligne, "B");
    }

    #[test]
    fn test_all_voyages_out_of_range_returns_empty() {
        let now = t("2026-01-01 10:30:00");
        let voyages = vec![
            make_voyage_on_ligne(
                "A",
                vec![
                    make_passage(1, "2026-01-01 09:00:00"),
                    make_passage(2, "2026-01-01 09:02:00"),
                ],
            ),
            make_voyage_on_ligne(
                "B",
                vec![
                    make_passage(3, "2026-01-01 11:00:00"),
                    make_passage(4, "2026-01-01 11:02:00"),
                ],
            ),
        ];
        let result = compute_positions_at(voyages, now);
        assert!(result.positions.is_empty());
    }

    #[test]
    fn test_multiple_valid_voyages_all_included() {
        let now = t("2026-01-01 10:01:00");
        let passages = || {
            vec![
                make_passage(1, "2026-01-01 10:00:00"),
                make_passage(2, "2026-01-01 10:02:00"),
            ]
        };
        let voyages = vec![
            make_voyage_on_ligne("A", passages()),
            make_voyage_on_ligne("B", passages()),
            make_voyage_on_ligne("C", passages()),
        ];
        let result = compute_positions_at(voyages, now);
        assert_eq!(result.positions.len(), 3);
    }
}
