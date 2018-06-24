extern crate rusty_butler_lib;
use rusty_butler_lib::plugins;

extern crate irc;
use irc::client::prelude::*;

fn main() {
    let config = Config::load("bot.toml").expect("Failed to load config");
    let mut reactor = IrcReactor::new().expect("Failed to create IrcReactor");
    let client = reactor
        .prepare_client_and_connect(&config)
        .expect("Failed to create client");
    client.identify().expect("Failed to identify");
    reactor.register_client_with_handler(client, |client, irc_msg| {
        plugins::print_msg(&irc_msg);
        plugins::beep_boop(client, &irc_msg);
        plugins::time::handler(client, &irc_msg);
        plugins::strava::handler(client, &irc_msg);
        Ok(())
    });
    reactor.run().expect("Failed to run IrcReactor");
}
