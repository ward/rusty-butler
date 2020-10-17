use chrono::prelude::*;
use chrono::Duration;
use football::*;
use irc::client::prelude::*;
use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

mod query;
mod toirc;
use toirc::ToIrc;

// To do for good enough parity:
//
// - League shortcuts (!game epl, !game bel, ...)
// - !epl

const MAX_NUMBER_OF_GAMES: usize = 20;

/// Sends a message to a given target. If the message is longer than a certain length, the message
/// is split up (unicode safe) and individual messages are sent separately.
///
/// This is being tested here. If it works well enough, I probably should move it up to lib.rs.
///
/// TODO: Delay between messages and/or maximum number of messages. (Global delay eventually?)
///
/// TODO: Length is currently hardcoded, ideally this bases itself on what the IRC server can
/// handle.
fn send_privmsg(client: &irc::client::Client, target: &str, message: &str) {
    // If there is no need to split up, just send immediately
    if message.len() < 400 {
        client.send_privmsg(target, message).unwrap();
    } else {
        // Otherwise, split at safe points and loop over
        let message: Vec<_> = message.graphemes(true).collect();
        for chunk in message.chunks(400) {
            let to_send: String = chunk.concat();
            client.send_privmsg(target, to_send).unwrap();
        }
    }
}

pub struct GamesHandler {
    games: Football,
    cached_at: DateTime<Utc>,
    cache_threshold: Duration,
    query_matcher: Regex,
    empty_query_matcher: Regex,
    query_parser: query::Parser,
}

impl GamesHandler {
    pub fn new() -> Self {
        let games = match get_all_games() {
            Ok(games) => games,
            Err(e) => {
                eprintln!(
                    "Error while getting games, returning empty instead. Error: {}",
                    e
                );
                Default::default()
            }
        };
        let cached_at = Utc::now();
        let cache_threshold = Duration::minutes(2);
        let query_matcher = Regex::new(r"^!games? +(.+)$").unwrap();
        let empty_query_matcher = Regex::new(r"^!games? *$").unwrap();
        Self {
            games,
            cached_at,
            cache_threshold,
            query_matcher,
            empty_query_matcher,
            query_parser: query::Parser::new(),
        }
    }

    /// If incoming message looks like it should be a query, get the query from it. Otherwise None.
    fn get_query(&self, msg: &str) -> Option<String> {
        if let Some(captures) = self.query_matcher.captures(msg) {
            if let Some(input) = captures.get(1) {
                Some(input.as_str().trim().to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// An empty query is just a "!game" or "!games" command
    fn is_empty_query(&self, msg: &str) -> bool {
        self.empty_query_matcher.is_match(msg)
    }

    /// Update the list of games if cache is older than a certain threshold.
    ///
    /// TODO: Should/can this be async? Kick off an update while still using current stored
    /// results. Or perhaps if updating takes longer than x seconds, use old data.
    fn update(&mut self) {
        let now = Utc::now();
        if now - self.cached_at > self.cache_threshold {
            println!("Starting football games update...");
            match get_all_games() {
                Ok(new_games) => {
                    println!("Got football games update.");
                    self.games = new_games;
                    self.cached_at = now;
                }
                Err(e) => eprintln!("Failed to update football games. {}", e),
            }
        }
    }
}

impl super::MutableHandler for GamesHandler {
    fn handle(&mut self, client: &Client, msg: &Message) {
        if let Command::PRIVMSG(ref channel, ref message) = msg.command {
            let query = if message == "!epl" {
                // !epl shortshortcut (in future replace this with an alias plugin)
                self.get_query("!game England Premier League")
            } else {
                // Otherwise check for regular !game query
                self.get_query(message)
            };
            if let Some(query) = query {
                println!("Handling !games query: '{}'", query);
                self.update();
                let query = self.query_parser.from_message(&query);
                println!("Query parsed as: {:?}", query);
                let filtered = self.games.query(&query.just_query_string());
                let filtered = match query.time {
                    query::QueryTime::Today => filtered.today(),
                    query::QueryTime::Yesterday => filtered.yesterday(),
                    query::QueryTime::Tomorrow => filtered.tomorrow(),
                    query::QueryTime::Finished => filtered.ended(),
                    query::QueryTime::Live => filtered.live(),
                    query::QueryTime::Upcoming => filtered.upcoming(),
                };

                let result = if filtered.countries.is_empty() {
                    String::from("Your !games query returned no results.")
                } else {
                    filtered.to_irc()
                };

                println!("{}", result);

                let total_games: usize = filtered.number_of_games();
                if total_games > MAX_NUMBER_OF_GAMES {
                    let too_many_games_msg = format!(
                        "Too many games ({}). Showing first {}.",
                        total_games, MAX_NUMBER_OF_GAMES
                    );
                    send_privmsg(client, &channel, &too_many_games_msg);
                }

                send_privmsg(client, &channel, &result);
            } else if self.is_empty_query(message) {
                println!("Handling empty !games");
                self.update();
                let mut result = String::new();
                let todays_games = self.games.today();
                if todays_games.countries.is_empty() {
                    result.push_str("I've got nothing today. Go outside and enjoy the weather.");
                } else {
                    result.push_str("Check out some places: ");
                    let mut country_names =
                        todays_games.countries.iter().map(|country| &country.name);
                    result.push_str(country_names.next().unwrap());
                    for country_name in country_names {
                        result.push_str(", ");
                        result.push_str(country_name);
                    }
                }
                println!("{}", result);
                client.send_privmsg(&channel, &result).unwrap();
            }
        }
    }
}

impl Default for GamesHandler {
    fn default() -> Self {
        Self::new()
    }
}
