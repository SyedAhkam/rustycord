use crate::{
    RustyCordResult,
    RustyCordError,
    http::HTTPClient,
};

#[derive(Clone, Debug)]
pub struct Client<'a> {
    http: Option<HTTPClient<'a>>
}

impl Client<'_> {
    pub fn new() -> Self {
       Client{
            http: None
       } 
    }

    pub async fn login(&mut self) -> RustyCordResult<()> {
        self.http.as_mut().unwrap().static_login().await?;

        Ok(())
    }

    pub async fn connect(&mut self, token: &str) -> RustyCordResult<()> {
        // Do something

        Ok(())
    }

    pub async fn run(&mut self, token: &'static str) -> RustyCordResult<()> {
        
        self.http = Some(HTTPClient::new(&token));

        {
            self.login().await?;
            self.connect(&token).await?;
        }

        Ok(())
    }
}
