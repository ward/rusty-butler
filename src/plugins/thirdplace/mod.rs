use super::send_privmsg;
use async_trait::async_trait;
use chrono::prelude::*;
use chrono::Duration;
use irc::client::prelude::*;
use regex::Regex;
use std::fs::File;
use std::io::Write;

pub struct ThirdPlaceHandler {
    /// Just going to reparse every time for now
    content: String,
    cached_at: DateTime<Utc>,
    cache_threshold: Duration,
}

impl ThirdPlaceHandler {
    pub async fn new() -> Self {
        let cache_threshold = Duration::minutes(2);
        let cached_at = Utc::now() - cache_threshold;
        let content = String::from("");
        Self {
            content,
            cached_at,
            cache_threshold,
        }
    }

    /// Fetch wiki page, save it, update cache
    async fn update(&mut self) -> Result<(), reqwest::Error> {
        println!("Running update");
        let url = "https://en.wikipedia.org/wiki/2026_FIFA_World_Cup";
        let client = reqwest::ClientBuilder::new().build()?;
        let req = client
            .get(url)
            .header(
                "Accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            )
            .header(
                reqwest::header::USER_AGENT,
                "rusty-butler-third-place-plugin",
            )
            .send()
            .await?;
        let content = req.text().await?;
        println!("Content received");
        if let Ok(mut f) = File::create("debug.thirdplace.html") {
            let _ = write!(f, "{}", content);
        }
        self.content = content;
        self.cached_at = Utc::now();
        Ok(())
    }

    /// Check cache, call update if past cache
    async fn update_maybe(&mut self) {
        if Utc::now() > self.cached_at + self.cache_threshold {
            match self.update().await {
                Ok(_) => {}
                Err(e) => eprintln!("Error updating ThirdPlaceHandler, {:?}", e),
            }
        }
    }

    /// This is a class method for testing purposes, otherwise need to mock the reqwest. Going to
    /// keep it like this for now.
    fn parse_content(content: &str) -> Option<Vec<String>> {
        let ranking_start_idx = content.find(r#"<h3 id="Ranking_of_third-placed_teams">"#)?;
        let ranking_end_idx = content[ranking_start_idx..]
            .find(r#"Updated to match(es) played"#)?
            + ranking_start_idx;
        // Broken tag at front, full tag, broken tag at the end, [..] notes
        let naive_tag = Regex::new("^[^<>]*>|<[^>]*>|<[^<>]*$|&#91;[^&]*&#93;").unwrap();
        let ranks: Vec<String> = content[ranking_start_idx..ranking_end_idx]
            .split("national football team")
            .skip(1)
            .map(|txt| String::from(naive_tag.replace_all(txt, "")))
            .enumerate()
            .map(|(idx, txt)| {
                let parts: Vec<&str> = txt
                    .split("\n")
                    .filter(|s| !s.eq_ignore_ascii_case(""))
                    .collect();
                if parts.len() < 9 {
                    return String::from("Errored");
                }
                let team = parts[0];
                let win = parts[2];
                let tie = parts[3];
                let loss = parts[4];
                let gf = parts[5];
                let ga = parts[6];
                let gd = parts[7].replace("&#8722;", "–");
                let pts = parts[8];
                // Hardcoding the output format for now
                format!(
                    "{}. {} {}-{}-{} {}-{} ({}) {}pts",
                    idx + 1,
                    team,
                    win,
                    tie,
                    loss,
                    gf,
                    ga,
                    gd,
                    pts
                )
            })
            .collect();
        Some(ranks)
    }
}

#[async_trait]
impl super::AsyncMutableHandler for ThirdPlaceHandler {
    async fn handle(&mut self, client: &Client, msg: &Message) {
        if let Command::PRIVMSG(ref channel, ref message) = msg.command {
            let input = message.trim();
            if input.eq_ignore_ascii_case("!3rd") || input.eq_ignore_ascii_case("!third") {
                self.update_maybe().await;
                if let Some(ranking) = ThirdPlaceHandler::parse_content(&self.content) {
                    send_privmsg(
                        client,
                        &channel,
                        &format!("[3rd] {}", ranking[0..6].join("; ")),
                    );
                    send_privmsg(
                        client,
                        &channel,
                        &format!("[3rd] {}", ranking[6..12].join("; ")),
                    );
                }
            }
        }
    }
}

impl super::help::Help for ThirdPlaceHandler {
    fn name(&self) -> String {
        String::from("3rd")
    }
    fn help(&self) -> Vec<super::help::HelpEntry> {
        vec![super::help::HelpEntry::new(
            "!3rd",
            "List third place ranking",
        )]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_20260619() {
        let content_20260619 = include_str!("example-20260619.html");
        let parsed = ThirdPlaceHandler::parse_content(content_20260619).unwrap();
        assert_eq!(parsed[0], "1. Netherlands 0-1-0 2-2 (0) 1pts");
        assert_eq!(parsed[6], "7. Bosnia and Herzegovina 0-1-1 2-5 (–3) 1pts");
        assert_eq!(parsed[11], "12. Turkey 0-0-1 0-2 (–2) 0pts");
    }

    #[test]
    fn test_parse_20260620() {
        let content_20260620 = include_str!("example-20260620.html");
        let parsed = ThirdPlaceHandler::parse_content(content_20260620).unwrap();
        assert_eq!(parsed[0], "1. Scotland 1-0-1 1-1 (0) 3pts");
        assert_eq!(parsed[6], "7. Czech Republic 0-1-1 2-3 (–1) 1pts");
        assert_eq!(parsed[11], "12. Jordan 0-0-1 1-3 (–2) 0pts");
    }
}
