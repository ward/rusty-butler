use chrono::prelude::{DateTime, Utc};
use irc::client::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::fmt;

// TODO: Think up a way to make this info accessible for other plugins
// TODO: Want to store this on disk and read it at start up in case of crashes
//       However, serde does not readily handle DateTime and Command :(

#[derive(Debug)]
pub struct LastSeenHandler {
    events: HashMap<String, LastSeenEvent>,
    seen_matcher: Regex,
}
#[derive(Debug)]
struct LastSeenEvent {
    when: DateTime<Utc>,
    what: Command,
}
impl fmt::Display for LastSeenEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: Default time format is too detailed
        // TODO: Debug format of self.what gives too much info
        write!(
            f,
            "Last seen at {when} doing {what}",
            when = self.when.format("%Y-%m-%d %H:%M:%S %Z").to_string(),
            what = format!("{:?}", self.what),
        )
    }
}
impl LastSeenHandler {
    pub fn new() -> LastSeenHandler {
        let seen_matcher = Regex::new(r"^!(?:last)?seen +(.+) *$").unwrap();
        LastSeenHandler {
            events: HashMap::new(),
            seen_matcher,
        }
    }

    /// Given an IRC Message, considers whether it should be logged for the user that triggered
    /// this Message.
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

    fn seen_trigger(&self, msg: &str) -> Option<String> {
        if let Some(capture) = self.seen_matcher.captures(msg) {
            Some(capture.get(1).unwrap().as_str().to_string())
        } else {
            None
        }
    }
}
impl super::MutableHandler for LastSeenHandler {
    fn handle(&mut self, client: &Client, msg: &Message) {
        // "!(last)seen nick" command
        if let Command::PRIVMSG(ref channel, ref message) = msg.command {
            if let Some(nick) = self.seen_trigger(message) {
                if let Some(event) = self.find_event(&nick) {
                    client.send_privmsg(&channel, &event.to_string()).unwrap();
                }
            }
        }
        self.log(msg);
    }
}

impl super::help::Help for LastSeenHandler {
    fn name(&self) -> String {
        String::from("seen")
    }

    fn help(&self) -> Vec<super::help::HelpEntry> {
        let mut result = vec![];
        result.push(super::help::HelpEntry::new(
            "!seen NICK",
            "Check what I saw NICK most recently do.",
        ));
        result
    }
}
