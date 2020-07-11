use chrono::prelude::*;
use football::*;
use irc::client::prelude::*;
use regex::Regex;

// TODO Need to invalidate cache after x minutes

pub struct GamesHandler {
    games: Football,
    query_matcher: Regex,
}

// TODO Use clap's get_matches_from to parse queries with options?

impl GamesHandler {
    pub fn new() -> Self {
        let games = get_all_games().expect("Failed to get games");
        let query_matcher = Regex::new(r"!games? +(.+)$").unwrap();
        Self {
            games,
            query_matcher,
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
}

impl super::MutableHandler for GamesHandler {
    fn handle(&mut self, client: &Client, msg: &Message) {
        if let Command::PRIVMSG(ref channel, ref message) = msg.command {
            if let Some(query) = self.get_query(message) {
                let filtered = self.games.query(&query);
                let mut result = String::new();
                for country in &filtered.countries {
                    result.push_str("<");
                    result.push_str(&country.name);
                    result.push_str("> ");
                    for competition in &country.competitions {
                        result.push_str("[");
                        result.push_str(&competition.name);
                        result.push_str("] ");
                        for game in &competition.games {
                            result.push_str(&GamesHandler::game_to_irc(game));
                            result.push_str(" ");
                        }
                    }
                }
                println!("{}", result);
                client.send_privmsg(&channel, &result).unwrap();
            }
        }
    }
}
