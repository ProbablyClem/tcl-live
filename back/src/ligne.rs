use serde::Serialize;

use crate::tcl::arret::Arret;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct Ligne {
    name: String,
    arrets: Vec<Arret>,
}

pub fn group_by_ligne(arrets: Vec<Arret>) -> Vec<Ligne> {
    let mut lignes: HashMap<String, Vec<Arret>> = HashMap::new();

    for arret in arrets {
        for ligne in &arret.lignes {
            lignes.entry(ligne.clone()).or_default().push(arret.clone());
        }
    }

    lignes
        .into_iter()
        .map(|e| Ligne {
            name: e.0,
            arrets: e.1,
        })
        .collect()
}
