use serde::Deserialize;

use crate::Config;

const PASSAGE_URL: &str = "httpsdata.grandlyon.com/fr/datapusher/ws/rdata/tcl_sytral.tclpassagearret/all.json?maxfeatures=-1&filename=prochains-passages-reseau-transports-commun-lyonnais-rhonexpress-disponibilites-temps-reel&start=1";

#[derive(Deserialize)]
pub struct PassageApiResponse {
    pub values: Vec<Passage>,
}

#[derive(Deserialize)]
pub struct Passage {
    pub id: u64,
    pub ligne: String,
    pub direction: String,
    pub coursetheorique: String,
    pub heurepassage: String,
    pub last_update_fme: String,
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
