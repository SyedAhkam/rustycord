pub mod models;

use log::{info, debug, trace};
use snafu::{ErrorCompat, ResultExt, OptionExt, Snafu};
use rustc_version_runtime::version;

use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client as ReqwestClient,
    Result as ReqResult,
    Error as ReqError,
    Method, Response, StatusCode, Url,
};

use tokio_tungstenite::{
    tungstenite,
    WebSocketStream,
    MaybeTlsStream,
};

use serde_json;

use tokio::{
    net::TcpStream,
    sync::Mutex,
};

use futures_util::{SinkExt, stream::StreamExt};

use std::env;
use std::sync::Arc;

use crate::models::{
    Token,
    DiscordError,
    Gateway,
    BotGateway,
    GatewayMessage,
    GatewayOpcode,
    Payload,
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

    #[snafu(display("Failed to connect to gateway url {}", url))]
    GatewayConnectError { url: String, source: tungstenite::Error },

    #[snafu(display("Failed to read socket message"))]
    GatewayMessageRead,

    #[snafu(display("Failed to send socket message: {:?}", message))]
    GatewayMessageSend { message: tungstenite::Message, source: tungstenite::Error }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

const BASE_URL: &str = "https://discord.com/api/v9";
const USER_ENDPOINT: &str = "/users";

const DEFAULT_GATEWAY: &str = r#"{"url": "wss://gateway.discord.gg""#;
const DEFAULT_BOT_GATEWAY: &str = r#"{"url": "wss://gateway.discord.gg""#;

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

    /// Makes a http request
    async fn request(&self, route: Route) -> ReqResult<Response> {
        self.req_client.execute(
            self.req_client.request(route.method, route.url).build()?
        ).await
    }

    /// Constructs an error according to the status code
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

    /// Inspects the recieved response for any potential errors
    async fn inspect_response(&self, data: &str, status: StatusCode) -> Result<()> {
        if let Ok(err_resp) = serde_json::from_str::<DiscordError>(data) {
            self.construct_error(status, err_resp).await?;
        }

        Ok(())
    }

    /// Returns `/gateway` response text
    async fn fetch_gateway(&self) -> Result<String> {
        let resp = self.request(Route {
            method: Method::GET,
            url: Url::parse(format!("{}/gateway", BASE_URL).as_str()).unwrap()
        })
            .await
            .context(RequestSend { endpoint: "/gateway" })?;

        let status = resp.status();

        let data = resp.text().await.context(RequestParse { status })?;

        self.inspect_response(data.as_str(), status).await?;

        Ok(data)
    }
    
    /// Returns `/gateway/bot` response text
    async fn fetch_bot_gateway(&self) -> Result<String> {
        let resp = self.request(Route {
            method: Method::GET,
            url: Url::parse(format!("{}/gateway/bot", BASE_URL).as_str()).unwrap()
        })
            .await
            .context(RequestSend { endpoint: "/gateway/bot" })?;

        let status = resp.status();

        let data = resp.text().await.context(RequestParse { status })?;

        self.inspect_response(data.as_str(), status).await?;

        Ok(data)
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

type WsStream = Arc<Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>>;

#[derive(Debug)]
struct WsConnection(WsStream);

impl WsConnection {
    async fn read_next(&self) -> Result<tungstenite::Message> {
        trace!("locking stream to read next");
        let stream = self.0.lock();

        Ok(stream.await.next()
            .await
            .context(GatewayMessageRead)
            .unwrap()
            .unwrap()
        )
    }

    async fn send_msg(&mut self, msg: tungstenite::Message) -> Result<()> {
        let msg_string = msg.to_text().unwrap().to_string();

        trace!("locking stream to send msg");
        let stream = self.0.lock();

        stream.await
            .send(msg)
            .await
            .context(GatewayMessageSend { message: msg_string })?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct GatewayClient {
    url: String,
    connection: Option<WsConnection>,
    heartbeat_interval: Option<i32>
}

impl GatewayClient {
    pub fn new(url: String) -> Self {
        Self { url, connection: None, heartbeat_interval: None }
    }

    async fn get_connection(&self) -> Result<WsConnection, tungstenite::Error> {
        let (ws_stream, _) = tokio_tungstenite::connect_async(self.url.clone()).await?;

        Ok(WsConnection(Arc::new(Mutex::new(ws_stream))))
    }
    
    async fn send_heartbeat(&mut self) -> Result<()> {
        self.connection.as_mut().unwrap().send_msg(tungstenite::Message::Text(
            serde_json::json!({
                "op": 1,
                "d": serde_json::json!(null)
            }).to_string()
        )).await?;

        Ok(())
    }

    async fn start_sending_heartbeats(&mut self) -> Result<()> {
        /* self.connection */
            // .as_mut()
            // .unwrap()
            // .send_continously(tungstenite::Message::Text(
            //     serde_json::json!({
            //         "op": 1,
            //         "d": serde_json::json!(null)
            //     }).to_string()
            // ), self.heartbeat_interval.unwrap() as u64)
            /* .await; */

        // tokio::spawn(async { loop {println!("hi");} });
        
        
        /* let mut connection = self.connection.as_mut().unwrap().clone(); */
        // let heartbeat_interval = self.heartbeat_interval.unwrap() as u64;
        //
        // tokio::spawn(async move {
        //     let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(heartbeat_interval));
        //
        //     loop {
        //         interval.tick().await;
        //         println!("sending");
        //
        //         connection
        //             .send_msg(tungstenite::Message::Text(
        //                 serde_json::json!({
        //                     "op": 1,
        //                     "d": serde_json::json!(null)
        //                 }).to_string()
        //             )).await;
        //     }
        /* }); */

        Ok(())
    }

    async fn recieve_hello(&mut self) -> Result<()> {
        let message = self.connection.as_ref().unwrap().read_next().await?;
        let message_str = message.to_text().unwrap();

        let gateway_message = serde_json::from_str::<GatewayMessage>(message_str)
            .context(DeserializeError { text: message_str, target: "GatewayMessage" })?;
        
        debug!("Recieved hello: {:?}", gateway_message);

        let Payload::Hello(payload) = gateway_message.d.unwrap();
        self.heartbeat_interval = Some(payload.heartbeat_interval);

        Ok(())
    }

    pub async fn start(&mut self) -> Result<()> {
        self.recieve_hello().await?;
        self.start_sending_heartbeats().await?;

        Ok(())
    }

    pub async fn connect(&mut self) -> Result<()> {
        debug!("Attempting to connect to: {}", self.url);

        self.connection = Some(self.get_connection()
            .await
            .context(GatewayConnectError { url: self.url.clone() })?
        );

        Ok(())
    }
}

#[derive(Debug)]
pub struct Client {
    http: HttpClient,
    gateway: Option<GatewayClient>,
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

    /// Fetches a `Gateway`
    async fn fetch_gateway(&self) -> Result<Gateway> {
        let gateway_text = match self.http.fetch_gateway().await {
            Ok(gateway) => gateway,
            Err(_) => DEFAULT_GATEWAY.to_string()
        };
        debug!("recieved gateway: {}", gateway_text);

        Ok(
            serde_json::from_str::<Gateway>(gateway_text.as_str())
                .context(DeserializeError { text: gateway_text, target: "Gateway" })?
        )
    }
    
    /// Fetches a `BotGateway`
    pub async fn fetch_bot_gateway(&self) -> Result<BotGateway> {
        let bot_gateway_text = match self.http.fetch_bot_gateway().await {
            Ok(gateway) => gateway,
            Err(_) => DEFAULT_BOT_GATEWAY.to_string()
        };
        debug!("recieved bot gateway: {}", bot_gateway_text);

        Ok(
            serde_json::from_str::<BotGateway>(bot_gateway_text.as_str())
                .context(DeserializeError { text: bot_gateway_text, target: "BotGateway" })?
        )
    }

    /// Logs in using a static token
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
    async fn connect(&mut self) -> Result<()> {
        let mut gateway = self.fetch_gateway().await?;
        gateway.url = format!("{}/?v=9&encoding=json", gateway.url);

        self.gateway = Some(GatewayClient::new(gateway.url));

        self.gateway.as_mut().unwrap().connect().await?;
        self.gateway.as_mut().unwrap().start().await?;

        Ok(())
    }

    /// Calls `login` and `connect` with error handling
    /// This is a blocking call, register event handlers before calling this function
    pub async fn run(mut self) {
        match self.login().await {
            Ok(_) => info!("Logged in successfully"),
            Err(e) => {
                eprintln!("An error occured while trying to login: {}", e);

                if let Some(backtrace) = ErrorCompat::backtrace(&e) {
                    eprintln!("{}", backtrace);
                }
                return
            }
        }
        
        match self.connect().await {
            Ok(_) => info!("Connected to gateway successfully"),
            Err(e) => {
                eprintln!("An error occured in gateway connection: {}", e);

                if let Some(backtrace) = ErrorCompat::backtrace(&e) {
                    eprintln!("{}", backtrace);
                }
            }
        }

        // Blocks until all pending tasks complete
        std::future::pending::<()>().await;
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
            gateway: None,
            user: None
        }
    }
}
