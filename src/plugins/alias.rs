use irc::client::prelude::*;
use regex::Regex;

/// Special plugin that gets used before messages are sent to the other plugins. Lets you rewrite
/// the input to match something else.
#[derive(Debug)]
struct AliasPlugin {
    replacements: Vec<(Regex, String)>,
}

impl AliasPlugin {
    fn new(config: &super::config::Config) -> Self {
        let mut replacements = vec![];
        if let Some(aliases) = &config.alias {
            for (needle, repl) in aliases {
                match Regex::new(needle) {
                    Ok(compiled_needle) => replacements.push((compiled_needle, repl.to_string())),
                    Err(e) => eprintln!("Failed to compile regex {}: {}", needle, e),
                }
            }
        }
        AliasPlugin { replacements }
    }

    fn rewrite(&self, msg: Message) -> Message {
        log::debug!("{:#?}", msg);
        match msg.command {
            Command::PRIVMSG(ref msgtarget, ref messagetext) => {
                for (needle, repl) in &self.replacements {
                    log::debug!("{}", needle.is_match(messagetext));
                    if needle.is_match(messagetext) {
                        let replaced = needle.replace(messagetext, repl);
                        return Message {
                            tags: msg.tags,
                            prefix: msg.prefix,
                            command: Command::PRIVMSG(msgtarget.to_string(), replaced.to_string()),
                        };
                    }
                }

                // No replacement
                msg
            }
            _ => msg,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::config::Config;
    use super::super::config::LeagueRankingConfig;
    use super::super::config::SimpleReplyConfig;
    // Also includes what I `use`d above
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn cl_replacement() {
        let _ = env_logger::builder().is_test(true).try_init();

        let mut alias = HashMap::new();
        alias.insert(
            "^!cl( .*)?$".to_string(),
            "!games --country Champions League$1".to_string(),
        );
        let config = Config {
            league_ranking: LeagueRankingConfig {
                leagues: HashMap::new(),
                competitions: HashMap::new(),
            },
            simple_reply: SimpleReplyConfig {
                replies: HashMap::new(),
            },
            strava: None,
            alias,
        };

        let plug = AliasPlugin::new(&config);

        let mut expected = vec![];
        expected.push(("!cl", "!games --country Champions League"));
        expected.push(("!cl @yday", "!games --country Champions League @yday"));

        for (orig, replaced) in expected {
            let msg = Message {
                tags: None,
                prefix: None,
                command: Command::PRIVMSG("#blab".to_string(), orig.to_string()),
            };
            let replmsg = plug.rewrite(msg);
            match replmsg.command {
                Command::PRIVMSG(_, replmsg) => assert_eq!(replmsg, replaced),
                _ => assert!(false),
            }
        }
    }
}
