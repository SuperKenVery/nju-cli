use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Data {
    pub ticket: String,
    pub env: Env
}

#[derive(Deserialize, Debug)]
pub struct Env {
    pub need: bool,
    pub timing: String
}
