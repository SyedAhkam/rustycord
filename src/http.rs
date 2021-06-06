use snafu::{Snafu, ResultExt};
use bytes::Bytes;

use reqwest::{
    header::{HeaderName, HeaderValue},
    Client as ReqwestClient,
    Result as ReqResult,
    Error as ReqError,
    Method, Response, StatusCode, Url,
};

use crate::{
    models::{Token},
};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("failed to send http request"))]
    Send { source: ReqError },

    #[snafu(display("unexpected status code recieved"))]
    Status { source: StatusError }
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum StatusError {
    #[snafu(display("Unauthorized to send http request"))]
    Unauthorized { source: Box<Error> }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

const BASE_URL: &str = "https://discord.com/api/v9";
const USER_ENDPOINT: &str = "/users";

struct Route {
    method: Method,
    url: Url,
}

#[derive(Debug)]
pub struct HTTPClient {
    token: Token,
    req_client: ReqwestClient,
}

impl HTTPClient {
    pub fn new(token: Token) -> Self {
        Self {
            token: token,
            req_client: ReqwestClient::new(),
        }
    }

    async fn request(&self, route: Route) -> ReqResult<Response> {
        self.req_client.execute(
            self.req_client.request(route.method, route.url)
            .header(HeaderName::from_static("authorization"), HeaderValue::from_str(format!("Bot {}", self.token).as_str()).unwrap())
            .build()?
        ).await
    }
    
    async fn check_response_status(&self, status: StatusCode) -> Result<()> {
        match status {
            StatusCode::UNAUTHORIZED => return Unauthorized.fail(),
            _ => Ok(())
        }
    }

    pub async fn get_current_user(&self) -> Result<Bytes> {
        let resp = self.request(Route {
            method: Method::GET,
            url: Url::parse(format!("{}{}/@me", BASE_URL, USER_ENDPOINT).as_str()).unwrap()
        }).await.context(Send).unwrap();

        self.check_response_status(resp.status()).await?;

        Ok(resp.bytes().await.unwrap())
    }

    pub async fn static_login(&mut self) -> Result<Bytes> {
        Ok(self.get_current_user()
            .await.context(Status)?
        )
    }
}
