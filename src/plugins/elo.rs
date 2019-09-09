use chrono::prelude::{DateTime, Utc};
use irc::client::prelude::*;
use reqwest;

pub struct EloHandler {
    elorankings: EloRanking,
    cache_time: u64,
    last_update: u64,
}
impl EloHandler {
    pub fn new() -> EloHandler {
    }
    fn elo_trigger(&self, msg: &str) -> bool {
        let first_four: String = msg.graphemes(true).take(4).collect();
        first_four.eq_ignore_ascii_case("!elo")
    }
    fn handle_elo_ranking(&self, msg: &str) -> Option<String> {
    }
}

impl super::MutableHandler for EloHandler {
    fn handle(&mut self, client: &IrcClient, msg: &Message) {
        if let Command::PRIVMSG(ref channel, ref message) = msg.command {
            // If !elo, show top 10, 15, ...
            self.handle_elo_ranking(message);
            // If !elo N, show Nth club
            // If !elo text, find club
        }
    }
}

struct EloRanking {
    ranking: Vec<EloEntry>,
}
impl EloRanking {
    fn new() -> Option<EloRanking> {
        let now = Utc::now();
        let url = now.format("http://api.clubelo.com/%Y-%m-%d");
        let req = reqwest::get(&url)?;
        let t = req.text()?;
        let s = EloRanking { ranking: vec![] };
        // Body is a csv file (with header)
        for line in t.lines().skip(1) {
            s.ranking.push(EloEntry.parse(line))
        }
    }
    fn top(n: u8) -> Vec<EloEntry> {
        vec![]
    }
    fn nth_place(n: u8) -> EloEntry {
    }
    fn find_club(name: &str) -> Vec<EloEntry> {
        vec![]
    }
}
struct EloEntry {
    rank: u8,
    club: String,
    country: String,
    level: String,
    elo: u16,
    from: String,
    to: String,
}
impl EloEntry {
    fn parse(csvline: &str) -> Option<EloEntry> {
        // Rank,Club,Country,Level,Elo,From,To
    }
}
