use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    LoginFailure,
    ConnectFailure,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
