// What i want to achieve
// #[listener(ListenerType::MessageCreate)]
// async fn on_message(message) {
//  message.channel.send("whatever");
// }
//
// Client::builder()
// .token("TOKEN")
// .listener_closure(ListenerType::MessageCreate, |message| { message.channel.send("hi") })
// .listener(on_message)
// .build()
// .run()
// .await?)
