use irc::client::prelude::*;
use regex::Regex;
use reqwest;
use std::str::FromStr;
use std::error;
use std::fmt;

pub fn handler(client: &IrcClient, msg: &Message, config: &Config) {
    // TODO Lots of needless checks every time. How to avoid?
    let access_token = get_access_token(config);
    if access_token.is_none() {
        return ()
    }
    let access_token = access_token.unwrap();
    if let Command::PRIVMSG(ref channel, ref message) = msg.command {
        let segment_reply = handle_segments(message, &access_token);
        match segment_reply {
            Some(segment_id) => client.send_privmsg(&channel, &segment_id).unwrap(),
            _ => (),
        }
        let activity_reply = handle_activities(message, &access_token);
        match activity_reply {
            Some(reply) => client.send_privmsg(&channel, &reply).unwrap(),
            _ => (),
        }
        if match_club(message) {
            let club_reply = handle_club(message, &access_token);
            for reply in club_reply {
                client.send_privmsg(&channel, &reply).unwrap()
            }
        }
        // TODO Matching a club's URL
    }
}

pub fn get_access_token(config: &Config) -> Option<&String> {
    let options = &config.options;
    match options {
        Some(hm) => hm.get("strava_access_token"),
        None => None
    }
}

fn match_club(msg: &str) -> bool {
    msg.len() >= 7 &&
        msg[..7].eq_ignore_ascii_case("!strava")
}
fn handle_club(msg: &str, access_token: &str) -> Vec<String> {
    let mut result = vec![];
    let input = msg[7..].trim();
    println!("Handling club");
    let club_id = "freenode_running";
    let club = Club::fetch(club_id, access_token);
    let leaderboard = ClubLeaderboard::fetch(club_id, access_token);
    match club {
        Ok(club) => result.push(format!("{club} https://www.strava.com/clubs/{club_id}",
                                        club = club.to_string(),
                                        club_id = club_id)),
        Err(e) => eprintln!("{}", e),
    }
    match leaderboard {
        Ok(mut leaderboard) => {
            match input.parse() {
                Ok(sort_by) => leaderboard.sort(sort_by),
                Err(e) => eprintln!("{}", e),
            }
            result.push(leaderboard.to_string())
        }
        Err(e) => eprintln!("{}", e),
    }
    result
}
#[derive(Deserialize,Debug)]
struct Club {
    name: String,
    sport_type: String,
    member_count: u32,
}
impl Club {
    fn fetch(id: &str, access_token: &str) -> Result<Club, reqwest::Error> {
        let url = format!("https://www.strava.com/api/v3/clubs/{}?access_token={}", id, access_token);
        let mut req = reqwest::get(&url)?;
        println!("{}", req.url());
        req.json()
    }
}
impl ToString for Club {
    fn to_string(&self) -> String {
        format!("[STRAVA CLUB] {name}, a {sport_type} club with {member_count} members.",
                name = self.name,
                sport_type = self.sport_type,
                member_count = self.member_count)
    }
}

#[derive(Deserialize,Debug)]
struct ClubLeaderboard {
    #[serde(rename = "data")]
    ranking: Vec<ClubLeaderboardAthlete>,
    // The following is never part of the json, but we want a default there anyway
    #[serde(default)]
    sorted_by: ClubLeaderboardSort,
}
impl ClubLeaderboard {
    fn fetch(id: &str, _access_token: &str) -> Result<ClubLeaderboard, reqwest::Error> {
        use reqwest::header::qitem;
        let url = format!("https://www.strava.com/clubs/{}/leaderboard", id);
        // More involved than the others because we need to change headers
        let client = reqwest::Client::new();
        let mut headers = reqwest::header::Headers::new();
        let accept = reqwest::header::Accept(vec![
                                             qitem(reqwest::mime::TEXT_JAVASCRIPT),
                                             qitem(reqwest::mime::APPLICATION_JAVASCRIPT),
                                             qitem("application/ecmascript".parse().unwrap()),
                                             qitem("application/x-ecmascript".parse().unwrap()),
        ]);
        headers.set(accept);
        headers.set_raw("X-Requested-With", "XmlHttpRequest");
        let mut req = client.get(&url)
            .headers(headers)
            .send()?;
        println!("{}", req.url());
        //println!("{}", req.text().unwrap());
        req.json()
    }
    fn sort(&mut self, sort_by: ClubLeaderboardSort) {
        if sort_by == self.sorted_by { return }
        match sort_by {
            ClubLeaderboardSort::Distance => {
                self.ranking.sort_unstable_by_key(|a| -a.distance as i64)
            },
            ClubLeaderboardSort::Elevation => {
                self.ranking.sort_unstable_by_key(|a| -a.elev_gain as i64)
            },
            ClubLeaderboardSort::Moving => {
                self.ranking.sort_unstable_by_key(|a| -(a.moving_time as i64))
            },
            ClubLeaderboardSort::Pace => {
                self.ranking.sort_unstable_by_key(|a| -(a.velocity * 1000.0) as i64)
            },
        }
    }
}
impl ToString for ClubLeaderboard {
    fn to_string(&self) -> String {
        let ranking = self.ranking.iter()
            .take(10)
            .enumerate()
            .map(|(idx, athlete)| format!("{idx}. {athlete}", idx = idx+1, athlete = athlete.to_string()))
            .fold("".to_string(), |acc, ele| format!("{} {}", acc, ele));
        // Space too many at the start so we use it here instead
        format!("[STRAVA CLUB]{ranking}", ranking = ranking)
    }
}
#[derive(Deserialize,Debug)]
struct ClubLeaderboardAthlete {
    #[serde(rename = "athlete_firstname")]
    first_name: String,
    distance: f64,
    moving_time: u32,
    elev_gain: f64,
    // Using for sorting (can I use it to get the pace/km number?)
    velocity: f64,
}
impl ToString for ClubLeaderboardAthlete {
    fn to_string(&self) -> String {
        let distance = (self.distance / 1000.0).floor();
        let pace = (self.moving_time as f64 / (self.distance / 1000.0)).round() as u32;
        let elev_gain = self.elev_gain.round() as u32;
        // Moving time format
        let hours = (self.moving_time as f64 / 3600.0) as u32;
        let minutes = ((self.moving_time as f64 % 3600.0) / 60.0) as u32;
        let moving_time = format!("{}h{:02}", hours, minutes);
        format!("{first_name} {distance}k in {moving_time} ({pace}/k ↑{elev_gain}m)",
                first_name = self.first_name,
                distance = distance,
                moving_time = moving_time,
                pace = format_time(pace),
                elev_gain = elev_gain)
    }
}
#[derive(Debug,Deserialize,PartialEq)]
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
            "elev" | "elevation" | "vertical" | "climb" | "climbing" => Ok(ClubLeaderboardSort::Elevation),
            "distance" | "dist" | "length" | "len" => Ok(ClubLeaderboardSort::Distance),
            "moving" | "time" | "duration" => Ok(ClubLeaderboardSort::Moving),
            "pace" | "speed" | "velocity" => Ok(ClubLeaderboardSort::Pace),
            _ => Err(ParseClubLeaderboardSortError),
        }
    }
}
#[derive(Debug,Clone)]
struct ParseClubLeaderboardSortError;
impl error::Error for ParseClubLeaderboardSortError {
    fn description(&self) -> &str {
        "Failed to parse sorting parameter."
    }
    fn cause(&self) -> Option<&error::Error> {
        None
    }
}
impl fmt::Display for ParseClubLeaderboardSortError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to parse sorting parameter.")
    }
}

