use snafu::ResultExt;

use bytes::Bytes;

use reqwest::{
    header::{HeaderName, HeaderValue},
    Client as ReqwestClient,
    Result as ReqResult,
    Method, Response, StatusCode, Url,
};

use crate::{
    models::{Token},
    http::error::{
        Result,
        RequestFailed
    }
};

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
            StatusCode::UNAUTHORIZED => return RequestFailed{ status: status}.fail()
        }
    }

    pub async fn get_current_user(&self) -> Result<Bytes> {
        let result =  self.request(Route {
            method: Method::GET,
            url: Url::parse(format!("{}{}/@me", BASE_URL, USER_ENDPOINT).as_str()).unwrap()
        }).await;

        if result.is_err() { return RequestFailed{}.fail() }

        let resp = result.unwrap();

        self.check_response_status(resp.status()).await?;

        Ok(resp.bytes().await.unwrap())
    }

    pub async fn static_login(&mut self) -> Result<(Bytes)> {
        Ok(self.get_current_user()
            .await?
        )
    }
}
