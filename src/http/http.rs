use snafu::ResultExt;

use reqwest::{
    header::{HeaderName, HeaderValue},
    Client as ReqwestClient,
    Result as ReqResult,
    Method, Response, StatusCode, Url,
};

use crate::{
    Result,
    models::{Token, User},
    http::error::{
        RequestFailed,
        ParseError
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
    user: Option<User>,
}

impl HTTPClient {
    pub fn new(token: Token) -> Self {
        Self {
            token: token,
            user: None,
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
    
    pub async fn get_current_user(&self) -> ReqResult<User> {
        Ok(self.request(Route {
            method: Method::GET,
            url: Url::parse(format!("{}{}/@me", BASE_URL, USER_ENDPOINT).as_str()).unwrap()
        })
            .await.unwrap()
            .json::<User>()
            .await.unwrap()
        )
    }

    pub async fn static_login(&mut self) -> Result<()> {
        self.user = Some(self.get_current_user().await.unwrap());

        Ok(())
    }
}
