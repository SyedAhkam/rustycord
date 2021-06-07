use rustycord::{
    Client,
};

#[tokio::main]
async fn main() {
    Client::builder()
        .token("TOKEN")
        .build()
        .run()
        .await;
}
