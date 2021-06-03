use rustycord::{
    Client,
    RustyCordResult
};

#[tokio::main]
async fn main() -> RustyCordResult<()> {
    let mut client = Client::new();

    client
        .run("TOKEN")
        .await?;

    Ok(())
}
