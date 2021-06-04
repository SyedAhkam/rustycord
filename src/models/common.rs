use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(Deserialize, Clone, Debug)]
pub struct Snowflake {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    #[serde(rename = "id")]
    value: i64,
}
