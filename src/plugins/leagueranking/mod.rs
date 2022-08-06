use super::send_privmsg;
use async_trait::async_trait;
use irc::client::prelude::*;
use std::collections::HashMap;

use football::ranking::beebs::CachedLeagues;

#[derive(Debug)]
pub struct LeagueRankingHandler {
    competitions: HashMap<String, CachedLeagues>,
    leagues: HashMap<String, CachedLeagues>,
    aliases: HashMap<String, String>,
}

impl LeagueRankingHandler {
    pub fn new() -> Self {
        // TODO: Move this config creation (i.e., parsing) out of here so it can be done once for
        // all plugins
        let config = super::config::Config::new();

        let mut competitions = HashMap::new();
        let mut leagues = HashMap::new();
        let mut aliases = HashMap::new();

        for (name, league_config) in config.league_ranking.leagues.iter() {
            leagues.insert(name.clone(), CachedLeagues::empty(&league_config.url));
            for alias in &league_config.alias {
                aliases.insert(alias.clone(), name.clone());
            }
        }
        for (name, competition_config) in config.league_ranking.competitions.iter() {
            competitions.insert(name.clone(), CachedLeagues::empty(&competition_config.url));
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

    /// Tries to resolve a potential alias. If no such alias is found, returns the given string.
    /// Will allocate a new String regardless.
    fn resolve_alias(&self, possible_alias: &str) -> String {
        for (alias, target) in &self.aliases {
            if alias.eq_ignore_ascii_case(possible_alias) {
                return target.clone();
            }
        }
        possible_alias.to_string()
    }
}

#[async_trait]
impl super::AsyncMutableHandler for LeagueRankingHandler {
    async fn handle(&mut self, client: &Client, msg: &Message) {
        if let Command::PRIVMSG(ref channel, ref message) = msg.command {
            let mut message_parts = message.split(' ');
            let rank_command = message_parts.next();
            if rank_command.is_none() {
                eprintln!(
                    "Tried to split message_parts and no first part found?? {}",
                    message
                );
                return;
            }
            let rank_command = rank_command.expect("Unreachable due to previous check");
            if !rank_command.eq_ignore_ascii_case("!rank") {
                return;
            }
            if let Some(league_name) = message_parts.next() {
                let league_name = self.resolve_alias(league_name);
                if let Some(league) = self.leagues.get_mut(&league_name) {
                    // This is why we need mut
                    if let Err(e) = league.update().await {
                        eprintln!("Failed to update group ranking: {}", e);
                    }

                    // In a regular league, there is only one
                    if let Some(league) = league.get(0) {
                        let ranking = if let Some(who) = message_parts.next() {
                            if let Ok(who) = who.parse::<usize>() {
                                league.get_ranking_around(if who > 1 { who - 1 } else { who })
                            } else {
                                let pos = league.find_team_position(who);
                                league.get_ranking_around(pos.into())
                            }
                        } else {
                            league.get_ranking_around(1)
                        };

                        let ranking_txt = ranking
                            .iter()
                            .map(|rank_entry| rank_entry.to_string())
                            .collect::<Vec<String>>()
                            .join("; ");
                        send_privmsg(
                            client,
                            &channel,
                            &format!("[{}] {}", league_name, ranking_txt),
                        );
                    }
                } else if let Some(competition) = self.competitions.get_mut(&league_name) {
                    if let Some(group) = message_parts.next() {
                        let group_name = group.to_lowercase();
                        let group_number = group_name_to_number(&group_name);
                        println!("{} - {}", group_name, group_number);

                        // This is why we need mut
                        if let Err(e) = competition.update().await {
                            eprintln!("Failed to update group ranking: {}", e);
                        }

                        if let Some(group) = competition.get(group_number) {
                            let ranking_txt = group
                                .get_ranking()
                                .iter()
                                .map(|rank_entry| rank_entry.to_string())
                                .collect::<Vec<String>>()
                                .join("; ");
                            send_privmsg(
                                client,
                                &channel,
                                &format!("[{}][{}] {}", league_name, group_name, ranking_txt),
                            );
                        } else {
                            send_privmsg(client, &channel, "Not a valid group");
                        }
                    } else {
                        send_privmsg(client, &channel, "You need to give a group too");
                    }
                }
            } else {
                // Perhaps a listing of available leagues? Might be too long.
            }
        }
    }
}

/// Bit of an assumption here that I know will fail (e.g. the A1 etc groups in nations league).
/// Good enough for now...
fn group_name_to_number(letter: &str) -> usize {
    match letter {
        "a" => 0,
        "b" => 1,
        "c" => 2,
        "d" => 3,
        "e" => 4,
        "f" => 5,
        "g" => 6,
        "h" => 7,
        "i" => 8,
        "j" => 9,
        "k" => 10,
        "l" => 11,
        "m" => 12,
        "n" => 13,
        "o" => 14,
        "p" => 15,
        "q" => 16,
        "r" => 17,
        "s" => 18,
        "t" => 19,
        "u" => 20,
        "v" => 21,
        "w" => 22,
        "x" => 23,
        "y" => 24,
        "a1" => 0,
        "a2" => 1,
        "a3" => 2,
        "a4" => 3,
        "b1" => 4,
        "b2" => 5,
        "b3" => 6,
        "b4" => 7,
        "c1" => 8,
        "c2" => 9,
        "c3" => 10,
        "c4" => 11,
        "d1" => 12,
        "d2" => 13,
        _ => 0,
    }
}

impl super::help::Help for LeagueRankingHandler {
    fn name(&self) -> String {
        String::from("league_ranking")
    }
    fn help(&self) -> Vec<super::help::HelpEntry> {
        vec![
            super::help::HelpEntry::new("!rank LEAGUE", "List top 6 ranking for given league"),
            super::help::HelpEntry::new(
                "!rank LEAGUE POSITION",
                "List teams around the position for given league",
            ),
            super::help::HelpEntry::new(
                "!rank COMPETITION GROUP",
                "List ranking for the group in given competition",
            ),
        ]
    }
}

impl Default for LeagueRankingHandler {
    fn default() -> Self {
        Self::new()
    }
}

mod tests {
    #[test]
    fn test_group_name_converter() {
        let letters = vec!["a", "b", "f", "b2", "c4"];
        let group_numbers = vec![0, 1, 5, 5, 11];
        for (letter, number) in letters.into_iter().zip(group_numbers.into_iter()) {
            assert_eq!(super::group_name_to_number(letter), number);
        }
    }
}
