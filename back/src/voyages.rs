use crate::tcl;
use crate::tcl::{Passage, VoyageId};
use std::collections::HashMap;

pub struct Voyage {
    pub voyage_id: VoyageId,
    pub ligne: String,
    pub direction: String,
    pub passages: Vec<Passage>,
}

pub fn group_by_voyage(passages: Vec<Passage>) -> Vec<Voyage> {
    let mut voyages: HashMap<VoyageId, Vec<Passage>> = HashMap::new();

    for passage in passages {
        if tcl::METRO_LINES.contains(&passage.ligne.as_str()) {
            voyages
                .entry(passage.voyage_id.clone())
                .or_default()
                .push(passage);
        }
    }

    voyages
        .into_iter()
        .map(|e| Voyage {
            voyage_id: e.0,
            ligne: e.1.first().expect("voyage has no passage").ligne.clone(),
            direction: e
                .1
                .first()
                .expect("voyage has no passage")
                .direction
                .clone(),
            passages: e.1,
        })
        .collect()
}
