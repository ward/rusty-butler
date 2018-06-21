extern crate irc;

use irc::client::prelude::*;

fn main() {
    let config = Config::load("bot.toml").expect("Failed to load config");
    let mut reactor = IrcReactor::new().unwrap();
    let client = reactor.prepare_client_and_connect(&config)
        .expect("Failed to create client");
    client.identify().expect("Failed to identify");
    reactor.register_client_with_handler(client, |client, irc_msg| {
        println!("{}", irc_msg);
        if let Command::PRIVMSG(channel, message) = irc_msg.command {
            if message.contains(client.current_nickname()) {
                // send_privmsg comes from ClientExt
                client.send_privmsg(&channel, "beep boop").unwrap();
            }
        }
        Ok(())
    });
    reactor.run().unwrap();
}
