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
    let mut lignes: Vec<Ligne> = lignes
        .into_iter()
        .map(|e| Ligne {
            name: e.0,
            arrets: e.1,
        })
        .collect();
    lignes.sort_by_key(|l| l.name.clone());
    lignes.iter_mut().for_each(|l| {
        l.arrets.sort_by_key(|a| a.nom.clone());
        l.arrets.dedup();
    });
    lignes
}
