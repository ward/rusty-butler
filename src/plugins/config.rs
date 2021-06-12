use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub fantasy: FantasyConfig,
    pub league_ranking: LeagueRankingConfig,
    pub simple_reply: SimpleReplyConfig,
}

impl Config {
    /// Parses the plugins.toml file into configuration for our plugins.
    pub fn new() -> Self {
        let contents =
            std::fs::read_to_string("plugins.toml").expect("No 'plugins.toml' file found.");
        toml::from_str(&contents).expect("Failed to parse 'plugins.toml'.")
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

// TODO How to keep the types for each plugin separate without creating circular dependencies?

#[derive(Deserialize, Debug)]
pub struct SimpleReplyConfig {
    pub replies: HashMap<String, ReplyConfig>,
}
#[derive(Deserialize, Debug)]
pub struct ReplyConfig {
    pub triggers: Vec<String>,
    pub replies: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct FantasyConfig {
    pub uefa: FantasyUefaConfig,
}
#[derive(Deserialize, Debug)]
pub struct FantasyUefaConfig {
    pub league: u64,
    pub predictor_league: u64,
    pub name: String,
    pub buster: String,
    pub cookie: String,
    pub auth_header: String,
    pub auth_header_predictor: String,
}

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
