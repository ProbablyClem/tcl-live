use axum::{Json, Router, extract::State, routing::get};

use crate::{
    config::Config,
    env::Env,
    positions::{Positions, compute_positions},
};
mod config;
mod env;
mod positions;
mod tcl;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv_override();
    let env = Env::load();
    let config = Config::from(env);

    let app = Router::new()
        .route("/", get(positions_handler))
        .with_state(config.clone());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.env.port.clone()))
        .await
        .unwrap();
    println!("Listening on http://0.0.0.0:{}", config.env.port);
    axum::serve(listener, app).await.unwrap();
}

async fn positions_handler(State(config): State<Config>) -> Json<Positions> {
    let passages = tcl::fetch_passages(config).await;
    Json(compute_positions(passages))
}
