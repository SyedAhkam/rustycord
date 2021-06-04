use serde::Deserialize;

use crate::models::Snowflake;

#[derive(Deserialize, Clone, Debug)]
pub struct User {
    #[serde(flatten)]
    pub id: Snowflake,
    #[serde(rename = "username")]
    pub user_name: String,
    pub discriminator: String,
    pub avatar: Option<String>,
    pub bot: bool,
    pub system: Option<bool>,
    pub mfa_enabled: bool,
    pub locale: Option<String>,
    pub verified: bool,
    pub email: Option<String>,
    pub flags: Option<i32>,
    pub premium_type: Option<i32>,
    pub public_flags: Option<i32>,
}
