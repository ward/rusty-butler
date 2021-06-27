use super::send_privmsg;
use async_trait::async_trait;
use irc::client::prelude::*;

const CACHE_DURATION: std::time::Duration = std::time::Duration::from_secs(3 * 60);

pub struct FantasyHandler {
    /// To get the actual request to work, we first need a request to the regular page. In a
    /// browser this would then load the ranking info via the actual url.
    fantasy_pre_url: String,
    /// Actual url where the data can be found in json format
    fantasy_url: String,
    predictor_pre_url: String,
    predictor_url: String,
    cookie: String,
    auth_header: String,
    auth_header_predictor: String,
    fantasy_ranking: Vec<UefaFantasyRanking>,
    predictor_ranking: Vec<UefaPredictorRanking>,
    last_fantasy_update: std::time::Instant,
    last_predictor_update: std::time::Instant,
}

impl FantasyHandler {
    pub fn new(plugin_config: &super::config::Config) -> FantasyHandler {
        let fantasy_url = format!(
                "https://gaming.uefa.com/en/uefaeuro2020fantasyfootball/services/api//Leagues/{league}/leagueleaderboard?optType=1&phaseId=0&matchdayId=0&vPageChunk=50&vPageNo=1&vPageOneChunk=50&leagueID={league}&fullName={name}&buster={buster}",
                league=plugin_config.fantasy.uefa.league,
                name=plugin_config.fantasy.uefa.name,
                buster=plugin_config.fantasy.uefa.buster);
        let fantasy_pre_url = format!(
            "https://gaming.uefa.com/en/uefaeuro2020fantasyfootball/league-leaderboard/{}",
            league = plugin_config.fantasy.uefa.league
        );
        let predictor_pre_url = format!(
            "https://gaming.uefa.com/en/uefaeuro2020matchpredictor/leagues/private/leaderboard/{}",
            league = plugin_config.fantasy.uefa.predictor_league
        );
        let predictor_url = format!(
"https://gaming.uefa.com/en/uefaeuro2020matchpredictor/api/v1/competition/3/season/current/predictor/leagues/{}/leaderboard",
league = plugin_config.fantasy.uefa.predictor_league
        );
        FantasyHandler {
            fantasy_pre_url,
            fantasy_url,
            predictor_url,
            predictor_pre_url,
            cookie: plugin_config.fantasy.uefa.cookie.clone(),
            auth_header: plugin_config.fantasy.uefa.auth_header.clone(),
            auth_header_predictor: plugin_config.fantasy.uefa.auth_header_predictor.clone(),
            fantasy_ranking: vec![],
            predictor_ranking: vec![],
            last_fantasy_update: std::time::Instant::now()
                .checked_sub(CACHE_DURATION)
                .expect("[fantasy] Failed to initialise last_updated value"),
            last_predictor_update: std::time::Instant::now()
                .checked_sub(CACHE_DURATION)
                .expect("[fantasy] Failed to initialise last_updated_predictor value"),
        }
    }

    /// Updates if last update is older than CACHE_DURATION
    pub async fn fantasy_update(&mut self) {
        if self.needs_fantasy_update() {
            let new_ranking = self.fetch().await;
            if new_ranking.is_empty() {
                eprintln!("Received empty ranking");
            } else {
                self.last_fantasy_update = std::time::Instant::now();
                self.fantasy_ranking = new_ranking;
            }
        }
    }

    pub async fn predictor_update(&mut self) {
        if self.needs_predictor_update() {
            let new_ranking = self.fetch_predictor().await;
            if new_ranking.is_empty() {
                eprintln!("Received empty predictor ranking");
            } else {
                self.last_predictor_update = std::time::Instant::now();
                self.predictor_ranking = new_ranking;
            }
        }
    }

    /// True if last update is older than CACHE_DURATION
    fn needs_fantasy_update(&self) -> bool {
        let now = std::time::Instant::now();
        let passed_time = now.duration_since(self.last_fantasy_update);
        passed_time > CACHE_DURATION
    }

    /// True if last update is older than CACHE_DURATION
    fn needs_predictor_update(&self) -> bool {
        let now = std::time::Instant::now();
        let passed_time = now.duration_since(self.last_predictor_update);
        passed_time > CACHE_DURATION
    }

