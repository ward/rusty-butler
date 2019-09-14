use super::formatting;
use irc::client::prelude::*;
use regex::Regex;
use reqwest;
use serde_json;
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::str::FromStr;
use unicode_segmentation::UnicodeSegmentation;

pub struct StravaHandler {
    access_token: Option<String>,
    segment_matcher: Regex,
    irc_links: StravaIrcLink,
}

impl StravaHandler {
    pub fn new(config: &Config) -> StravaHandler {
        let segment_matcher = Regex::new(r"https?://www\.strava\.com/segments/(\d+)").unwrap();
        let irc_links = StravaIrcLink::from_file_or_new("irc_links.json");
        match config.options {
            Some(ref hashmap) => match hashmap.get("strava_access_token") {
                Some(access_token) => StravaHandler {
                    access_token: Some(access_token.clone()),
                    segment_matcher,
                    irc_links,
                },
                None => StravaHandler {
                    access_token: None,
                    segment_matcher,
                    irc_links,
                },
            },
            None => StravaHandler {
                access_token: None,
                segment_matcher,
                irc_links,
            },
        }
    }

    fn handle_segments(&self, msg: &str, access_token: &str) -> Option<String> {
        for captures in self.segment_matcher.captures_iter(msg) {
            println!("{}", captures.get(1).unwrap().as_str());
            let segment = Segment::fetch(captures.get(1).unwrap().as_str(), access_token);
            return match segment {
                Ok(segment) => Some(segment.to_string()),
                Err(e) => {
                    println!("{}", e);
                    None
                }
            };
        }
        None
    }

    fn match_club(msg: &str) -> bool {
        let first_seven: String = msg.graphemes(true).take(7).collect();
        first_seven.eq_ignore_ascii_case("!strava")
    }
    fn handle_club(&self, msg: &str, access_token: &str) -> Vec<String> {
        let mut result = vec![];
        let input: String = msg.graphemes(true).skip(7).collect();
        let input = input.trim();
        println!("Handling club");
        let club_id = "freenode_running";
        let club = Club::fetch(club_id, access_token);
        let leaderboard = ClubLeaderboard::fetch(club_id, access_token);
        match club {
            Ok(club) => result.push(format!(
                "{club} https://www.strava.com/clubs/{club_id}",
                club = club,
                club_id = club_id
            )),
            Err(e) => eprintln!("{}", e),
        }
        match leaderboard {
            Ok(mut leaderboard) => {
                match input.parse() {
                    Ok(sort_by) => leaderboard.sort(sort_by),
                    Err(e) => eprintln!("{}", e),
                }
                leaderboard.override_names(&self.irc_links);
                leaderboard.drop_ignored(&self.irc_links);
                result.push(leaderboard.to_string())
            }
            Err(e) => eprintln!("Error fetching leaderboard: {}", e),
        }
        result
    }
}

impl super::Handler for StravaHandler {
    fn handle(&self, client: &IrcClient, msg: &Message) {
        if let Some(ref access_token) = self.access_token {
            if let Command::PRIVMSG(ref channel, ref message) = msg.command {
                let segment_reply = self.handle_segments(message, &access_token);
                if let Some(segment_id) = segment_reply {
                    client.send_privmsg(&channel, &segment_id).unwrap()
                }
                if StravaHandler::match_club(message) {
                    let club_reply = self.handle_club(message, &access_token);
                    for reply in club_reply {
                        client.send_privmsg(&channel, &reply).unwrap()
                    }
                }
                // TODO Matching a club's URL
            }
        }
    }
}

#[derive(Deserialize, Debug)]
struct Club {
    name: String,
    sport_type: String,
    member_count: u32,
}
impl Club {
    fn fetch(id: &str, access_token: &str) -> Result<Club, reqwest::Error> {
        let url = format!(
            "https://www.strava.com/api/v3/clubs/{}?access_token={}",
            id, access_token
        );
        let mut req = reqwest::get(&url)?;
        println!("{}", req.url());
        req.json()
    }
}
impl fmt::Display for Club {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[STRAVA CLUB] {name}, a {sport_type} club with {member_count} members.",
            name = self.name,
            sport_type = self.sport_type,
            member_count = self.member_count
        )
    }
}

