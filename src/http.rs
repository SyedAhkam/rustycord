use reqwest::{
    Client as ReqwestClient,
    Url,
    Method,
    Response,
    StatusCode,
    Error,
    header::{HeaderName, HeaderValue}
};

use snafu::{ResultExt};

use crate::{
    RustyCordResult,
    errors::{
        HTTPRequestFailure,
        ParseFailure,
        IncorrectToken,
        Unauthorized
    },
    models::{
        User
    }
};

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
            .build()?
        ).await
    }

    pub async fn get_current_user(&self) -> RustyCordResult<User> {
        Ok(self.request(Route {
            method: Method::GET,
            url: Url::parse(format!("{}/users/@me", BASE_URL).as_str()).unwrap()
        })
            .await
            .context(HTTPRequestFailure { route: "fhweuif".to_string() })?
            .error_for_status()
            .context(Unauthorized { resource: "/users/@me" })?
            .json::<User>()
            .await
            .context(ParseFailure { resp: "uwfhe".to_string() })?
        )
    }

    pub async fn static_login(&mut self) -> RustyCordResult<()> {
        self.user = Some(self.get_current_user()
            .await
            .context(IncorrectToken { token: self.token.to_string() })?
        );

        Ok(())
    }
}
