mod soccerway;

use super::send_privmsg;

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
        // TODO: Move this config creation (i.e., parsing) out of here so it can be done once for
        // all plugins
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

impl super::MutableHandler for LeagueRankingHandler {
    fn handle(&mut self, client: &Client, msg: &Message) {
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
                    league.update(); // This is why we need mut

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
                } else if let Some(competition) = self.competitions.get_mut(&league_name) {
                    if let Some(group) = message_parts.next() {
                        let group_name = group.to_lowercase();
                        if let Some(group) = competition.get_group_mut(&group_name) {
                            group.update(); // This is why we need mut

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
