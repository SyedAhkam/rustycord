use rustycord::{
    Client,
    Result
};

#[tokio::main]
async fn main() {
    Client::builder()
        .token("TOKEN")
        .build()
        .run()
        .await;
}
