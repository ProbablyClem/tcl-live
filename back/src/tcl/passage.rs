use crate::Config;
use crate::tcl::METRO_LINES;
use crate::tcl::tcl_date_utils::{self, parse_tcl_date};
use crate::tcl::voyage_id::VoyageId;
use chrono::NaiveDateTime;
use fixture_rs::Fixture;
use serde::Deserialize;

const PASSAGE_URL: &str = "https://data.grandlyon.com/fr/datapusher/ws/rdata/tcl_sytral.tclpassagearret/all.json?maxfeatures=-1&start=1&field=ligne&value=";

#[derive(Deserialize, Fixture, PartialEq, Eq, Debug)]
pub struct PassageApiResponse {
    pub values: Vec<Passage>,
}

#[derive(Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Passage {
    pub id: u64,
    pub ligne: String,
    pub direction: String,
    #[serde(rename = "idtarretdestination")]
    pub id_arret_destination: u64,
    #[serde(rename = "coursetheorique")]
    pub voyage_id: VoyageId,
    #[serde(with = "tcl_date_utils")]
    pub heurepassage: NaiveDateTime,
}

impl Fixture for Passage {
    fn fixture() -> Self {
        Passage {
            id: 1,
            ligne: "A".to_string(),
            direction: "Perrache".to_string(),
            id_arret_destination: 1,
            voyage_id: VoyageId::fixture(),
            heurepassage: parse_tcl_date("2026-04-16 18:31:14").unwrap(),
        }
    }
}

pub async fn fetch_passages(conf: Config) -> Vec<Passage> {
    let client = reqwest::Client::new();
    let mut tasks = tokio::task::JoinSet::new();

    // We can filter by ligne, but we can pass multiples values
    // We need make multiples request with the lignes one by one
    // Reduced the fetch from 12s to 1s
    // We use tokio to run the queries in parralel and then we join
    for &ligne in METRO_LINES {
        let client = client.clone();
        let conf = conf.clone();
        tasks.spawn(async move {
            client
                .get(format!("{PASSAGE_URL}{ligne}"))
                .basic_auth(conf.env.user, Some(conf.env.password))
                .send()
                .await
                .expect("API request failed")
                .json::<PassageApiResponse>()
                .await
                .expect("Failed to parse API response")
                .values
        });
    }

    let mut passages = Vec::new();
    while let Some(result) = tasks.join_next().await {
        passages.extend(result.expect("task panicked"));
    }
    passages
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
                "idtarretdestination": 1,
                "last_update_fme": "2026-04-16 18:31:01",
                "ligne": "A",
                "type": "E"
            }]
        }"#;

        let response: PassageApiResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response, PassageApiResponse::fixture());
    }
}
