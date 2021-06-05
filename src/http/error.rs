use reqwest::Error as ReqError;
use serde_json::Error as SerdeError;

use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    RequestFailed { source: ReqError },
    ParseError { source: SerdeError },
}
