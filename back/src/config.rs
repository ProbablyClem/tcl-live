use crate::env::Env;

#[derive(Clone)]
pub struct Config {
    pub env: Env,
}

impl Config {
    pub fn from(env: Env) -> Config {
        Config { env }
    }
}
