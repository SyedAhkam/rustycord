use reqwest::{Error as ReqError, StatusCode};
use serde_json::Error as SerdeError;

use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    RequestFailed {
        status: StatusCode,
        source: ReqError,
    },
    ParseError {
        source: SerdeError,
    },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
