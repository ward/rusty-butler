use async_trait::async_trait;
use chrono::prelude::{DateTime, Utc};
use irc::client::prelude::*;
use unicode_segmentation::UnicodeSegmentation;

pub struct EloHandler {
    ranking: Vec<EloEntry>,
    cache_time: chrono::Duration,
    last_update: DateTime<Utc>,
}

impl EloHandler {
    pub fn new() -> EloHandler {
        EloHandler {
            ranking: vec![],
            cache_time: chrono::Duration::hours(12),
            last_update: Utc::now() - chrono::Duration::hours(13),
        }
    }

    fn is_cache_stale(&self) -> bool {
        let now = Utc::now();
        now - self.last_update > self.cache_time
    }

    /// Tries to update the rankings, leaves old one untouched if it fails.
    async fn update_rankings(&mut self) {
        if let Some(csvtext) = Self::fetch().await {
            let ranking = Self::parse(&csvtext);
            if ranking.len() > 0 {
                self.ranking = ranking;
                self.last_update = Utc::now();
            }
        }
    }

    fn is_elo_trigger(msg: &str) -> bool {
        "!elo".eq_ignore_ascii_case(msg.trim())
    }

    fn handle_elo_ranking(&self, msg: &str) -> Option<String> {
        if EloHandler::is_elo_trigger(msg) {
            let ranks: Vec<String> = self
                .ranking
                .iter()
                .take(15)
                .map(|entry| entry.to_string())
                .collect();
            Some(ranks.join("; "))
        } else {
            None
        }
    }

    fn handle_elo_nth(&self, msg: &str) -> Option<String> {
        let exclamation = msg.graphemes(true).next();
        if exclamation == Some("!") {
            // Note: unicode_words removes '!'
            let mut words = msg.unicode_words();
            if let Some(trigger) = words.next() {
                if let Some(nth) = words.next() {
                    let cruft = words.next();
                    if cruft.is_none() && trigger.eq_ignore_ascii_case("elo") {
                        if let Ok(nth) = nth.parse::<usize>() {
                            return self
                                .ranking
                                .get(nth - 1)
                                .map(|entry: &EloEntry| entry.to_string());
                        }
                    }
                }
            }
        }
        None
    }
    fn handle_search(&self, msg: &str) -> Option<String> {
        let exclamation = msg.graphemes(true).next();
        if exclamation == Some("!") {
            let mut words = msg.unicode_words();
            if let Some(trigger) = words.next() {
                if trigger.eq_ignore_ascii_case("elo") {
                    let query: Vec<&str> = words.collect();
                    let query = query.join(" ");
                    let results = self.find_club(&query);
                    if results.is_empty() {
                        return Some("No club found for your query".to_owned());
                    } else {
                        let results: Vec<String> =
                            results.iter().map(|entry| entry.to_string()).collect();
                        return Some(results.join("; "));
                    }
                }
            }
        }
        None
    }

    /// Fetch the current clubelo ranking from http://api.clubelo.com/
    async fn fetch() -> Option<String> {
        let now = Utc::now();
        let url = now.format("http://api.clubelo.com/%Y-%m-%d").to_string();

        let req = reqwest::get(&url).await.ok()?;
        req.text().await.ok()
    }

    /// Parse a string in csv format representing current clubelo ranking. The csv format follows
    /// the one api.clubelo.com uses.
    fn parse(csvtext: &str) -> Vec<EloEntry> {
        let mut ranking = vec![];
        // Body is a csv file (with header)
        for (ctr, line) in csvtext.lines().skip(1).enumerate() {
            if let Some(entry) = EloEntry::parse(line, ctr) {
                ranking.push(entry)
            }
        }
        ranking
    }

    /// Search our ranking for teams matching the search term. Note that clubelo's team names
    /// are... a bit weird.
    fn find_club(&self, name: &str) -> Vec<EloEntry> {
        // TODO: Have a list of name conversions since "Man City" is specific and does not match
        // "Manchester".
        // TODO: Lowercasing every club in the list cannot be efficient...
        let name = name.to_lowercase();
        self.ranking
            .iter()
            .filter(|entry| entry.club.to_lowercase().contains(&name))
            .cloned()
            .collect()
    }
}

