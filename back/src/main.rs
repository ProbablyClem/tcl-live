use axum::{Json, Router, extract::State, response::Html, routing::get};
use tower_http::services::ServeDir;

use crate::{
    config::Config,
    env::Env,
    ligne::Ligne,
    positions::{Positions, compute_positions},
    voyages::group_by_voyage,
};
mod config;
mod env;
mod ligne;
mod positions;
mod tcl;
mod voyages;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv_override();
    let env = Env::load();
    let config = Config::from(env);

    let app = Router::new()
        .route("/", get(index))
        .route("/api/positions", get(positions_handler))
        .route("/api/lignes", get(lignes_handler))
        .with_state(config.clone())
        .nest_service("/pkg", ServeDir::new("../static/pkg"));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.env.port.clone()))
        .await
        .unwrap();
    println!("Listening on http://0.0.0.0:{}", config.env.port);
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../../static/index.html"))
}

async fn positions_handler(State(config): State<Config>) -> Json<Positions> {
    let passages = tcl::fetch_passages(config).await;
    let voyages = group_by_voyage(passages);
    Json(compute_positions(voyages))
}

async fn lignes_handler() -> Json<Vec<Ligne>> {
    let arrets = tcl::fetch_arrets().await;
    Json(ligne::group_by_ligne(arrets))
}
