/// Public modules
pub mod client;
pub mod http;
pub mod models;

// Re-exports
pub use client::Client;

// Error
use snafu::Snafu;

/* #[derive(Debug, Snafu)] */
/* pub struct Error(InnerError); */

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    HTTPError { source: http::Error },
    ClientError { source: client::Error },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