fn handle_activities(msg: &str, access_token: &str) -> Option<String> {
    // TODO The regex crate suggests using lazy_static crate to avoid creating
    // this regex every time
    let activity_regex = Regex::new(r"https?://www\.strava\.com/activities/(\d+)").unwrap();
    for captures in activity_regex.captures_iter(msg) {
        let activity = Activity::fetch(captures.get(1).unwrap().as_str(), access_token);
        return match activity {
            Ok(activity) => Some(activity.to_string()),
            Err(e) => {
                eprintln!("{}", e);
                None
            }
        }
    }
    None
}

#[derive(Deserialize, Debug)]
struct Activity {
    name: String,
    #[serde(rename = "type")]
    sport: String,
    distance: f64,
    total_elevation_gain: f64,
    moving_time: u32,
}
impl Activity {
    fn fetch(id: &str, access_token: &str) -> Result<Activity, reqwest::Error> {
        let url = format!("https://www.strava.com/api/v3/activities/{}?access_token={}", id, access_token);
        let mut req = reqwest::get(&url)?;
        println!("{}", req.url());
        req.json()
    }
}
impl ToString for Activity {
    fn to_string(&self) -> String {
        let distance = (self.distance / 100.0).floor() / 10.0;
        let pace = (self.moving_time as f64 / (self.distance / 1000.0)).round() as u32;
        format!("[STRAVA {sport}] \"{name}\", {distance} km (↑{elev}m) in {time} ({pace}/km)",
                sport = self.sport.to_uppercase(),
                name = self.name,
                distance = distance,
                elev = self.total_elevation_gain.round(),
                time = format_time(self.moving_time),
                pace = format_time(pace))
    }
}

fn handle_segments(msg: &str, access_token: &str) -> Option<String> {
    // TODO The regex crate suggests using lazy_static crate to avoid creating
    // this regex every time
    let segment_regex = Regex::new(r"https?://www\.strava\.com/segments/(\d+)").unwrap();
    for captures in segment_regex.captures_iter(msg) {
        println!("{}", captures.get(1).unwrap().as_str());
        let segment = Segment::fetch(captures.get(1).unwrap().as_str(), access_token);
        return match segment {
            Ok(segment) => Some(segment.to_string()),
            Err(e) => {
                println!("{}", e);
                None
            }
        }
    }
    None
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
        let url = format!("https://www.strava.com/api/v3/segments/{}?access_token={}", id, access_token);
        let mut req = reqwest::get(&url)?;
        println!("{}", req.url());
        req.json()
    }
}
impl ToString for Segment {
    fn to_string(&self) -> String {
        let distance = (self.distance / 100.0).floor() / 10.0;
        let state = match self.state {
            Some(ref s) => s,
            None => "-",
        };
        format!("[STRAVA SEGMENT] \"{name}\", {activity_type} of {distance}km @ {grade}%. {effort_count} attempts by {athlete_count} athletes. Located in {city}, {state}, {country}.",
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

fn format_time(seconds: u32) -> String {
    let hours = (seconds as f64 / 3600.0).floor();
    let minutes = ((seconds % 3600) as f64 / 60.0).floor();
    let seconds = seconds % 60;
    if hours == 0.0 {
        return format!("{}:{:02}", minutes, seconds);
    } else {
        return format!("{}:{:02}:{:02}", hours, minutes, seconds);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stuff() {
        // TODO Do not commit!
        let s = handle_club("!strava pace", "");
        for reply in s {
            println!("{}", reply);
        }

    }
}
