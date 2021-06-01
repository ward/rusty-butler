use irc::client::prelude::*;
use rand::seq::SliceRandom;

pub struct SimpleReplyHandler {
    replies: Vec<SimpleReply>,
}

impl SimpleReplyHandler {
    pub fn new(config: &super::config::Config) -> SimpleReplyHandler {
        let mut replies = vec![];
        for (_name, reply_config) in &config.simple_reply.replies {
            let reply = SimpleReply {
                triggers: reply_config.triggers.clone(),
                replies: reply_config.replies.clone(),
            };
            replies.push(reply);
        }
        SimpleReplyHandler { replies }
    }

    fn matcher(&self, message: &str) -> Option<String> {
        for reply in &self.replies {
            if reply.triggered(message) {
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

struct SimpleReply {
    triggers: Vec<String>,
    replies: Vec<String>,
}

impl SimpleReply {
    /// Check whether any of the triggers are matched by msg
    fn triggered(&self, msg: &str) -> bool {
        self.triggers
            .iter()
            .any(|trigger| trigger.eq_ignore_ascii_case(msg))
    }

    /// Returns a random reply
    fn get_reply(&self) -> Option<String> {
        if self.replies.len() == 1 {
            // Shortcut if there is no choice to be made
            self.replies.get(1).map(|s| s.to_owned())
        } else if let Some(choice) = self.replies.choose(&mut rand::thread_rng()) {
            Some(choice.to_owned())
        } else {
            eprintln!(
                "Failed to choose a reply after matching {:#?}",
                self.triggers
            );
            None
        }
    }
}
