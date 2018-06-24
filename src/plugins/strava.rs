use irc::client::prelude::*;
use regex::Regex;
use reqwest;

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

#[derive(Deserialize, Debug)]
struct Activity {
    name: String,
    #[serde(rename = "type")]
    sport: String,
    distance: f64,
    total_elevation_gain: f64,
    moving_time: u32,
}
impl Activity {
    fn fetch(id: &str) -> Result<Activity, reqwest::Error> {
        let strava_token = "";
        let url = format!("https://www.strava.com/api/v3/activities/{}?access_token={}", id, strava_token);
        let mut req = reqwest::get(&url)?;
        println!("{}", req.url());
        req.json()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stuff() {
        println!("{:?}", Activity::fetch("1658946676"));
        panic!("Stop!");
    }
}
