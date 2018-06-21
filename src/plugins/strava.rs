use irc::client::prelude::*;
use regex::Regex;

pub fn handler(client: &IrcClient, msg: &Message) {
    if let Command::PRIVMSG(ref channel, ref message) = msg.command {
        let activity_reply = handle_activities(message);
        match activity_reply {
            Some(activity_id) => client.send_privmsg(&channel, activity_id).unwrap(),
            _ => (),
        }
    }
}

fn handle_activities(msg: &str) -> Option<&str> {
    let activity_regex = Regex::new(r"https?:\/\/www\.strava\.com\/activities\/(\d+)").unwrap();
    for captures in activity_regex.captures_iter(msg) {
        return Some(captures.get(1).unwrap().as_str());
    }
    None
}
