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
    // Note: because of the move there, the register_client_with_handler takes
    // ownership of `config` so we cannot use it afterwards anymore!
    // Don't think we care to use it again (for now) anyway.
    reactor.register_client_with_handler(client, move |client, irc_msg| {
        plugins::print_msg(&irc_msg);
        plugins::beep_boop(client, &irc_msg);
        plugins::time::handler(client, &irc_msg);
        plugins::strava::handler(client, &irc_msg, &config);
        Ok(())
    });
    reactor.run().expect("Failed to run IrcReactor");
}
