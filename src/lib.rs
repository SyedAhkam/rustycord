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

use std::env;

use crate::models::Token;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to send http request to: {}", endpoint))]
    RequestSend { endpoint: String, source: ReqError },

    #[snafu(display("Failed to parse response text"))]
    RequestParse { status: StatusCode, source: ReqError },

    #[snafu(display("400: Bad request"))]
    BadRequest,

    #[snafu(display("401: Unauthorized"))]
    Unauthorized,

    #[snafu(display("403: Forbidden"))]
    Forbidden,

    #[snafu(display("404: Not Found"))]
    NotFound,

    #[snafu(display("429: Too Many Requests"))]
    TooManyRequests,

    #[snafu(display("500: Internal Server Error (discord)"))]
    InternalServerError,

    #[snafu(display("504: Bad Gateway (discord)"))]
    BadGateway,

    #[snafu(display("503: Service Unavailable (discord)"))]
    ServiceUnavailable,

    #[snafu(display("Invalid token was passed: {:?}", token))]
    InvalidToken { token: Token }
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
                    "DiscordBot ({}, {}) {}",
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

    async fn check_status_code(&self, status: StatusCode) -> Result<()> {
        if status.is_client_error() {
            match status {
                StatusCode::BAD_REQUEST => return BadRequest.fail(),
                StatusCode::UNAUTHORIZED => return Unauthorized.fail(),
                StatusCode::FORBIDDEN => return Forbidden.fail(),
                StatusCode::NOT_FOUND => return NotFound.fail(),
                StatusCode::TOO_MANY_REQUESTS => return TooManyRequests.fail(),
                _ => ()
            }
        }

        if status.is_server_error() {
            match status {
                StatusCode::INTERNAL_SERVER_ERROR => return InternalServerError.fail(),
                StatusCode::BAD_GATEWAY => return BadGateway.fail(),
                StatusCode::SERVICE_UNAVAILABLE => return ServiceUnavailable.fail(),
                _ => ()
            }
        }

        Ok(())
    }

    async fn inspect_response(&self, resp: &Response) -> Result<()> {
        //TODO: check error "message" field somehow

        self.check_status_code(resp.status()).await?;

        Ok(())
    }

    /// Returns `/users/@me` response text
    pub async fn fetch_current_user(&self) -> Result<String> {
        let resp = self.request(Route {
            method: Method::GET,
            url: Url::parse(format!("{}{}/@me", BASE_URL, USER_ENDPOINT).as_str()).unwrap()
        }).await.context(RequestSend { endpoint: "/users/@me" })?;

        let status = &resp.status();

        self.inspect_response(&resp).await?;

        Ok(resp.text().await.context(RequestParse { status })?)
    }

    /// Logs in using static token
    pub async fn static_login(&self) -> Result<String> {
        debug!("Logging in using static token");
        Ok(self.fetch_current_user().await?)
    }
}

#[derive(Debug)]
pub struct Client {
    http: HttpClient
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

    /// Logs in using the static token
    async fn login(&self) -> Result<bool> {
        match self.http.static_login().await {
            Ok(user_text) => {
                debug!("Recieved current user info: {}", user_text);
                Ok(true)
            },
            Err(e) => Err(e)
        }
        // InvalidToken{ token: Token("e") }.fail()
    }

    /// Connects to discord gateway
    async fn connect(&self) -> Result<bool> {
        Ok(true)
    }

    /// Calls `login` and `connect` with error handling
    pub async fn run(self) {
        match self.login().await {
            Ok(true) => info!("Logged in successfully"),
            Ok(false) => error!("Failed to login"),
            Err(e) => {
                eprintln!("An error occured while trying to login: {}", e);

                if let Some(backtrace) = ErrorCompat::backtrace(&e) {
                    eprintln!("{}", backtrace);
                }
            }
        }
        
        match self.connect().await {
            Ok(true) => info!("Connected to gateway successfully"),
            Ok(false) => error!("Failed to connect"),
            Err(e) => {
                eprintln!("An error occured while trying to connect: {}", e);

                if let Some(backtrace) = ErrorCompat::backtrace(&e) {
                    eprintln!("{}", backtrace);
                }
            }
        }

        println!("Running: {:#?}", self);
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
            http: HttpClient::new(token.clone())
        }
    }
}
