use chrono::prelude::*;

/// Often I want the Display of a struct to be different from the way I want it to look on IRC.
/// This trait scratches that itch.
pub trait ToIrc {
    fn to_irc(&self) -> String;
    fn to_irc_ordered_by(&self, order: Order) -> String;
}

impl ToIrc for football::Game {
    fn to_irc(&self) -> String {
        match &self.status {
            football::GameStatus::Ended => format!(
                "(FT) {home} {home_score}-{away_score} {away}",
                home = self.home_team,
                home_score = self.home_score.unwrap_or(100),
                away_score = self.away_score.unwrap_or(100),
                away = self.away_team
            ),
            football::GameStatus::Upcoming => {
                let now = Utc::today();
                if self.start_time.date().ordinal() == now.ordinal() {
                    format!(
                        "({}) {} - {}",
                        self.start_time.format("%H:%M"),
                        self.home_team,
                        self.away_team
                    )
                } else {
                    format!(
                        "({}) {} - {}",
                        self.start_time.format("%d/%m %H:%M"),
                        self.home_team,
                        self.away_team
                    )
                }
            }
            football::GameStatus::Ongoing(t) => format!(
                "({}) {} {}-{} {}",
                t,
                self.home_team,
                self.home_score.unwrap_or(100),
                self.away_score.unwrap_or(100),
                self.away_team
            ),
            football::GameStatus::Postponed => {
                format!("(postp.) {} - {}", self.home_team, self.away_team)
            }
            football::GameStatus::Cancelled => {
                format!("(cancld) {} - {}", self.home_team, self.away_team)
            }
        }
    }

    fn to_irc_ordered_by(&self, order: Order) -> String {
        self.to_irc()
    }
}

impl ToIrc for football::Football {
    fn to_irc(&self) -> String {
        let mut result = String::new();
        let mut gamecounter = 0;
        'countryloop: for country in &self.countries {
            result.push('<');
            result.push_str(&country.name);
            result.push_str("> ");
            for competition in &country.competitions {
                result.push('[');
                result.push_str(&competition.name);
                result.push_str("] ");
                for game in &competition.games {
                    gamecounter += 1;
                    // Max number of games to show. Circular dependency doesn't bother the compiler
                    // apparently.
                    if gamecounter > super::MAX_NUMBER_OF_GAMES {
                        break 'countryloop;
                    }
                    result.push_str(&game.to_irc());
                    result.push(' ');
                }
            }
        }
        result
    }

    fn to_irc_ordered_by(&self, order: Order) -> String {
        let mut result = String::new();
        // Create tuples since a Game does not know its competition / country.
        let all_games = self.countries.into_iter().map(|&country| {
            country.competitions.into_iter().map(|&competition| {
                competition
                    .games
                    .into_iter()
                    .map(|&game| (country, competition, game))
            })
        });
        result
    }
}

#[derive(Debug)]
enum Order {
    CountryCompetition,
    Time,
}
