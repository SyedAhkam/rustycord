use serde::{Deserialize, Serialize};

#[derive(Debug)]
enum GatewayOpcode {}

#[derive(Debug)]
enum GatewayEvent {}

#[derive(Debug, Deserialize)]
pub struct SessionStartLimit {
    total: i32,
    remaining: i32,
    reset_after: i32,
    max_concurrency: i32,
}

#[derive(Debug, Deserialize)]
pub struct Gateway {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct BotGateway {
    pub url: String,
    pub shards: i32,
    pub session_start_limit: SessionStartLimit,
}

#[derive(Debug)]
struct GatewayMessage {
    op: GatewayOpcode,
    d: Option<String>,
    s: Option<i32>,
    t: GatewayEvent,
}
