use serenity::client::Client;
use std::env;

trait Command {
    fn matches(_: &str) -> bool;
}

struct Pong;
impl Command for Pong {
    fn matches(some_str: &str) -> bool {
        some_str.starts_with(".ping")
    }
}

struct Handler;
impl EventHandler for Handler {}

const OWNER_ID: i64 = 5403774601187819941;

fn main() {
    // 1. Instantiate client
    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("Token missing"), Handler)
        .expect("Error creating client context");
    if let Err(why) = client.start() {
        eprintln!(why)
    }
}
