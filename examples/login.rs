use rustycord::{
    Client,
    RustyCordResult
};

#[tokio::main]
async fn main() -> RustyCordResult<()> {
    let mut client = Client::new();

    client
        .run("NzY4MjA0MzkxNTU1Mzk5NzIy.X49EFw.2Bk5AFeVsRDWXh-14Y8Wamaszbk")
        .await?;

    println!("{:#?}", client);

    Ok(())
}
