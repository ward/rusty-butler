use chrono::prelude::*;
use chrono::Duration;
use football::*;
use irc::client::prelude::*;
use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

// To do for good enough parity:
//
// - League shortcuts (!game epl, !game bel, ...)
// - !epl

const MAX_NUMBER_OF_GAMES: i32 = 20;

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

    /// Turn a Game object into a String as I prefer to show them on IRC.
    fn game_to_irc(game: &Game) -> String {
        match &game.status {
            GameStatus::Ended => format!(
                "(FT) {home} {home_score}-{away_score} {away}",
                home = game.home_team,
                home_score = game.home_score.unwrap_or(100),
                away_score = game.away_score.unwrap_or(100),
                away = game.away_team
            ),
            GameStatus::Upcoming => {
                let now = Utc::today();
                if game.start_time.date().ordinal() == now.ordinal() {
                    format!(
                        "({}) {} - {}",
                        game.start_time.format("%H:%M"),
                        game.home_team,
                        game.away_team
                    )
                } else {
                    format!(
                        "({}) {} - {}",
                        game.start_time.format("%d/%m %H:%M"),
                        game.home_team,
                        game.away_team
                    )
                }
            }
            GameStatus::Ongoing(t) => format!(
                "({}) {} {}-{} {}",
                t,
                game.home_team,
                game.home_score.unwrap_or(100),
                game.away_score.unwrap_or(100),
                game.away_team
            ),
            GameStatus::Postponed => format!("(postponed) {} - {}", game.home_team, game.away_team),
            GameStatus::Cancelled => format!("(cancelled) {} - {}", game.home_team, game.away_team),
        }
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
            if let Some(query) = self.get_query(message) {
                println!("Handling !games query: '{}'", query);
                self.update();
                let query = Query::from_message(&query);
                let filtered = self.games.query(&query.just_query_string());
                let filtered = match query.time {
                    QueryTime::Today => filtered.today(),
                    QueryTime::Yesterday => filtered.yesterday(),
                    QueryTime::Tomorrow => filtered.tomorrow(),
                    QueryTime::Finished => filtered.ended(),
                    QueryTime::Live => filtered.live(),
                    QueryTime::Upcoming => filtered.upcoming(),
                };
                let mut result = String::new();
                let mut gamecounter = 0;
                for country in &filtered.countries {
                    result.push_str("<");
                    result.push_str(&country.name);
                    result.push_str("> ");
                    for competition in &country.competitions {
                        result.push_str("[");
                        result.push_str(&competition.name);
                        result.push_str("] ");
                        for game in &competition.games {
                            gamecounter += 1;
                            // Max number of games to show
                            if gamecounter > MAX_NUMBER_OF_GAMES {
                                continue;
                            }
                            result.push_str(&GamesHandler::game_to_irc(game));
                            result.push_str(" ");
                        }
                    }
                }
                // Say something if nothing was found
                if filtered.countries.is_empty() {
                    result = String::from("Your !games query returned no results.");
                }
                println!("{}", result);
                if gamecounter > MAX_NUMBER_OF_GAMES {
                    let too_many_games_msg = format!(
                        "Too many games ({}). Showing first {}.",
                        gamecounter, MAX_NUMBER_OF_GAMES
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

/// Attempt at putting some structure into the queries that people can use. Will parse messages
/// into a Query object. Query is then what is used to decide what games to show.
#[derive(Debug, PartialEq)]
struct Query {
    query: Vec<String>,
    country: Option<String>,
    competition: Option<String>,
    time: QueryTime,
}

impl Query {
    fn from_message(msg: &str) -> Self {
        let msg_parts = msg.split(' ');

        // Ugly parsing
        // Parse out @time ones
        // If encountering a --country or --competition, everything that follows (except another
        // special thing) will be added as country or competition query.
        let mut query = vec![];
        let mut country: Option<String> = None;
        let mut competition: Option<String> = None;
        let mut parsing_country = false;
        let mut parsing_competition = false;
        let mut time = QueryTime::Today;
        for part in msg_parts {
            if part.eq_ignore_ascii_case("--country") {
                parsing_country = true;
                parsing_competition = false;
                country = Some(String::new());
            } else if part.eq_ignore_ascii_case("--competition") {
                parsing_country = true;
                parsing_competition = true;
                competition = Some(String::new());
            } else if part.eq_ignore_ascii_case("@today") {
                time = QueryTime::Today;
            } else if part.eq_ignore_ascii_case("@now") || part.eq_ignore_ascii_case("@live") {
                time = QueryTime::Live;
            } else if part.eq_ignore_ascii_case("@tomorrow") {
                time = QueryTime::Tomorrow;
            } else if part.eq_ignore_ascii_case("@yesterday") || part.eq_ignore_ascii_case("@yday")
            {
                time = QueryTime::Yesterday;
            } else if part.eq_ignore_ascii_case("@finished") || part.eq_ignore_ascii_case("@past") {
                time = QueryTime::Finished;
            } else if part.eq_ignore_ascii_case("@upcoming") || part.eq_ignore_ascii_case("@soon") {
                time = QueryTime::Upcoming;
            } else if parsing_country {
                let mut curr_country = country.expect("Should not be possible to be None");
                if !curr_country.is_empty() {
                    curr_country.push_str(" ");
                }
                curr_country.push_str(part);
                country = Some(curr_country);
            } else if parsing_competition {
                let mut curr_competition = competition.expect("Should not be possible to be None");
                if !curr_competition.is_empty() {
                    curr_competition.push_str(" ");
                }
                curr_competition.push_str(part);
                competition = Some(curr_competition);
            } else if part.is_empty() {
                continue;
            } else {
                query.push(part.to_owned());
            }
        }
        Self {
            query,
            country,
            competition,
            time,
        }
    }

    fn just_query_string(&self) -> String {
        self.query.join(" ")
    }
}

// TODO Part of these should probably be put into a QueryStatus instead of a QueryTime. Querying
// finished might not expect to get everything from yesterday too, currently it would.
#[derive(Debug, Clone, PartialEq)]
enum QueryTime {
    Today,
    Tomorrow,
    Yesterday,
    Finished,
    Live,
    Upcoming,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_query() {
        let desired = Query {
            query: vec![String::from("anderlecht"), String::from("brugge")],
            country: None,
            competition: None,
            time: QueryTime::Today,
        };
        let query = Query::from_message("anderlecht brugge");
        assert_eq!(desired, query);
    }

    #[test]
    fn test_country_query() {
        let desired = Query {
            query: vec![],
            country: Some(String::from("Belgium")),
            competition: None,
            time: QueryTime::Today,
        };
        let query = Query::from_message("--country Belgium");
        assert_eq!(desired, query);
    }

    #[test]
    fn test_query_with_many_spaces() {
        let desired = Query {
            query: vec![String::from("anderlecht"), String::from("brugge")],
            country: None,
            competition: None,
            time: QueryTime::Today,
        };
        let query = Query::from_message("anderlecht    brugge");
        assert_eq!(desired, query);
    }

    #[test]
    fn test_country_with_spaces() {
        let desired = Query {
            query: vec![],
            country: Some(String::from("San Marino")),
            competition: None,
            time: QueryTime::Today,
        };
        let query = Query::from_message("--country San Marino");
        assert_eq!(desired, query);
    }
}
