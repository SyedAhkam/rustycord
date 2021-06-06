use rustycord::{
    Client,
};

#[tokio::main]
async fn main() {
    let client = Client::builder()
        .token("TOKEN")
        .build()
        .run()
        .await;

    println!("{:#?}", client);
}
