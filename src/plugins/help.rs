//! Used to show !help messages for plugins, but as I write this I think that calling it "meta"
//! might have been better. Future work! Could add plugin version and such in that case without
//! making things weird. The plugin "name" already feels a little out of place right now.

use super::send_privmsg;
use irc::client::prelude::*;
use std::collections::HashMap;

/// Handlers of plugins will want to implement this trait in order to be used by this plugin.
pub trait Help {
    fn help(&self) -> Vec<HelpEntry>;
    fn name(&self) -> String;
    // TODO
    // fn version(&self) -> String;
    // (Or a special type for version?)
}

pub struct HelpEntry {
    command: String,
    description: String,
}

impl HelpEntry {
    pub fn new(command: &str, description: &str) -> Self {
        Self {
            command: command.to_owned(),
            description: description.to_owned(),
        }
    }
}

pub struct HelpHandler {
    regex_match: regex::Regex,
    data: HashMap<String, Vec<HelpEntry>>,
}

impl HelpHandler {
    pub fn new() -> Self {
        let mut res = Self {
            regex_match: regex::Regex::new(r"(?i)^!help(?: (\w+)(?: (\d+))?)?").unwrap(),
            data: HashMap::new(),
        };
        // Due to borrow checker, need to explicitly add our own help
        res.data.insert(res.name(), res.help());
        res
    }

    pub fn add_help<T>(&mut self, entry: &T)
    where
        T: Help,
    {
        self.data.insert(entry.name(), entry.help());
    }

    fn plugins(&self) -> Vec<&String> {
        self.data.keys().collect()
    }

    fn commands(&self, plugin_name: &str) -> Vec<&String> {
        self.data
            .get(plugin_name)
            .map_or_else(Vec::new, |help_entries| {
                help_entries
                    .iter()
                    .map(|help_entry| &help_entry.command)
                    .collect()
            })
    }

    fn help_entry(&self, plugin_name: &str, position: usize) -> Option<&HelpEntry> {
        self.data
            .get(plugin_name)
            .and_then(|help_entries| help_entries.get(position))
    }

    fn join_vec(parts: Vec<&String>) -> String {
        let mut result = String::new();
        let mut parts = parts.iter();
        if let Some(part) = parts.next() {
            result.push_str(part);
            for part in parts {
                result.push_str(", ");
                result.push_str(part);
            }
        }
        result
    }
}

impl super::Handler for HelpHandler {
    fn handle(&self, client: &Client, msg: &Message) {
        if let Command::PRIVMSG(ref channel, ref message) = msg.command {
            if let Some(captures) = self.regex_match.captures(message) {
                if let Some(position) = captures.get(2) {
                    // !help plugin_name position
                    if let Ok(position) = position.as_str().parse() {
                        // 2nd capture does not exist without the 1st
                        let plugin_name = captures.get(1).unwrap().as_str();
                        if let Some(help_entry) = self.help_entry(plugin_name, position) {
                            // Found help for request
                            let result = format!(
                                "Command \"{command}\" in {plugin_name}: {description}",
                                command = help_entry.command,
                                plugin_name = plugin_name,
                                description = help_entry.description
                            );
                            send_privmsg(client, channel, &result);
                        } else {
                            // No help entry found (e.g., out of bounds)
                            let result = format!(
                                "No help found at position {} for {}",
                                position, plugin_name
                            );
                            send_privmsg(client, channel, &result);
                        }
                    }
                } else if let Some(plugin_name) = captures.get(1) {
                    // !help plugin_name
                    let plugin_name = plugin_name.as_str();
                    let commands = self.commands(plugin_name);
                    if !commands.is_empty() {
                        let result = format!(
                            "Plugin {plugin_name}: {commands}. Try !help {plugin_name} NUMBER",
                            plugin_name = plugin_name,
                            commands = HelpHandler::join_vec(commands)
                        );
                        send_privmsg(client, channel, &result);
                    } else {
                        let result = format!("No help found for {}", plugin_name);
                        send_privmsg(client, channel, &result);
                    }
                } else {
                    // !help
                    let result = format!("Plugins: {}", HelpHandler::join_vec(self.plugins()));
                    send_privmsg(client, channel, &result);
                }
            }
        }
    }
}

impl std::default::Default for HelpHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Help for HelpHandler {
    fn name(&self) -> String {
        String::from("help")
    }

    fn help(&self) -> Vec<HelpEntry> {
        let mut result = vec![];
        result.push(HelpEntry {
            command: String::from("!help"),
            description: String::from("Shows a list of plugins for which some help exists"),
        });
        result.push(HelpEntry {
            command: String::from("!help PLUGINNAME"),
            description: String::from("Shows a list of commands for the given plugin"),
        });
        result.push(HelpEntry {
            command: String::from("!help PLUGINNAME INDEX"),
            description: String::from(
                "Shows the INDEXth command for the given plugin. Zero-based.",
            ),
        });
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_help() {
        let help_handler = HelpHandler::new();
        let _m1 = help_handler.regex_match.captures("!help").unwrap();
        let _m1 = help_handler.regex_match.captures("!HELP").unwrap();
        let m2 = help_handler
            .regex_match
            .captures("!help a_plugin_name")
            .unwrap();
        m2.get(1).unwrap();
        let m3 = help_handler
            .regex_match
            .captures("!help a_plugin_name 3")
            .unwrap();
        m3.get(1).unwrap();
        m3.get(2).unwrap();
    }
}