#[derive(Deserialize, Debug)]
struct ClubLeaderboard {
    #[serde(rename = "data")]
    ranking: Vec<ClubLeaderboardAthlete>,
    // The following is never part of the json, but we want a default there anyway
    #[serde(default)]
    sorted_by: ClubLeaderboardSort,
}
impl ClubLeaderboard {
    fn fetch(id: &str, _access_token: &str) -> Result<ClubLeaderboard, reqwest::Error> {
        let url = format!("https://www.strava.com/clubs/{}/leaderboard", id);
        // More involved than the others because we need to change headers
        let client = reqwest::Client::new();
        let mut req = client.get(&url)
            .header("Accept", "text/javascript, application/javascript, application/ecmascript, application/x-ecmascript")
            .header("X-Requested-With", "XmlHttpRequest")
            .send()?;
        println!("{}", req.url());
        req.json()
    }
    fn sort(&mut self, sort_by: ClubLeaderboardSort) {
        if sort_by == self.sorted_by {
            return;
        }
        match sort_by {
            ClubLeaderboardSort::Distance => {
                self.ranking.sort_unstable_by_key(|a| -a.distance as i64)
            }
            ClubLeaderboardSort::Elevation => {
                self.ranking.sort_unstable_by_key(|a| -a.elev_gain as i64)
            }
            ClubLeaderboardSort::Moving => self
                .ranking
                .sort_unstable_by_key(|a| -i64::from(a.moving_time)),
            ClubLeaderboardSort::Pace => self
                .ranking
                .sort_unstable_by_key(|a| -(a.velocity * 1000.0) as i64),
        }
    }
    fn override_names(&mut self, irc_links: &StravaIrcLink) {
        self.ranking.iter_mut().for_each(|athlete| {
            if let Some(nick) = irc_links.get_first_nick(athlete.strava_id) {
                athlete.first_name = nick.to_owned()
            }
        })
    }
    fn drop_ignored(&mut self, irc_links: &StravaIrcLink) {
        self.ranking
            .retain(|athlete| !irc_links.is_ignored(athlete.strava_id));
    }
}
impl fmt::Display for ClubLeaderboard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ranking = self
            .ranking
            .iter()
            .take(10)
            .enumerate()
            .map(|(idx, athlete)| format!("{idx}. {athlete}", idx = idx + 1, athlete = athlete,))
            .fold("".to_string(), |acc, ele| format!("{} {}", acc, ele));
        // Space too many at the start so we use it here instead
        write!(f, "[STRAVA CLUB]{ranking}", ranking = ranking)
    }
}
#[derive(Deserialize, Debug)]
struct ClubLeaderboardAthlete {
    #[serde(rename = "athlete_id")]
    strava_id: u64,
    #[serde(rename = "athlete_firstname")]
    first_name: String,
    distance: f64,
    moving_time: u32,
    elev_gain: f64,
    // Using for sorting (can I use it to get the pace/km number?)
    velocity: f64,
}
impl ClubLeaderboardAthlete {
    /// To prevent triggering people's highlights in IRC, add a zero width space after the first
    /// character. Possible problem: seems to screw up things at times in weechat used through
    /// iTerm2.
    fn prevent_irc_highlight(input: &str) -> String {
        let mut newname = input.to_owned();
        let mut idx = 1;
        while !input.is_char_boundary(idx) {
            idx += 1;
        }
        newname.insert(idx, '\u{200d}');
        newname
    }
}
impl fmt::Display for ClubLeaderboardAthlete {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let distance = (self.distance / 1000.0).floor();
        let pace = (f64::from(self.moving_time) / (self.distance / 1000.0)).round() as u32;
        let elev_gain = self.elev_gain.round() as u32;
        // Moving time format
        let hours = (f64::from(self.moving_time) / 3600.0) as u32;
        let minutes = ((f64::from(self.moving_time) % 3600.0) / 60.0) as u32;
        let moving_time = format!("{}h{:02}", hours, minutes);
        write!(
            f,
            "{format_start}{first_name}{format_end} {distance}k in {moving_time} ({pace}/k ↑{elev_gain}m)",
            // Disabled for now, gives troubles in iterm2 at least.
            first_name = ClubLeaderboardAthlete::prevent_irc_highlight(&self.first_name),
            // first_name = self.first_name,
            distance = distance,
            moving_time = moving_time,
            pace = format_time(pace),
            elev_gain = elev_gain,
            format_start = formatting::IrcFormat::Bold,
            format_end = formatting::IrcFormat::Normal,
        )
    }
}

