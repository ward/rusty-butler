use chrono::prelude::{DateTime, Utc};
use irc::client::prelude::*;
use reqwest;
use unicode_segmentation::UnicodeSegmentation;

pub struct EloHandler {
    elorankings: EloRanking,
    cache_time: u64,
    last_update: u64,
}
impl EloHandler {
    pub fn new() -> EloHandler {
        EloHandler {
            elorankings: EloRanking::new().unwrap(),
            cache_time: 0,
            last_update: 0,
        }
    }
    fn elo_trigger(&self, msg: &str) -> bool {
        let first_four: String = msg.graphemes(true).take(4).collect();
        first_four.eq_ignore_ascii_case("!elo")
    }
    fn handle_elo_ranking(&self, msg: &str) -> Option<String> {
        None
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

#[derive(Debug)]
struct EloRanking {
    ranking: Vec<EloEntry>,
}
impl EloRanking {
    /// Fetch the current clubelo ranking from http://api.clubelo.com/ and parse it
    fn new() -> Option<EloRanking> {
        let csvtext = EloRanking::fetch()?;
        EloRanking::parse(&csvtext)
    }

    /// Fetch the current clubelo ranking from http://api.clubelo.com/
    fn fetch() -> Option<String> {
        let now = Utc::now();
        let url = now.format("http://api.clubelo.com/%Y-%m-%d").to_string();

        // Ugly checking/unwrapping. How to do nicer?
        let req = reqwest::get(&url);
        if req.is_err() {
            return None;
        }
        let text = req.unwrap().text();
        if text.is_err() {
            return None;
        }
        Some(text.unwrap())
    }

    /// Parse a string in csv format representing current clubelo ranking. The csv format follows
    /// the one api.clubelo.com uses.
    fn parse(csvtext: &str) -> Option<EloRanking> {
        let mut s = EloRanking { ranking: vec![] };
        // Body is a csv file (with header)
        for line in csvtext.lines().skip(1) {
            if let Some(entry) = EloEntry::parse(line) {
                s.ranking.push(entry)
            }
        }
        Some(s)
    }

    fn top(&self, n: usize) -> Vec<EloEntry> {
        // TODO: Can I make this a less heavy operation using references? Would need to tinker with
        // lifetimes though.
        let mut top = vec![];
        for entry in self.ranking.iter().take(n) {
            top.push(entry.clone());
        }
        top
    }

    fn nth_place(&self, n: usize) -> Option<EloEntry> {
        // TODO: Can I make this a less heavy operation using references? Would need to tinker with
        // lifetimes though.
        self.ranking.get(n).and_then(|e| Some(e.clone()))
    }

    fn find_club(name: &str) -> Vec<EloEntry> {
        vec![]
    }
}

#[derive(Debug, Default, Clone)]
struct EloEntry {
    rank: u8,
    club: String,
    country: String,
    level: String,
    elo: f32,
    from: String,
    to: String,
}
impl EloEntry {
    fn parse(csvline: &str) -> Option<EloEntry> {
        // Rank,Club,Country,Level,Elo,From,To
        if csvline.is_empty() {
            return None;
        }
        let mut parts = csvline.split(',');
        let rank = parts.next().unwrap().parse();
        let rank = if rank.is_err() { 0 } else { rank.unwrap() };
        let club = parts.next().unwrap().to_string();
        let country = parts.next().unwrap().to_string();
        let level = parts.next().unwrap().to_string();
        let elo = parts.next().unwrap().parse().unwrap();
        let from = parts.next().unwrap().to_string();
        let to = parts.next().unwrap().to_string();
        Some(EloEntry {
            rank,
            club,
            country,
            level,
            elo,
            from,
            to,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_to_string(path: &str) -> String {
        use std::fs::File;
        use std::io::Read;

        let mut f = File::open(path).unwrap();
        let mut buffer = String::new();
        f.read_to_string(&mut buffer).unwrap();
        buffer
    }

    #[test]
    fn parse_ranking() {
        let text = read_to_string("ranking.20190910.csv");
        let elo = EloRanking::parse(&text);
        println!("{:#?}", elo);
        assert!(false);
    }
}
