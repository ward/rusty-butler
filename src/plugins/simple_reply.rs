use irc::client::prelude::*;
use rand::seq::SliceRandom;
use regex::Regex;

pub struct SimpleReplyHandler {
    replies: Vec<SimpleReply>,
}

impl SimpleReplyHandler {
    pub fn new() -> SimpleReplyHandler {
        // TODO: Make this something that reads in from a .toml file or so
        let replies = vec![
            SimpleReply {
                regex: Regex::new(r"^(?i)!esl$").unwrap(),
                replies: vec![
                    "DREAMS CAN'T BE BUY",
                    "This Super League business is a betrayal to the memory of Prince Philip",
                    "The traitors: Arsenal, Chelsea, Liverpool, Manchester City, Manchester United, Tottenham, AC Milan, Atletico Madrid, Barcelona, Inter Milan, Juventus, and Real Madrid",
                    "where were you when football was kill",
                    "positive message about ESL"
                ],
            },
            SimpleReply {
                regex: Regex::new(r"^(?i)!uptime$").unwrap(),
                replies: vec!["Mitzy who?", "Mitzy where?"],
            },
            SimpleReply {
                regex: Regex::new(r"^(?i)!stats$").unwrap(),
                replies: vec!["Three months: https://irc.wxm.be/stats/reddit-soccer.html; All time: https://irc.wxm.be/stats/reddit-soccer.all.html"]
            }
        ];
        SimpleReplyHandler { replies }
    }

    fn matcher(&self, message: &str) -> Option<String> {
        for reply in &self.replies {
            if reply.regex.is_match(message) {
                return reply.get_reply();
            }
        }
        None
    }
}

impl super::Handler for SimpleReplyHandler {
    fn handle(&self, client: &Client, msg: &Message) {
        if let Command::PRIVMSG(ref channel, ref message) = msg.command {
            if let Some(result) = self.matcher(message) {
                client.send_privmsg(&channel, &result).unwrap();
            }
        }
    }
}

impl super::help::Help for SimpleReplyHandler {
    fn name(&self) -> String {
        String::from("simply_reply")
    }

    fn help(&self) -> Vec<super::help::HelpEntry> {
        let result = vec![super::help::HelpEntry::new(
            "various text triggers",
            "Various replies that require but a static string",
        )];
        result
    }
}

impl Default for SimpleReplyHandler {
    fn default() -> Self {
        Self::new()
    }
}

struct SimpleReply {
    regex: Regex,
    replies: Vec<&'static str>,
}
impl SimpleReply {
    fn get_reply(&self) -> Option<String> {
        if self.replies.len() == 1 {
            // Shortcut if there is no choice to be made
            self.replies.get(1).map(|&s| s.to_owned())
        } else if let Some(&choice) = self.replies.choose(&mut rand::thread_rng()) {
            Some(choice.to_owned())
        } else {
            eprintln!("Failed to choose a reply after matching {:#?}", self.regex);
            None
        }
    }
}