/// Enum to handle the different inputs by which the leaderboard can be sorted.
/// Ensures in the actual sorting we only deal with some known values. The input string is parsed
/// into one of the enum's values.
#[derive(Debug, Deserialize, PartialEq)]
enum ClubLeaderboardSort {
    Elevation,
    Distance,
    Moving,
    Pace,
}
impl Default for ClubLeaderboardSort {
    fn default() -> Self {
        ClubLeaderboardSort::Distance
    }
}
impl FromStr for ClubLeaderboardSort {
    type Err = ParseClubLeaderboardSortError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "elev" | "elevation" | "vertical" | "climb" | "climbing" => {
                Ok(ClubLeaderboardSort::Elevation)
            }
            "distance" | "dist" | "length" | "len" => Ok(ClubLeaderboardSort::Distance),
            "moving" | "time" | "duration" => Ok(ClubLeaderboardSort::Moving),
            "pace" | "speed" | "velocity" => Ok(ClubLeaderboardSort::Pace),
            _ => Err(ParseClubLeaderboardSortError),
        }
    }
}
#[derive(Debug, Clone)]
struct ParseClubLeaderboardSortError;
impl error::Error for ParseClubLeaderboardSortError {
    fn description(&self) -> &str {
        "Failed to parse sorting parameter."
    }
    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}
impl fmt::Display for ParseClubLeaderboardSortError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to parse sorting parameter.")
    }
}

#[derive(Deserialize, Debug)]
struct Segment {
    name: String,
    activity_type: String,
    distance: f64,
    average_grade: f64,
    effort_count: u32,
    athlete_count: u32,
    city: String,
    // State can be null
    state: Option<String>,
    country: String,
}
impl Segment {
    fn fetch(id: &str, access_token: &str) -> Result<Segment, reqwest::Error> {
        let url = format!(
            "https://www.strava.com/api/v3/segments/{}?access_token={}",
            id, access_token
        );
        let mut req = reqwest::get(&url)?;
        println!("{}", req.url());
        req.json()
    }
}
impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let distance = (self.distance / 100.0).floor() / 10.0;
        let state = match self.state {
            Some(ref s) => s,
            None => "-",
        };
        write!(f,
               "[STRAVA SEGMENT] \"{name}\", {activity_type} of {distance}km @ {grade}%. {effort_count} attempts by {athlete_count} athletes. Located in {city}, {state}, {country}.",
                name = self.name,
                activity_type = self.activity_type,
                distance = distance,
                grade = self.average_grade,
                effort_count = self.effort_count,
                athlete_count = self.athlete_count,
                city = self.city,
                state = state,
                country = self.country)
    }
}

/// Formats a given amount of seconds to the form m:ss or h:mm:ss, depending on the length.
fn format_time(seconds: u32) -> String {
    let hours = (f64::from(seconds) / 3600.0).floor();
    let minutes = (f64::from(seconds % 3600) / 60.0).floor();
    let seconds = seconds % 60;
    if hours == 0.0 {
        return format!("{}:{:02}", minutes, seconds);
    } else {
        return format!("{}:{:02}:{:02}", hours, minutes, seconds);
    }
}

/// Link Strava user IDs to IRC nicks. This struct also provides the convenience functions to
/// access things.
#[derive(Serialize, Deserialize, Debug, Default)]
struct StravaIrcLink {
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
    fn match_club() {
        let input = "!strava";
        assert!(StravaHandler::match_club(input));
        let input = "!stravasdifohoefsbv";
        assert!(StravaHandler::match_club(input));
        let input = "!strava pace";
        assert!(StravaHandler::match_club(input));
    }

    #[test]
    fn match_club_and_unicode() {
        // Input starts with some unicode.
        // In production we got a panic that we were splitting halfway through a character.
        // This crashed the bot
        let input = "🏃🏃";
        assert!(!StravaHandler::match_club(input));
    }

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

    #[test]
    fn irc_highlight_prevention() {
        assert_eq!(
            ClubLeaderboardAthlete::prevent_irc_highlight("ward"),
            "w‍ard"
        );
        assert_eq!(
            ClubLeaderboardAthlete::prevent_irc_highlight("Žilvinas"),
            "Ž‍ilvinas"
        );
        assert_eq!(
            ClubLeaderboardAthlete::prevent_irc_highlight("🇧🇪🇧🇪🇧🇪"),
            "🇧‍🇪🇧🇪🇧🇪"
        );
    }
}
