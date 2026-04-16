use fixture_rs::Fixture;
use serde::Deserialize;

use crate::Config;

const PASSAGE_URL: &str = "https://data.grandlyon.com/fr/datapusher/ws/rdata/tcl_sytral.tclpassagearret/all.json?maxfeatures=-1&filename=prochains-passages-reseau-transports-commun-lyonnais-rhonexpress-disponibilites-temps-reel&start=1";

#[derive(Deserialize, Fixture, PartialEq, Eq, Debug)]
pub struct PassageApiResponse {
    pub values: Vec<Passage>,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct Passage {
    pub id: u64,
    pub ligne: String,
    pub direction: String,
    pub coursetheorique: String,
    pub heurepassage: String,
    pub last_update_fme: String,
}

impl Fixture for Passage {
    fn fixture() -> Self {
        Passage {
            id: 1,
            ligne: "A".to_string(),
            direction: "Perrache".to_string(),
            coursetheorique: "31_31B-023AT_00601030".to_string(),
            heurepassage: "2026-04-16 18:31:14".to_string(),
            last_update_fme: "2026-04-16 18:31:01".to_string(),
        }
    }
}

pub async fn fetch_passages(conf: Config) -> Vec<Passage> {
    let response: PassageApiResponse = reqwest::Client::new()
        .get(PASSAGE_URL)
        .basic_auth(conf.env.user, Some(conf.env.password))
        .send()
        .await
        .expect("API request failed")
        .json()
        .await
        .expect("Failed to parse API response");
    response.values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_passage_api_response() {
        let json = r#"{
            "fields": ["id", "ligne", "direction", "delaipassage", "type", "heurepassage",
                       "idtarretdestination", "coursetheorique", "gid", "last_update_fme"],
            "layer_name": "tcl_sytral.tclpassagearret",
            "nb_results": 1,
            "table_alias": null,
            "table_href": "https://data.grandlyon.com/fr/datapusher/ws/rdata/tcl_sytral.tclpassagearret.json",
            "values": [{
                "coursetheorique": "31_31B-023AT_00601030",
                "delaipassage": "Proche",
                "direction": "Perrache",
                "gid": 1,
                "heurepassage": "2026-04-16 18:31:14",
                "id": 1,
                "idtarretdestination": 528,
                "last_update_fme": "2026-04-16 18:31:01",
                "ligne": "A",
                "type": "E"
            }]
        }"#;

        let response: PassageApiResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response, PassageApiResponse::fixture());
    }
}