impl std::default::Default for EloHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl super::AsyncMutableHandler for EloHandler {
    async fn handle(&mut self, client: &Client, msg: &Message) {
        if let Command::PRIVMSG(ref channel, ref message) = msg.command {
            // Only update when command is used
            if message.starts_with("!elo") && self.is_cache_stale() {
                self.update_rankings().await;
            }

            let reply = self
                .handle_elo_ranking(message)
                .or_else(|| self.handle_elo_nth(message))
                .or_else(|| self.handle_search(message));
            if let Some(reply) = reply {
                client
                    .send_privmsg(&channel, &format!("[ELO] {}", reply))
                    .unwrap()
            }
        }
    }
}

impl super::help::Help for EloHandler {
    fn name(&self) -> String {
        String::from("elo")
    }

    fn help(&self) -> Vec<super::help::HelpEntry> {
        vec![
            super::help::HelpEntry::new("!elo", "Show the top few teams ranked by clubelo."),
            super::help::HelpEntry::new(
                "!elo QUERY",
                "Search for teams matching QUERY and list their clubelo.",
            ),
            super::help::HelpEntry::new(
                "!elo POSITION",
                "Search for the team in POSITIONth place and list their clubelo.",
            ),
        ]
    }
}

#[derive(Debug, Default, Clone)]
struct EloEntry {
    rank: usize,
    club: String,
    country: String,
    level: String,
    elo: f32,
    from: String,
    to: String,
}
impl EloEntry {
    fn parse(csvline: &str, rank: usize) -> Option<EloEntry> {
        // Rank,Club,Country,Level,Elo,From,To
        if csvline.is_empty() {
            return None;
        }
        let mut parts = csvline.split(',');
        let _rank = parts.next(); //.unwrap().parse();
                                  // let rank = if rank.is_err() { 0 } else { rank.unwrap() };
        let rank = rank + 1;
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
impl std::fmt::Display for EloEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{rank}. {club} {pts:.0}pts",
            rank = self.rank,
            club = self.club,
            pts = self.elo,
        )
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
        let text = include_str!("clubelo.ranking.20190910.csv");
        let elo = EloHandler::parse(&text);
        assert!(elo.len() > 0);
        let p612 = elo.get(612).expect("There should be a 612th place");
        assert_eq!(p612.club, "La Fiorita");
        assert!(elo.get(613).is_none());
        let p187 = elo.get(187).expect("There should be a 187th place");
        assert_eq!(p187.club, "Anderlecht");
    }

    #[test]
    fn get_top_10() {
        let text = include_str!("clubelo.ranking.20190910.csv");
        let elo = EloHandler::parse(&text);
        assert!(elo.len() > 0);
        let top10: Vec<&EloEntry> = elo.iter().take(10).collect();
        assert_eq!(top10.len(), 10);
        assert_eq!(top10[0].club, "Liverpool");
        assert_eq!(top10[9].club, "Ajax");
    }

    #[test]
    fn find_exact_club() {
        let text = include_str!("clubelo.ranking.20190910.csv");
        let elorank = EloHandler::parse(&text);
        assert!(elorank.len() > 0);
        let mut elo = EloHandler::new();
        elo.ranking = elorank;
        let results = elo.find_club("Anderlecht");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].club, "Anderlecht");
        assert_eq!(results[0].rank, 187);
    }

    #[test]
    fn find_different_case_club() {
        let text = include_str!("clubelo.ranking.20190910.csv");
        let elorank = EloHandler::parse(&text);
        assert!(elorank.len() > 0);
        let mut elo = EloHandler::new();
        elo.ranking = elorank;
        let results = elo.find_club("anderlecht");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].club, "Anderlecht");
    }

    #[test]
    fn find_many_clubs() {
        let text = include_str!("clubelo.ranking.20190910.csv");
        let elorank = EloHandler::parse(&text);
        assert!(elorank.len() > 0);
        let mut elo = EloHandler::new();
        elo.ranking = elorank;
        let results = elo.find_club("man");
        assert_eq!(results.len(), 6);
        assert_eq!(results[0].club, "Man City");
        assert_eq!(results[1].club, "Man United");
        assert_eq!(results[1].rank, 12);
        assert_eq!(results[5].club, "Neman Grodno");
    }
}
