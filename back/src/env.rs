use std::env;

#[derive(Clone)]
pub struct Env {
    pub user: String,
    pub password: String,
    pub port: u32,
}

impl Env {
    pub fn load() -> Env {
        Env {
            user: env::var("USER").expect("USER env var not found"),
            password: env::var("PASSWORD").expect("PASSWORD env var not found"),
            port: env::var("PORT")
                .map(|s| s.parse::<u32>().unwrap())
                .unwrap_or(3000),
        }
    }
}
