pub mod models;

use crate::models::Token;

#[derive(Debug)]
struct Config {
    token: Option<Token>
}

#[derive(Debug)]
pub struct Client {}

#[derive(Debug)]
pub struct ClientBuilder {
    config: Config
}


impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new() 
    }

    pub async fn run(&self) {
        println!("running");
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
        Client {}
    }
}
