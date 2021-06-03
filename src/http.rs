use reqwest::{
    Client as ReqwestClient,
    Url,
    Method,
    Response,
    Error,
    header::{HeaderName, HeaderValue}
};

use crate::{
    RustyCordResult,
    errors::HTTPException
};

const BASE_URL: &str = "https://discord.com/api/v9";

struct Route {
    method: Method,
    url: Url
}

#[derive(Clone, Debug)]
pub struct HTTPClient<'a> {
    pub token: &'a str,
    pub req_client: ReqwestClient
}

impl<'a> HTTPClient<'a> {
    pub fn new(token: &'a str) -> Self {
        Self { token: token, req_client: ReqwestClient::new() }
    }

    async fn request(&self, route: Route) -> Result<Response, Error> {
        self.req_client.execute(
            self.req_client.request(route.method, route.url)
            .header(HeaderName::from_static("authorization"), HeaderValue::from_str(format!("Bot {}", self.token).as_str()).unwrap())
            .build()
            .unwrap()
        ).await
    }

    pub async fn static_login(&self) -> RustyCordResult<()> {
        match self.request(Route {
            method: Method::GET,
            url: Url::parse(format!("{}/users/@me", BASE_URL).as_str()).unwrap()
        }).await {
            Ok(resp) => println!("{:#?}", resp.text().await),
            Err(err) => return Err(Box::new(HTTPException(format!("Failed to send http request: {:?}", err.url()))))
        }

        Ok(())
    }
}
