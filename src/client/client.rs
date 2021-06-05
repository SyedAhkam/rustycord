use snafu::ResultExt;

use crate::{
    http::HTTPClient,
    Result,
    models::Token,
    client::error::{
        LoginFailure,
        ConnectFailure
    }
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
        self.http.as_mut().unwrap().static_login().await?;

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