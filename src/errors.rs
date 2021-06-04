use snafu::Snafu;

use reqwest::Error as ReqwestError;
use serde_json::Error as SerdeError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum RustyCordError {
    #[snafu(display("Failed to make http request to {}", route))]
    HTTPRequestFailure { route: String, source: ReqwestError },

    #[snafu(display("Unauthorized to access resource: {}", resource))]
    Unauthorized {
        resource: String,
        source: ReqwestError,
    },

    #[snafu(display("Incorrect token was passed: {}", token))]
    IncorrectToken { token: String, source: ReqwestError },

    #[snafu(display("Failed to parse response: {}", resp))]
    ParseFailure { resp: String, source: SerdeError },
}

pub type RustyCordResult<T, E = RustyCordError> = Result<T, E>;
