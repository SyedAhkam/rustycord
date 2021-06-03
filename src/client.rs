use crate::{
    RustyCordResult,
    http::HTTPClient,
    errors::{ClientException}
};

#[derive(Clone, Debug)]
pub struct Client<'a> {
    http: Option<HTTPClient<'a>>
}

pub struct Token(String);

impl Client<'_> {
    pub fn new() -> Self {
       Client{
            http: None
       } 
    }

    pub async fn login(&self, token: &str) -> RustyCordResult<()> {
        self.http.as_ref().unwrap().static_login(token).await?;

        Ok(())
    }

    pub async fn connect(&self, token: &str) -> RustyCordResult<()> {
        // Do something
        Ok(())
    }

    pub async fn run(&mut self, token: &'static str) -> RustyCordResult<()> {
        
        self.http = Some(HTTPClient::new(&token));

        self.login(&token).await.unwrap();
        self.connect(&token).await.unwrap();

        Ok(())
    }
}
