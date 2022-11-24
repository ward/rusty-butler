use chrono::prelude::*;

/// Often I want the Display of a struct to be different from the way I want it to look on IRC.
/// This trait scratches that itch.
pub trait ToIrc {
    fn to_irc(&self) -> String;
    fn to_irc_ordered_by(&self, order: DisplayOrder) -> String;
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

    fn to_irc_ordered_by(&self, _order: DisplayOrder) -> String {
        self.to_irc()
    }
}

impl ToIrc for football::Football {
    fn to_irc(&self) -> String {
        self.to_irc_ordered_by(DisplayOrder::CountryCompetition)
    }

    fn to_irc_ordered_by(&self, order: DisplayOrder) -> String {
        // Create tuples since a Game does not know its competition / country.
        // Once we have tuples, we can do sorting
        let mut all_games: Vec<_> = self
            .countries
            .iter()
            .map(|country| {
                country
                    .competitions
                    .iter()
                    .map(|competition| {
                        competition.games.iter().map(|game| {
                            (game, competition.name.to_string(), country.name.to_string())
                        })
                    })
                    .flatten()
            })
            .flatten()
            .collect();

        // TODO Does this handle the entire to_irc case? If so, remove the code duplication.
        // Just gotta think about the max number of games to show.
        match order {
            DisplayOrder::CountryCompetition => {}
            DisplayOrder::Time => {
                // Relying on stable sort: all games within a country / competition are next to eachother
                // by default. If you sort by time now, then they'd still be together if the same time.
                all_games.sort_by(
                    |(game_a, _competition_a, _country_a), (game_b, _competition_b, _country_b)| {
                        game_a.start_time.partial_cmp(&game_b.start_time).unwrap()
                    },
                );
            }
        }

        let mut result = String::new();
        let mut previous_competition = String::new();
        let mut previous_country = String::new();
        let mut ctr = 0;
        for (game, competition, country) in all_games {
            if previous_country != country {
                result.push('<');
                result.push_str(&country);
                result.push_str("> ");
            }
            if previous_competition != competition {
                result.push('[');
                result.push_str(&competition);
                result.push_str("] ");
            }
            result.push_str(&game.to_irc());
            result.push(' ');
            previous_country = country;
            previous_competition = competition;
            ctr += 1;
            if ctr > super::MAX_NUMBER_OF_GAMES {
                break;
            }
        }
        result
    }
}

#[derive(Debug)]
pub enum DisplayOrder {
    CountryCompetition,
    Time,
}
