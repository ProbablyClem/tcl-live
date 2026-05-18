use serde::{Deserialize, Serialize};

use crate::tcl::arret::Arret;
use std::collections::HashMap;

const TRACE_URL: &str = "https://data.grandlyon.com/fr/geoserv/ogc/features/v1/collections/sytral:tcl_sytral.tcllignemf_2_0_0/items?&f=application/geo%2Bjson&crs=EPSG:4171&startIndex=0&sortby=gid&limit=100";
#[derive(Serialize, Deserialize)]
struct GeoJsonResponse {
    pub features: Vec<Feature>,
}

#[derive(Serialize, Deserialize)]
pub struct Feature {
    #[serde(rename = "type")]
    pub feature_type: String,
    pub geometry: Geometry,
    pub properties: Properties,
    pub bbox: Vec<f64>,
}
#[derive(Serialize, Deserialize)]
pub struct Geometry {
    pub coordinates: Vec<Vec<Vec<f64>>>,
}

#[derive(Serialize, Deserialize)]
pub struct Properties {
    pub gid: u64,
    pub ligne: String,
    pub code_ligne: String,
    pub nom_trace: String,
    pub sens: String,
    pub origine: String,
    pub destination: String,
    pub famille_transport: String,
}

#[derive(Serialize)]

pub struct Ligne {
    name: String,
    arrets: Vec<Arret>,
    traces: Vec<Feature>,
}

pub fn group_by_ligne(arrets: Vec<Arret>, features: Vec<Feature>) -> Vec<Ligne> {
    let mut lignes: HashMap<String, Vec<Arret>> = HashMap::new();

    for arret in arrets {
        for ligne in &arret.lignes {
            lignes.entry(ligne.clone()).or_default().push(arret.clone());
        }
    }

    let mut traces_by_ligne: HashMap<String, Vec<Feature>> = HashMap::new();
    for feature in features {
        traces_by_ligne
            .entry(feature.properties.ligne.clone())
            .or_default()
            .push(feature);
    }

    let mut lignes: Vec<Ligne> = lignes
        .into_iter()
        .map(|(name, arrets)| Ligne {
            traces: traces_by_ligne.remove(&name).unwrap_or_default(),
            name,
            arrets,
        })
        .collect();
    lignes.sort_by_key(|l| l.name.clone());
    lignes.iter_mut().for_each(|l| {
        l.arrets.sort_by_key(|a| a.nom.clone());
        l.arrets.dedup();
    });
    lignes
}

pub async fn fetch_traces() -> Vec<Feature> {
    let response = reqwest::get(TRACE_URL)
        .await
        .expect("arret API request failed")
        .json::<GeoJsonResponse>()
        .await
        .expect("Failed to parse arret response");

    response.features
}
