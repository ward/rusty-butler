use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub league_ranking: LeagueRankingConfig,
}

impl Config {
    /// Parses the plugins.toml file into configuration for our plugins.
    pub fn new() -> Self {
        let contents = std::fs::read_to_string("plugins.toml").unwrap();
        toml::from_str(&contents).unwrap()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

// TODO How to keep the types for each plugin separate without creating circular dependencies?

#[derive(Deserialize, Debug)]
pub struct LeagueRankingConfig {
    pub leagues: HashMap<String, LeagueConfig>,
    pub competitions: HashMap<String, CompetitionConfig>,
}

#[derive(Deserialize, Debug)]
pub struct LeagueConfig {
    pub alias: Vec<String>,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct CompetitionConfig {
    pub alias: Vec<String>,
    pub groups: HashMap<String, String>,
}
