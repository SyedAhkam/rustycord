use rustycord::Client;

#[tokio::main]
async fn main() {
    env_logger::init();

    Client::builder()
        .token_from_env("DISCORD_TOKEN") // you can change this key as you prefer
        .build()
        .run()
        .await;
}
