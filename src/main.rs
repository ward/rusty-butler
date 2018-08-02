extern crate rusty_butler_lib;
use rusty_butler_lib::plugins;
use rusty_butler_lib::plugins::Handler;
use rusty_butler_lib::plugins::MutableHandler;

extern crate irc;
use irc::client::prelude::*;
use std::sync::Mutex;

fn main() {
    let config = Config::load("bot.toml").expect("Failed to load config");
    let mut reactor = IrcReactor::new().expect("Failed to create IrcReactor");
    let client = reactor
        .prepare_client_and_connect(&config)
        .expect("Failed to create client");
    client.identify().expect("Failed to identify");
    // Note: because of the move there, the register_client_with_handler takes
    // ownership of `config` so we cannot use it afterwards anymore!
    // Don't think we care to use it again (for now) anyway.
    let mut handlers: Vec<Box<Handler>> = vec![];
    handlers.push(Box::new(plugins::strava::StravaHandler::new(&config)));
    handlers.push(Box::new(plugins::time::TimeHandler::new()));
    handlers.push(Box::new(plugins::calc::CalcHandler::new()));
    let mut mutable_handlers: Vec<Mutex<Box<MutableHandler>>> = vec![];
    mutable_handlers.push(Mutex::new(Box::new(plugins::nickname::NicknameHandler::new(&config))));
    reactor.register_client_with_handler(client, move |client, irc_msg| {
        plugins::print_msg(&irc_msg);
        plugins::beep_boop(client, &irc_msg);
        for handler in &handlers {
            handler.handle(client, &irc_msg);
        }
        for mutable_handler in &mutable_handlers {
            // TODO Is there a possibility of this slowing things down in unforseen ways?
            let mut mutable_handler = mutable_handler.lock().expect("Likely fatal! Getting a lock failed which implies another thread holding the lock panicked");
            mutable_handler.handle(client, &irc_msg);
        }
        Ok(())
    });
    reactor.run().expect("Failed to run IrcReactor");
}