    /// Fetches the EURO 2020 fantasy ranking from UEFA website. On any failure, an empty Vec is
    /// returned. Ideally this will become a Result<> in the future.
    async fn fetch(&self) -> Vec<UefaFantasyRanking> {
        use reqwest::header::*;
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&self.auth_header)
                .expect("Could not turn fantasy auth_header into a HeaderValue"),
        );
        headers.insert(
            COOKIE,
            HeaderValue::from_str(&self.cookie)
                .expect("Could not turn fantasy cookie into a HeaderValue"),
        );
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .default_headers(headers)
            .referer(true)
            .build();
        match client {
            Ok(client) => match client.get(&self.fantasy_pre_url).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    println!("Initial request gave status: {}", status);
                    println!("Fetching url: {}", self.fantasy_url);
                    let req = client
                .get(&self.fantasy_url)
                .header("TE", "Trailers")
                .header("entity", "d3@t4N0te")
                .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:89.0) Gecko/20100101 Firefox/89.0");
                    match req.send().await {
                        Ok(resp) => match resp.json().await {
                            Ok(UefaResponse::Response { data }) => data.value.rest,
                            Ok(UefaResponse::Failure { status, title }) => {
                                eprintln!("Received error from fantasy url. {} {}", status, title);
                                vec![]
                            }
                            Err(e) => {
                                eprintln!("Failed to parse fantasy url response. {}", e);
                                vec![]
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to get fantasy url. {}", e);
                            vec![]
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to send(). {}", e);
                    vec![]
                }
            },
            Err(e) => {
                eprintln!("Could not create reqwest client. {}", e);
                vec![]
            }
        }
    }

    async fn fetch_predictor(&self) -> Vec<UefaPredictorRanking> {
        use reqwest::header::*;
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&self.auth_header_predictor)
                .expect("Could not turn fantasy auth_header into a HeaderValue"),
        );
        headers.insert(
            COOKIE,
            HeaderValue::from_str(&self.cookie)
                .expect("Could not turn fantasy cookie into a HeaderValue"),
        );
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .default_headers(headers)
            .referer(true)
            .build();
        match client {
            Ok(client) => match client.get(&self.predictor_pre_url).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    println!("Initial request gave status: {}", status);
                    println!("Fetching url: {}", self.predictor_url);
                    let req = client
                        .get(&self.predictor_url)
                        // .header("TE", "Trailers")
                        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:89.0) Gecko/20100101 Firefox/89.0");
                    match req.send().await {
                        Ok(resp) => {
                            let status = resp.status();
                            if status.is_client_error() || status.is_server_error() {
                                eprintln!("Failed to fetch predictor url: {}", status);
                                return vec![];
                            }
                            match resp.json::<UefaPredictorResponse>().await {
                                Ok(uefa_predictor_response) => uefa_predictor_response.data.items,
                                Err(e) => {
                                    eprintln!("Failed to parse predictor url response. {}", e);
                                    vec![]
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to get predictor url. {}", e);
                            vec![]
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to send(). {}", e);
                    vec![]
                }
            },
            Err(e) => {
                eprintln!("Could not create reqwest client. {}", e);
                vec![]
            }
        }
    }

    fn fantasy_matches(text: &str) -> bool {
        let text = text.trim();
        text.eq_ignore_ascii_case("!fantasy")
            || text.eq_ignore_ascii_case("!ufpl")
            || text.eq_ignore_ascii_case("!uefafantasy")
            || text.eq_ignore_ascii_case("!fantasyuefa")
            || text.eq_ignore_ascii_case("!efpl")
    }

    fn predictor_matches(text: &str) -> bool {
        let text = text.trim();
        text.eq_ignore_ascii_case("!predict")
            || text.eq_ignore_ascii_case("!predictor")
            || text.eq_ignore_ascii_case("!uefapredict")
            || text.eq_ignore_ascii_case("!uefapredictor")
    }
}

#[async_trait]
impl super::AsyncMutableHandler for FantasyHandler {
    async fn handle(&mut self, client: &Client, msg: &Message) {
        if let Command::PRIVMSG(ref channel, ref message) = msg.command {
            if FantasyHandler::fantasy_matches(message) {
                self.fantasy_update().await;
                let ranking_txt = self
                    .fantasy_ranking
                    .iter()
                    .take(15)
                    .map(|rank_entry| rank_entry.to_string())
                    .collect::<Vec<String>>()
                    .join("; ");
                send_privmsg(client, &channel, &format!("[EURO FANTASY] {}", ranking_txt));
            } else if FantasyHandler::predictor_matches(message) {
                self.predictor_update().await;
                let ranking_txt = self
                    .predictor_ranking
                    .iter()
                    .take(15)
                    .map(|rank_entry| rank_entry.to_string())
                    .collect::<Vec<String>>()
                    .join("; ");
                send_privmsg(
                    client,
                    &channel,
                    &format!("[EURO PREDICTOR] {}", ranking_txt),
                );
            }
        }
    }
}

