pub mod models;

use log::{info, trace, warn, debug, error};
use snafu::{ensure, Backtrace, ErrorCompat, ResultExt, Snafu};
use rustc_version_runtime::version;

use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client as ReqwestClient,
    Result as ReqResult,
    Method, Response, StatusCode, Url,
};

use crate::models::Token;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Invalid token was passed: {:?}", token))]
    InvalidToken { token: Token }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

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

    pub async fn request(&self, route: Route) -> ReqResult<Response> {
        self.req_client.execute(
            self.req_client.request(route.method, route.url).build()?
        ).await
    }

    pub async fn static_login() -> Result<()> {Ok(())}
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
        // InvalidToken{ token: Token("e") }.fail()
        Ok(true)
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
    pub fn token(mut self, token: &'static str) -> ClientBuilder {
        self.config.token = Some(Token(&token));
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
