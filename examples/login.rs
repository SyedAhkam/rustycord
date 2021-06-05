use rustycord::{
    Client,
    Result,
    client::Error
};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Client::new();

    client
        .run("TOKEN")
        .await?;

    println!("{:#?}", client);

    Ok(())
}
