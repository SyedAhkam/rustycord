pub mod models;

use log::{info, trace, warn, debug, error};
use snafu::{ensure, Backtrace, ErrorCompat, ResultExt, Snafu};
use rustc_version_runtime::version;

use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client as ReqwestClient,
    Result as ReqResult,
    Error as ReqError,
    Method, Response, StatusCode, Url,
};

use serde_json;

use std::env;

use crate::models::{
    Token,
    DiscordError,
    User
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to send http request to: {}", endpoint))]
    RequestSend { endpoint: String, source: ReqError },

    #[snafu(display("Failed to parse response text"))]
    RequestParse { status: StatusCode, source: ReqError },

    #[snafu(display("({}) {}", code, message))]
    BadRequest { message: String, code: i32 },

    #[snafu(display("({}) {}", code, message))]
    Unauthorized { message: String, code: i32 },

    #[snafu(display("({}): {}", code, message))]
    Forbidden { message: String, code: i32 },

    #[snafu(display("({}): {}", code, message))]
    NotFound { message: String, code: i32 },

    #[snafu(display("({}): {}", code, message))]
    TooManyRequests { message: String, code: i32 },

    #[snafu(display("({}): {}", code, message))]
    InternalServerError { message: String, code: i32 },

    #[snafu(display("({}): {}", code, message))]
    BadGateway { message: String, code: i32 },

    #[snafu(display("({}): {}", code, message))]
    ServiceUnavailable { message: String, code: i32 },

    #[snafu(display("Invalid token was passed: {:?}", token))]
    InvalidToken { token: Token },

    #[snafu(display("Failed to deserialize to {}: {}", target, text))]
    DeserializeError { text: String, target: String, source: serde_json::Error },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

const BASE_URL: &str = "https://discord.com/api/v9";
const USER_ENDPOINT: &str = "/users";

#[derive(Debug)]
struct Config {
    token: Option<Token>
}

#[derive(Debug)]
pub struct Route {
    method: Method,
    url: Url,
}

#[derive(Debug)]
pub struct HttpClient {
    token: Token,
    req_client: ReqwestClient
}

impl HttpClient {
    /// Returns a new instance
    pub fn new(token: Token) -> Self {
        let mut headers = HeaderMap::new();
        
        let mut authorization_header_value = HeaderValue::from_str(format!("Bot {}", token).as_str()).unwrap();
        authorization_header_value.set_sensitive(true);

        headers.insert(
            HeaderName::from_static("authorization"),
            authorization_header_value
        );

        let req_client = ReqwestClient::builder()
            .user_agent(
                format!(
                    "DiscordBot ({}, {}) Rust {}",
                    env!("CARGO_PKG_REPOSITORY"),
                    env!("CARGO_PKG_VERSION"),
                    version()
                )
            )
            .default_headers(headers)
            .build()
            .unwrap();

        Self { token, req_client }
    }

    async fn request(&self, route: Route) -> ReqResult<Response> {
        self.req_client.execute(
            self.req_client.request(route.method, route.url).build()?
        ).await
    }

    async fn construct_error(&self, status: StatusCode, error: DiscordError) -> Result<()> {
        let DiscordError { message, code, .. } = error;

        if status.is_client_error() {
            match status {
                StatusCode::BAD_REQUEST => return BadRequest { message, code }.fail(),
                StatusCode::UNAUTHORIZED => return Unauthorized { message, code }.fail(),
                StatusCode::FORBIDDEN => return Forbidden { message, code }.fail(),
                StatusCode::NOT_FOUND => return NotFound { message, code }.fail(),
                StatusCode::TOO_MANY_REQUESTS => return TooManyRequests { message, code }.fail(),
                _ => ()
            }
        }

        if status.is_server_error() {
            match status {
                StatusCode::INTERNAL_SERVER_ERROR => return InternalServerError { message, code }.fail(),
                StatusCode::BAD_GATEWAY => return BadGateway { message, code }.fail(),
                StatusCode::SERVICE_UNAVAILABLE => return ServiceUnavailable { message, code }.fail(),
                _ => ()
            }
        }

        Ok(())
    }

    async fn inspect_response(&self, data: &str, status: StatusCode) -> Result<()> {
        if let Ok(err_resp) = serde_json::from_str::<DiscordError>(data) {
            self.construct_error(status, err_resp).await?;
        }

        Ok(())
    }

    /// Returns `/users/<user_id>` response text
    pub async fn fetch_user(&self, user_id: i64) -> Result<String> {
        let resp = self.request(Route {
            method: Method::GET,
            url: Url::parse(format!("{}{}/{}", BASE_URL, USER_ENDPOINT, user_id).as_str()).unwrap()
        })
            .await
            .context(RequestSend { endpoint: format!("/users/{}", user_id) })?;

        let status = resp.status();

        let data = resp.text().await.context(RequestParse { status })?;

        self.inspect_response(data.as_str(), status).await?;

        Ok(data)
    }

    /// Returns `/users/@me` response text
    pub async fn fetch_current_user(&self) -> Result<String> {
        let resp = self.request(Route {
            method: Method::GET,
            url: Url::parse(format!("{}{}/@me", BASE_URL, USER_ENDPOINT).as_str()).unwrap()
        }).await.context(RequestSend { endpoint: "/users/@me" })?;

        let status = resp.status();

        let data = resp.text().await.context(RequestParse { status })?;

        self.inspect_response(data.as_str(), status).await?;

        Ok(data)
    }

    /// Logs in using static token
    pub async fn static_login(&self) -> Result<String> {
        debug!("Logging in using static token");

        match self.fetch_current_user().await {
            Ok(user_text) => Ok(user_text),
            Err(Error::Unauthorized { .. }) => InvalidToken { token: self.token.clone() }.fail()?,
            Err(err) => Err(err)
        }
    }
}

#[derive(Debug)]
pub struct Client {
    http: HttpClient,
    user: Option<User>
}

#[derive(Debug)]
pub struct ClientBuilder {
    config: Config
}


impl Client {
    /// Start constructing a new instance of `ClientBuilder` 
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new() 
    }

    /// Fetches a `User`
    /// This is an API call
    pub async fn fetch_user(&self, user_id: i64) -> Result<User> {
        let user_text = self.http.fetch_user(user_id).await?;
        debug!("recieved user on fetch_user: {}", user_text);

        Ok(
            serde_json::from_str::<User>(user_text.as_str())
                .context(DeserializeError { text: user_text, target: "User" })?
        )
    }

    /// Logs in using the static token
    async fn login(&mut self) -> Result<()> {
        let user_text = self.http.static_login().await?;
        debug!("recieved current user on login: {}", user_text);

        // Deserialize User then add to self.user
        self.user = Some(
            serde_json::from_str::<User>(user_text.as_str())
                .context(DeserializeError { text: user_text, target: "current User" })?
        );

        Ok(())
    }

    /// Connects to discord gateway
    async fn connect(&self) -> Result<()> {
        // TODO
        Ok(())
    }

    /// Calls `login` and `connect` with error handling
    pub async fn run(mut self) {
        match self.login().await {
            Ok(_) => info!("Logged in successfully"),
            Err(e) => {
                eprintln!("An error occured while trying to login: {}", e);

                if let Some(backtrace) = ErrorCompat::backtrace(&e) {
                    eprintln!("{}", backtrace);
                }
            }
        }
        
        match self.connect().await {
            Ok(_) => info!("Connected to gateway successfully"),
            Err(e) => {
                eprintln!("An error occured while trying to connect: {}", e);

                if let Some(backtrace) = ErrorCompat::backtrace(&e) {
                    eprintln!("{}", backtrace);
                }
            }
        }
    }
}

impl Default for ClientBuilder {
    /// Aliased to `ClientBuilder::new()`
    fn default() -> Self {
        Self::new()
    }
}

impl ClientBuilder {
    /// Returns a new instance of builder
    /// Same as `Client::builder()`
    pub fn new() -> Self {
        Self { 
            config: Config {
                token: None
            }
        }
    }

    /// Sets bot token
    pub fn token(mut self, token: &str) -> ClientBuilder {
        self.config.token = Some(Token(token.to_string()));
        self
    }

    /// Sets bot token using environment variable
    pub fn token_from_env(mut self, env_name: &'static str) -> ClientBuilder {
        self.config.token = Some(Token(env::var(env_name).expect("Environment variable not set! ")));
        self
    }

    /// Builds a `Client` object
    pub fn build(self) -> Client {
        let token = self.config.token.unwrap();

        if token.is_empty() {
            panic!("Empty token was passed")
        }

        Client {
            http: HttpClient::new(token.clone()),
            user: None
        }
    }
}