impl crate::plugins::help::Help for FantasyHandler {
    fn name(&self) -> String {
        String::from("fantasy")
    }
    fn help(&self) -> Vec<crate::plugins::help::HelpEntry> {
        vec![
            crate::plugins::help::HelpEntry::new(
                "!fantasy",
                "Return ranking in the EURO 2020-1 fantasy competition",
            ),
            crate::plugins::help::HelpEntry::new(
                "!predict",
                "Returning ranking in the EURO 2020-1 match predictor competition",
            ),
        ]
    }
}

#[derive(Deserialize, Debug)]
struct UefaFantasyRanking {
    #[serde(rename = "teamName")]
    team_name: String,
    #[serde(rename = "fullName")]
    full_name: String,
    #[serde(rename = "overallPoints")]
    overall_points: String,
    #[serde(rename = "rankNo")]
    rank: String,
    // There is also an actual rank field, but just says null atm
}

impl std::fmt::Display for UefaFantasyRanking {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.overall_points.is_empty() {
            write!(
                f,
                "{rank}. {team} no pts",
                rank = self.rank,
                team = prevent_irc_highlight(&self.team_name),
            )
        } else {
            write!(
                f,
                "{rank}. {team} {points}pts",
                rank = self.rank,
                team = prevent_irc_highlight(&self.team_name),
                points = self.overall_points,
            )
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum UefaResponse {
    Response { data: UefaResponseData },
    Failure { status: u16, title: String },
}
#[derive(Deserialize, Debug)]
struct UefaResponseData {
    value: UefaResponseValue,
}
#[derive(Deserialize, Debug)]
struct UefaResponseValue {
    rest: Vec<UefaFantasyRanking>,
}

#[derive(Deserialize, Debug)]
struct UefaPredictorResponse {
    data: UefaPredictorResponseData,
}
#[derive(Deserialize, Debug)]
struct UefaPredictorResponseData {
    items: Vec<UefaPredictorRanking>,
}
#[derive(Debug)]
struct UefaPredictorRanking {
    position: u32,
    points: u32,
    current_md_points: u32,
    username: String,
}
// From https://github.com/serde-rs/serde/issues/1098
fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + serde::Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}
// From https://stackoverflow.com/questions/41042767/is-it-possible-to-flatten-sub-object-fields-while-parsing-with-serde-json
impl<'de> serde::Deserialize<'de> for UefaPredictorRanking {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Outer {
            position: u32,
            #[serde(deserialize_with = "deserialize_null_default")]
            points: u32,
            #[serde(deserialize_with = "deserialize_null_default")]
            current_md_points: u32,
            gh_user_data: Inner,
        }

        #[derive(Deserialize)]
        struct Inner {
            username: String,
        }

        let helper = Outer::deserialize(deserializer)?;
        Ok(UefaPredictorRanking {
            position: helper.position,
            points: helper.points,
            current_md_points: helper.current_md_points,
            username: helper.gh_user_data.username,
        })
    }
}

impl std::fmt::Display for UefaPredictorRanking {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{rank}. {team} {points}pts (md: {md_points})",
            rank = self.position,
            team = prevent_irc_highlight(&self.username),
            points = self.points,
            md_points = self.current_md_points,
        )
    }
}
fn prevent_irc_highlight(input: &str) -> String {
    let mut newname = input.to_owned();
    let mut idx = 1;
    while !input.is_char_boundary(idx) {
        idx += 1;
    }
    newname.insert(idx, '\u{200d}');
    newname
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn stuff() {
//         let c = super::super::config::Config::new();
//         let f = FantasyHandler::new(&c);
//         let ranking = f.fetch();
//         println!("{:#?}", ranking);
//         assert!(false);
//     }
//
//     #[test]
//     fn predictor_stuff() {
//         let c = super::super::config::Config::new();
//         let f = FantasyHandler::new(&c);
//         let ranking = f.fetch_predictor();
//         println!("{:#?}", ranking);
//         assert!(false);
//     }
// }
