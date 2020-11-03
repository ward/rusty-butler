use irc::client::prelude::*;
use regex::Regex;

pub mod api;

pub struct UntappdHandler {
    client_id: Option<String>,
    client_secret: Option<String>,
    untappd_matcher: Regex,
}

impl UntappdHandler {
    /// Create UntappdHandler using a valid irc config. Requires untappd_client_id and
    /// untappd_client_secret to be set in the options section.
    pub fn new(config: &Config) -> Self {
        let untappd_matcher = Regex::new(r"^!(?:untappd|beer) (.*)$").unwrap();
        match (
            config.options.get("untappd_client_id"),
            config.options.get("untappd_client_secret"),
        ) {
            (Some(client_id), Some(client_secret)) => Self {
                client_id: Some(client_id.to_owned()),
                client_secret: Some(client_secret.to_owned()),
                untappd_matcher,
            },
            _ => {
                println!("Missing untappd_client_id or untappd_client_secret in options section, disabling plugin");
                Self {
                    client_id: None,
                    client_secret: None,
                    untappd_matcher,
                }
            }
        }
    }
}

// TODO
// - optional @number for search

impl super::Handler for UntappdHandler {
    fn handle(&self, client: &Client, msg: &Message) {
        if self.client_id.is_none() || self.client_secret.is_none() {
            return;
        }
        if let Command::PRIVMSG(ref channel, ref message) = msg.command {
            if let Some(captures) = self.untappd_matcher.captures(message) {
                if let Some(query) = captures.get(1) {
                    let query = query.as_str();
                    let beers = api::search(
                        query,
                        self.client_id.as_ref().unwrap(),
                        self.client_secret.as_ref().unwrap(),
                    );
                    if beers.is_empty() {
                        super::send_privmsg(client, channel, "Your query returned no results");
                    } else if beers.len() == 1 {
                        super::send_privmsg(client, channel, &beers[0].to_irc());
                    } else {
                        super::send_privmsg(
                            client,
                            channel,
                            &format!(
                                "{} --- {} more results",
                                &beers[0].to_irc(),
                                beers.len() - 1,
                            ),
                        );
                    }
                }
            }
        }
    }
}

impl super::help::Help for UntappdHandler {
    fn name(&self) -> String {
        String::from("untappd")
    }

    fn help(&self) -> Vec<super::help::HelpEntry> {
        let mut result = vec![];
        result.push(super::help::HelpEntry::new(
            "!untappd searchterm",
            "Search for beer matching your search term.",
        ));
        result.push(super::help::HelpEntry::new(
            "!beer searchterm",
            "Search for beer matching your search term.",
        ));
        // result.push(super::help::HelpEntry::new(
        //     "@NUMBER",
        //     "Modifier for your search, return the NUMBERth result.",
        // ));
        result
    }
}
