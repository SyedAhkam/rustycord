pub mod models;

use snafu::{ensure, Backtrace, ErrorCompat, ResultExt, Snafu};
use log::{info, trace, warn, debug, error};

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
pub struct HttpClient {
    token: Token
}

impl HttpClient {
    pub fn new(token: Token) -> Self {
        Self { token }
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
            Ok(true) => info!("Connected to WS successfully"),
            Ok(false) => error!("Failed to connect"),
            Err(e) => {
                eprintln!("An error occured while trying to connect: {}", e);

                if let Some(backtrace) = ErrorCompat::backtrace(&e) {
                    eprintln!("{}", backtrace);
                }
            }
        }

        println!("Running");
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
