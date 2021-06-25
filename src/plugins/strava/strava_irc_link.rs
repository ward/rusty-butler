use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::io::Write;

/// Link Strava user IDs to IRC nicks. This struct also provides the convenience functions to
/// access things.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct StravaIrcLink {
    users: HashMap<u64, StravaIrcUser>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct StravaIrcUser {
    #[serde(default)]
    nicks: Vec<String>,
    #[serde(default)]
    ignore: bool,
}

impl StravaIrcLink {
    pub fn new() -> StravaIrcLink {
        StravaIrcLink {
            users: HashMap::new(),
        }
    }
    pub fn from_file_or_new(filename: &str) -> StravaIrcLink {
        StravaIrcLink::from_file(filename).unwrap_or_else(StravaIrcLink::new)
    }

    pub fn from_file(filename: &str) -> Option<StravaIrcLink> {
        // This would be cleaner if we returned a Result<> instead of Option<>
        // Could use ? macro then.
        if let Ok(mut f) = File::open(filename) {
            let mut buffer = String::new();
            if f.read_to_string(&mut buffer).is_ok() {
                match serde_json::from_str(&buffer) {
                    Ok(parsed) => return Some(parsed),
                    Err(e) => {
                        eprintln!("Failed to parse StravaIrcLink: {}", e);
                        return None;
                    }
                }
            }
        }
        None
    }

    pub fn _to_file(&self, filename: &str) {
        // TODO Need to handle failure here better
        match File::create(filename) {
            Ok(mut f) => {
                if let Ok(serialized) = serde_json::to_string(self) {
                    f.write_all(serialized.as_bytes()).unwrap();
                }
            }
            Err(e) => panic!("Failed to save, {}", e),
        }
    }

    pub fn _get_nicks(&self, strava_id: u64) -> Option<Vec<String>> {
        let mut res = vec![];
        for nick in &self.users.get(&strava_id)?.nicks {
            res.push(nick.clone())
        }
        if self.users.get(&strava_id)?.nicks.is_empty() {
            None
        } else {
            Some(res)
        }
    }

    pub fn get_first_nick(&self, strava_id: u64) -> Option<String> {
        let nicks = &self.users.get(&strava_id)?.nicks;
        if self.users.get(&strava_id)?.nicks.is_empty() {
            None
        } else {
            Some(nicks.get(0).unwrap().to_owned())
        }
    }

    pub fn _get_strava_id(&self, nick: &str) -> Option<u64> {
        let nick = nick.to_owned();
        for (strava_id, user) in self.users.iter() {
            if user.nicks.contains(&nick) {
                return Some(strava_id.to_owned());
            }
        }
        None
    }

    // Disabling this clippy warning on a function we do not currently use.
    #[allow(clippy::map_entry)]
    pub fn _insert_connection(&mut self, strava_id: u64, nick: &str) {
        let owned_nick = nick.to_string();
        if self.users.contains_key(&strava_id) {
            let user = self.users.get_mut(&strava_id).unwrap();
            if !user.nicks.contains(&owned_nick) {
                user.nicks.push(owned_nick)
            }
        } else {
            let new_user = StravaIrcUser {
                nicks: vec![owned_nick],
                ignore: false,
            };
            self.users.insert(strava_id, new_user);
        }
    }

    pub fn _remove_nick(&mut self, nick: &str) {
        let nick = nick.to_owned();
        self.users.iter_mut().for_each(|(_strava_id, user)| {
            if user.nicks.contains(&nick) {
                // Update once https://github.com/rust-lang/rust/issues/40062 is stable and
                // done.
                user.nicks.retain(|n| n != &nick);
            }
        });
        // Looping over it again, ugly
        self.users.retain(|_strava_id, user| user.nicks.len() > 1);
    }

    pub fn _remove_strava_id(&mut self, strava_id: u64) {
        self.users.retain(|id, _user| id != &strava_id);
    }

    /// Decide whether a certain user should be considered ignored.
    pub fn is_ignored(&self, strava_id: u64) -> bool {
        match self.users.get(&strava_id) {
            None => true,
            Some(user) => user.ignore,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strava_irc_link() {
        let mut db = StravaIrcLink::new();
        db._insert_connection(123, "ward");
        let result = db._get_nicks(123);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(1, result.len());
        assert_eq!("ward", result.get(0).unwrap());
        db._insert_connection(123, "ward_");
        db._insert_connection(234, "butler");
        db._to_file("testresult.json");
        let result = db._get_nicks(123);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!("ward", result.get(0).unwrap());
        assert_eq!("ward_", result.get(1).unwrap());
        let result = db._get_nicks(234).unwrap();
        assert_eq!("butler", result.get(0).unwrap());
        assert_eq!(1, result.len());
        db._remove_nick("butler");
        assert!(db._get_nicks(234).is_none());
        db._remove_strava_id(123);
        assert!(db._get_strava_id("ward_").is_none());
    }

    #[test]
    fn strava_irc_link_parse() {
        let input = "{ \"users\":
          {
                \"1\": {
                  \"nicks\": [\"ward\",\"ward_\"]
                },
                \"2\": {
                  \"ignore\": true
                }
                }}";
        let parsed: StravaIrcLink = serde_json::from_str(&input).unwrap();
        assert!(parsed.is_ignored(2));
        assert!(!parsed.is_ignored(1));
        assert_eq!(parsed.get_first_nick(1).unwrap(), "ward");
        assert!(parsed.get_first_nick(2).is_none());
    }
}
