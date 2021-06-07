pub mod models;

use snafu::{ensure, Backtrace, ErrorCompat, ResultExt, Snafu};

use crate::models::Token;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("Invalid token was passed: {}", token))]
    InvalidToken { token: Token }
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
struct Config {
    token: Option<Token>
}

#[derive(Debug)]
pub struct HttpClient {}

impl HttpClient {
    pub fn new() -> Self {
        Self {}
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
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new() 
    }

    pub async fn run(&self) {
        println!("running: {:?}", self.http);
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self { 
            config: Config {
                token: None
            }
        }
    }

    pub fn token(mut self, token: &'static str) -> ClientBuilder {
        self.config.token = Some(Token(&token));
        self
    }

    pub fn build(self) -> Client {
        let http_client = HttpClient::new();

        Client {
            http: http_client
        }
    }
}
