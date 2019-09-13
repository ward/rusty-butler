extern crate rusty_butler_lib;
use rusty_butler_lib::plugins;
use rusty_butler_lib::plugins::Handler;
use rusty_butler_lib::plugins::MutableHandler;

extern crate irc;
use irc::client::prelude::*;
use std::sync::Mutex;

extern crate clap;
use clap::{App, Arg};

fn main() {
    let matches = App::new("rusty-butler")
        .version("0.1.0")
        .author("Ward Muylaert")
        .about("An IRC bot. The world needed more of those.")
        .arg(
            Arg::with_name("config")
                .long("config")
                .value_name("FILE")
                .help("Use a different configuration file")
                .default_value("bot.toml"),
        )
        .get_matches();
    let config_file_name = matches.value_of("config").unwrap();
    let config = Config::load(config_file_name).expect("Failed to load config");
    let mut reactor = IrcReactor::new().expect("Failed to create IrcReactor");
    let client = reactor
        .prepare_client_and_connect(&config)
        .expect("Failed to create client");
    client.identify().expect("Failed to identify");
    // Note: because of the move there, the register_client_with_handler takes
    // ownership of `config` so we cannot use it afterwards anymore!
    // Don't think we care to use it again (for now) anyway.
    let mut handlers: Vec<Box<dyn Handler>> = vec![];
    handlers.push(Box::new(plugins::strava::StravaHandler::new(&config)));
    handlers.push(Box::new(plugins::time::TimeHandler::new()));
    let mut mutable_handlers: Vec<Mutex<Box<dyn MutableHandler>>> = vec![];
    mutable_handlers.push(Mutex::new(Box::new(
        plugins::nickname::NicknameHandler::new(&config),
    )));
    mutable_handlers.push(Mutex::new(Box::new(plugins::calc::CalcHandler::new())));
    mutable_handlers.push(Mutex::new(Box::new(
        plugins::lastseen::LastSeenHandler::new(),
    )));
    mutable_handlers.push(Mutex::new(Box::new(plugins::elo::EloHandler::new())));
    reactor.register_client_with_handler(client, move |client, irc_msg| {
        plugins::print_msg(&irc_msg);
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
