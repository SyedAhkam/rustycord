use reqwest::{
    Client as ReqwestClient,
    Url,
    Method,
    Response,
    StatusCode,
    Error,
    header::{HeaderName, HeaderValue}
};

use crate::{
    RustyCordResult,
    errors::{
        HTTPException,
        UnauthorizedException,
        LoginException
    },
    models::{
        User
    }
};

use std::any::Any;

const BASE_URL: &str = "https://discord.com/api/v9";

struct Route {
    method: Method,
    url: Url
}

#[derive(Clone, Debug)]
pub struct HTTPClient<'a> {
    pub token: &'a str,
    pub req_client: ReqwestClient,
    pub user: Option<User>
}

impl<'a> HTTPClient<'a> {
    pub fn new(token: &'a str) -> Self {
        Self { token: token, req_client: ReqwestClient::new(), user: None }
    }

    async fn request(&self, route: Route) -> Result<Response, Error> {
        self.req_client.execute(
            self.req_client.request(route.method, route.url)
            .header(HeaderName::from_static("authorization"), HeaderValue::from_str(format!("Bot {}", self.token).as_str()).unwrap())
            .build()
            .unwrap()
        ).await
    }

    pub async fn get_current_user(&self) -> RustyCordResult<User> {
        match self.request(Route {
            method: Method::GET,
            url: Url::parse(format!("{}/users/@me", BASE_URL).as_str()).unwrap()
        }).await {
            Ok(resp) => {
                match resp.error_for_status() {
                    Ok(resp) => {
                        match resp.json::<User>().await {
                            Ok(user) => return Ok(user),
                            Err(err) => return Err(Box::new(HTTPException(format!("Failed to parse user response: {:?}", err))))
                        };
                    },
                    Err(err) => {
                        if err.status() == Some(StatusCode::UNAUTHORIZED) {
                            return Err(Box::new(UnauthorizedException(format!("Not authorized to get user: {:?}", err))))
                        } else {
                            return Err(Box::new(HTTPException(format!("Unexpected response code: {:?}", err))))
                        }
                    }
                }
            },
            Err(err) => return Err(Box::new(HTTPException(format!("Failed to send http request: {:?}", err.url()))))
        }
    }

    pub async fn static_login(&mut self) -> RustyCordResult<()> {
        match self.get_current_user().await {
            Ok(user) => self.user = Some(user),
            Err(err) => {
                //TODO: need to work on better error handling
                return Err(Box::new(LoginException(format!("Invalid token was passed: {:?}", err))))
            }
        }

        Ok(())
    }
}
