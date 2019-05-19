#[macro_use] extern crate lazy_static;
use serenity::{
    model::channel::Message,
    client::Client,
    prelude::{
        EventHandler, Context,
    },
};
use std::env;
use crossbeam_utils::thread::scope;
mod commands;

#[derive(Debug)]
pub struct CommandInfo<'a> {
    name: &'a str,
}

pub trait Command: Send + Sync {
    fn matches(&self, _: &str) -> bool;
    fn run(&self, _: &Context, _: &mut Message);
    fn report(&self) -> CommandInfo;
}

struct Pong;

impl Command for Pong {
    fn matches(&self, some_str: &str) -> bool {
        some_str.starts_with(".ping")
    }

    fn run(&self, ctx: &Context, msg: &mut Message) {
        loop {
            println!("{:?}", std::time::Instant::now())
        }
//        msg.reply("pong!").expect("failed to reply pong lol");
    }

    fn report(&self) -> CommandInfo {
        CommandInfo {
            name: "pong.",
        }
    }
}

struct Handler<'a>(Vec<&'a Command>);

impl<'a> Handler<'a> {
    fn get_matching_command(&'a self, message_content: &'a str)
                            -> Option<&'a Command> {
        self.0.iter().find(|&cmd|
            cmd.matches(message_content)).map(|c| *c)
    }
}

impl<'a> EventHandler for Handler<'a> {
    fn message(&self, context: Context, new_message: Message) {
        if new_message.is_own() {
            let message_content = &new_message.content_safe();
            if let Some(command) = self.get_matching_command(message_content) {
                let mut message = new_message;
                let result = scope(|_| command.run(&context, &mut message));
                if let Err(why) = result {
                    println!("{:?}", why);
                }
            }
        }
    }
}


fn main() {
    // 0. Instantiate commands
    let handler = Handler {
        0: vec![
            &Pong, &commands::TextWindow,
        ],
    };
    // 1. Instantiate client
    let mut client = Client::new(
        &env::var("DISCORD_TOKEN").expect("Token missing"),
        &"relay alpha 1".to_string(), handler)
        .expect("Error creating client context");
    // 2. Go crazy
    if let Err(why) = client.start() {
        eprintln!("{}", why);
    }
}
