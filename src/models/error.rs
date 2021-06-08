use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct DiscordError {
    pub code: i32,
    pub errors: Option<String>, // this response is dynamic, no idea how to deserialize this
    pub message: String,
}
