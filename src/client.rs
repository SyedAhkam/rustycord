use snafu::{Snafu, ResultExt};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    LoginFailure,
    ConnectFailure,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

use crate::{
    http::HTTPClient,
    models::Token,
};

#[derive(Debug)]
pub struct Client {
    http: Option<HTTPClient>,
}

impl Client {
    pub fn new() -> Self {
        Self { http: None }
    }

    pub async fn login(&mut self) -> Result<()> {
        self.http.as_mut().unwrap().static_login().await.unwrap();

        Ok(())
    }

    pub async fn connect(&self, token: Token) -> Result<()> {
        Ok(())
    }

    pub async fn run(&mut self, token_as_str: &str) -> Result<()> {
        let token = Token(token_as_str.to_string());

        self.http = Some(HTTPClient::new(token.clone()));

        self.login().await?;

        self.connect(token.clone()).await?;

        Ok(())
    }
}
