mod soccerway;

use std::collections::HashMap;

use irc::client::prelude::*;

#[derive(Debug)]
pub struct LeagueRankingHandler {
    competitions: HashMap<String, soccerway::Competition>,
    leagues: HashMap<String, soccerway::League>,
    aliases: HashMap<String, String>,
}

impl LeagueRankingHandler {
    pub fn new() -> Self {
        // TODO: Move this config creation (parsing) to be done once for all plugins
        let config = super::config::Config::new();

        let mut competitions = HashMap::new();
        let mut leagues = HashMap::new();
        let mut aliases = HashMap::new();

        for (name, league_config) in config.league_ranking.leagues.iter() {
            leagues.insert(
                name.clone(),
                soccerway::League::new(league_config.url.clone()),
            );
            for alias in &league_config.alias {
                aliases.insert(alias.clone(), name.clone());
            }
        }
        for (name, competition_config) in config.league_ranking.competitions.iter() {
            competitions.insert(
                name.clone(),
                soccerway::Competition::new(&competition_config.groups),
            );
            for alias in &competition_config.alias {
                aliases.insert(alias.clone(), name.clone());
            }
        }

        Self {
            competitions,
            leagues,
            aliases,
        }
    }
}

impl Default for LeagueRankingHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let handler = LeagueRankingHandler::new();
        println!("{:#?}", handler);
        assert!(false);
    }
}
