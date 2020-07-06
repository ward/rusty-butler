use rusty_butler_lib::plugins;
use rusty_butler_lib::plugins::Handler;
use rusty_butler_lib::plugins::MutableHandler;

use futures::prelude::*;
use irc::client::prelude::*;
use std::sync::Mutex;

use clap::{App, Arg};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    let config_for_handlers = Config::load(config_file_name).expect("Failed to load config");

    let mut client = Client::from_config(config).await?;
    client.identify().expect("Failed to identify");
    let mut stream = client.stream()?;

    let mut handlers: Vec<Box<dyn Handler>> = vec![];
    handlers.push(Box::new(plugins::strava::StravaHandler::new(
        &config_for_handlers,
    )));
    handlers.push(Box::new(plugins::time::TimeHandler::new()));
    let mut mutable_handlers: Vec<Mutex<Box<dyn MutableHandler>>> = vec![];
    mutable_handlers.push(Mutex::new(Box::new(
        plugins::nickname::NicknameHandler::new(&config_for_handlers),
    )));
    mutable_handlers.push(Mutex::new(Box::new(plugins::calc::CalcHandler::new())));
    mutable_handlers.push(Mutex::new(Box::new(
        plugins::lastseen::LastSeenHandler::new(),
    )));
    mutable_handlers.push(Mutex::new(Box::new(plugins::elo::EloHandler::new())));

    // Note: because of the move there, the register_client_with_handler takes
    // ownership of `config` so we cannot use it afterwards anymore!
    // Don't think we care to use it again (for now) anyway.
    // reactor.register_client_with_handler(client, move |client, irc_msg| {
    while let Some(irc_msg) = stream.next().await.transpose()? {
        plugins::print_msg(&irc_msg);
        for handler in &handlers {
            handler.handle(&client, &irc_msg);
        }
        for mutable_handler in &mutable_handlers {
            // TODO Is there a possibility of this slowing things down in unforseen ways?
            let mut mutable_handler = mutable_handler.lock().expect("Likely fatal! Getting a lock failed which implies another thread holding the lock panicked");
            mutable_handler.handle(&client, &irc_msg);
        }
    }
    // reactor.run().expect("Failed to run IrcReactor");

    Ok(())
}
