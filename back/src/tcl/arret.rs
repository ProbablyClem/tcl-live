use super::METRO_LINES;
use serde::{Deserialize, Serialize};

const ARRET_URL: &str = "https://data.grandlyon.com/fr/datapusher/ws/rdata/tcl_sytral.tclarret/all.json?maxfeatures=-1&start=1";

#[derive(Deserialize)]
struct ArretApiResponse {
    values: Vec<ArretRaw>,
}

#[derive(Deserialize)]
struct ArretRaw {
    id: u64,
    nom: String,
    lat: f64,
    lon: f64,
    desserte: String,
}

#[derive(Serialize, Clone)]
pub struct Arret {
    pub id: u64,
    pub nom: String,
    pub lat: f64,
    pub lon: f64,
    pub lignes: Vec<String>,
}

impl From<ArretRaw> for Arret {
    fn from(value: ArretRaw) -> Self {
        Arret {
            id: value.id,
            nom: value.nom,
            lat: value.lat,
            lon: value.lon,
            lignes: extract_lignes_from_desserte(value.desserte),
        }
    }
}

fn extract_lignes_from_desserte(desserte: String) -> Vec<String> {
    let mut lignes: Vec<String> = desserte
        .split(',')
        .filter_map(|entry| entry.split(':').next().map(str::to_string))
        .collect();
    lignes.sort();
    lignes.dedup();
    lignes
}

pub async fn fetch_arrets() -> Vec<Arret> {
    let response = reqwest::get(ARRET_URL)
        .await
        .expect("arret API request failed")
        .json::<ArretApiResponse>()
        .await
        .expect("Failed to parse arret response");

    response
        .values
        .into_iter()
        .map(Arret::from)
        .filter(|a| {
            METRO_LINES
                .iter()
                .any(|&m| a.lignes.contains(&m.to_string()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_arret_api_response() {
        let json = r#"{
            "fields": ["id", "nom", "desserte", "pmr", "ascenseur", "escalator", "gid",
                       "last_update", "last_update_fme", "adresse", "commune", "insee", "zone", "lon", "lat"],
            "layer_name": "tcl_sytral.tclarret",
            "nb_results": 1,
            "table_alias": null,
            "table_href": "https://data.grandlyon.com/fr/datapusher/ws/rdata/tcl_sytral.tclarret.json",
            "values": [{
                "adresse": "PLACE BELLECOUR",
                "ascenseur": true,
                "commune": "Lyon",
                "desserte": "A:R",
                "escalator": false,
                "gid": 1,
                "id": 1340,
                "insee": "69123",
                "last_update": "2026-04-19 02:53:00",
                "last_update_fme": "2026-04-19 05:06:25",
                "lat": 45.757836,
                "localise_face_a_adresse": null,
                "lon": 4.832338,
                "nom": "Bellecour",
                "pmr": true,
                "zone": "1"
            }]
        }"#;

        let response: ArretApiResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.values.len(), 1);
        let arret = &response.values[0];
        assert_eq!(arret.id, 1340);
        assert_eq!(arret.nom, "Bellecour");
        assert_eq!(arret.desserte, "A:R");
        assert!((arret.lat - 45.757836).abs() < 1e-6);
        assert!((arret.lon - 4.832338).abs() < 1e-6);
    }

    #[test]
    fn test_extract_lignes_from_desserte() {
        assert_eq!(extract_lignes_from_desserte("A:R".to_string()), vec!["A"]);
        assert_eq!(
            extract_lignes_from_desserte("39:R,39:A,JD11:R".to_string()),
            vec!["39", "JD11"]
        );
        assert_eq!(
            extract_lignes_from_desserte("A:R,B:A".to_string()),
            vec!["A", "B"]
        );
    }
}
