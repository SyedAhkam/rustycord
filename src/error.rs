use snafu::Snafu;

use crate::client;
use crate::http;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    ClientError { source: client::Error },
    HTTPError { source: http::Error },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
