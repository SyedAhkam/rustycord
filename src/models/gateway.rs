#![allow(non_camel_case_types)]

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum GatewayOpcode {
    Dispatch,
    Heartbeat,
    Identify,
    PresenceUpdate,
    VoiceStateUpdate,
    Resume,
    Reconnect,
    RequestGuildMembers,
    InvalidSession,
    Hello,
    HeartbeatAck,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum GatewayEvent {
    HELLO,
    READY,
    RESUMED,
    RECONNECT,
    INVALID_SESSION,

    APPLICATION_COMMAND_CREATE,
    APPLICATION_COMMAND_UPDATE,
    APPLICATION_COMMAND_DELETE,

    CHANNEL_CREATE,
    CHANNEL_UPDATE,
    CHANNEL_DELETE,
    CHANNEL_PINS_UPDATE,

    THREAD_CREATE,
    THREAD_UPDATE,
    THREAD_DELETE,
    THREAD_LIST_SYNC,
    THREAD_MEMBER_UPDATE,
    THREAD_MEMBERS_UPDATE,

    GUILD_CREATE,
    GUILD_UPDATE,
    GUILD_DELETE,

    GUILD_BAN_ADD,
    GUILD_BAN_REMOVE,

    GUILD_EMOJIS_UPDATE,
    GUILD_INTEGRATIONS_UPDATE,

    GUILD_MEMBER_ADD,
    GUILD_MEMBER_REMOVE,
    GUILD_MEMBER_UPDATE,
    GUILD_MEMBERS_CHUNK,

    GUILD_ROLE_CREATE,
    GUILD_ROLE_UPDATE,
    GUILD_ROLE_DELETE,

    INTEGRATION_CREATE,
    INTEGRATION_UPDATE,
    INTEGRATION_DELETE,

    INTERACTION_UPDATE,

    INVITE_CREATE,
    INVITE_DELTE,

    MESSAGE_CREATE,
    MESSAGE_UPDATE,
    MESSAGE_DELETE,
    MESSAGE_DELETE_BULK,
    MESSAGE_REACTION_ADD,
    MESSAGE_REACTION_REMOVE,
    MESSAGE_REACTION_REMOVE_ALL,
    MESSAGE_REACTION_REMOVE_EMOJI,

    PRESENCE_UPDATE,
    TYPING_START,
    USER_UPDATE,
    VOICE_STATE_UPDATE,
    VOICE_SERVER_UPDATE,
    WEBHOOKS_UPDATE,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HelloPayload {
    pub heartbeat_interval: i32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Payload {
    Hello(HelloPayload),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SessionStartLimit {
    pub total: i32,
    pub remaining: i32,
    pub reset_after: i32,
    pub max_concurrency: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Gateway {
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BotGateway {
    pub url: String,
    pub shards: i32,
    pub session_start_limit: SessionStartLimit,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GatewayMessage {
    pub op: GatewayOpcode,

    pub d: Option<Payload>,
    pub s: Option<i32>,
    pub t: Option<GatewayEvent>,
}
