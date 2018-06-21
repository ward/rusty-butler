extern crate irc;

pub mod plugins {
    use irc::client::prelude::*;

    pub fn print_msg(msg: &Message) {
        println!("{}", msg);
    }

    pub fn beep_boop(client: &IrcClient, msg: &Message) {
        if let Command::PRIVMSG(ref channel, ref message) = msg.command {
            if message.contains(client.current_nickname()) {
                // send_privmsg comes from ClientExt
                client.send_privmsg(&channel, "beep boop").unwrap();
            }
        }
    }
}
