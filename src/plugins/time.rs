use chrono::prelude::{DateTime, Utc};
use irc::client::prelude::*;

pub struct TimeHandler {}

impl TimeHandler {
    pub fn new() -> TimeHandler {
        TimeHandler {}
    }

    fn matcher(msg: &str) -> bool {
        msg.eq_ignore_ascii_case("!time")
            || msg.eq_ignore_ascii_case("!gmt")
            || msg.eq_ignore_ascii_case("!utc")
            || msg.eq_ignore_ascii_case("!now")
    }
}

impl super::Handler for TimeHandler {
    fn handle(&self, client: &IrcClient, msg: &Message) {
        if let Command::PRIVMSG(ref channel, ref message) = msg.command {
            if TimeHandler::matcher(message) {
                let now: DateTime<Utc> = Utc::now();
                let now = now.format("It is currently %A %d %B %Y %H:%M:%S UTC")
                    .to_string();
                let now = if message.eq_ignore_ascii_case("!gmt") {
                    String::from("Lol GMT, get with the times, grandpa. ") + &now
                } else {
                    now
                };
                client.send_privmsg(&channel, &now).unwrap();
            }
        }
    }
}