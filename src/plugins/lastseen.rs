use chrono::prelude::{DateTime, Utc};
use irc::client::prelude::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct LastSeenHandler {
    events: HashMap<String, LastSeenEvent>,
}
#[derive(Debug)]
struct LastSeenEvent {
    when: DateTime<Utc>,
    what: Command,
}
impl LastSeenHandler {
    pub fn new() -> LastSeenHandler {
        LastSeenHandler {
            events: HashMap::new(),
        }
    }

    fn log(&mut self, msg: &Message) {
        match msg.command {
            Command::PRIVMSG(_, _)
            | Command::NOTICE(_, _)
            | Command::JOIN(_, _, _)
            | Command::PART(_, _)
            | Command::QUIT(_)
            | Command::NICK(_)
            | Command::TOPIC(_, _) => {
                if let Some(nick) = msg.source_nickname() {
                    if !(nick.eq_ignore_ascii_case("nickserv")
                        || nick.eq_ignore_ascii_case("freenode-connect"))
                    {
                        let nick = nick.to_owned();
                        let command = msg.command.clone();
                        let event = LastSeenEvent {
                            when: Utc::now(),
                            what: command,
                        };
                        self.events.insert(nick, event);
                    }
                }
            }
            _ => {}
        }
    }

    fn find_event<'a>(&'a self, nick: &str) -> Option<&'a LastSeenEvent> {
        self.events.get(nick)
    }
}
impl super::MutableHandler for LastSeenHandler {
    fn handle(&mut self, _client: &IrcClient, msg: &Message) {
        self.log(msg);
        println!("{:?}", self);
        // TODO: "!(last)seen nick" command
        // TODO: Think up a way to make this info accessible for other plugins
    }
}
