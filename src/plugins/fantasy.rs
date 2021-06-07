use super::send_privmsg;
use irc::client::prelude::*;

pub struct FantasyHandler {
    /// To get the actual request to work, we first need a request to the regular page. In a
    /// browser this would then load the ranking info via the actual url.
    pre_url: String,
    /// Actual url where the data can be found in json format
    url: String,
    cookie: String,
    auth_header: String,
}
impl FantasyHandler {
    pub fn new(plugin_config: &super::config::Config) -> FantasyHandler {
        let url = format!(
                "https://gaming.uefa.com/en/uefaeuro2020fantasyfootball/services/api//Leagues/{league}/leagueleaderboard?optType=1&phaseId=0&matchdayId=0&vPageChunk=50&vPageNo=1&vPageOneChunk=50&leagueID={league}&fullName={name}&buster={buster}",
                league=plugin_config.fantasy.uefa.league,
                name=plugin_config.fantasy.uefa.name,
                buster=plugin_config.fantasy.uefa.buster);
        let pre_url = format!(
            "https://gaming.uefa.com/en/uefaeuro2020fantasyfootball/league-leaderboard/{}",
            league = plugin_config.fantasy.uefa.league
        );
        FantasyHandler {
            pre_url,
            url,
            cookie: plugin_config.fantasy.uefa.cookie.clone(),
            auth_header: plugin_config.fantasy.uefa.auth_header.clone(),
        }
    }

    fn fetch(&self) -> Vec<UefaFantasyRanking> {
        use reqwest::header::*;
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&self.auth_header).unwrap(),
        );
        headers.insert(COOKIE, HeaderValue::from_str(&self.cookie).unwrap());
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .default_headers(headers)
            .referer(true)
            .build()
            .unwrap();
        if let Ok(resp) = client.get(&self.pre_url).send() {
            let status = resp.status();
            println!("Initial request gave status: {}", status);
            println!("Fetching url: {}", self.url);
            let req = client
                .get(&self.url)
                .header("TE", "Trailers")
                .header("entity", "d3@t4N0te")
                .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:89.0) Gecko/20100101 Firefox/89.0");
            match req.send() {
                Ok(mut resp) => match resp.json() {
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
        } else {
            vec![]
        }
    }

    fn matches(text: &str) -> bool {
        text.eq_ignore_ascii_case("!fantasy")
    }
}
impl super::MutableHandler for FantasyHandler {
    fn handle(&mut self, client: &Client, msg: &Message) {
        if let Command::PRIVMSG(ref channel, ref message) = msg.command {
            if FantasyHandler::matches(message) {
                let ranking = self.fetch();
                if ranking.is_empty() {
                    send_privmsg(client, &channel, "Ranking empty, something went wrong");
                } else {
                    let ranking_txt = ranking
                        .iter()
                        .take(15)
                        .map(|rank_entry| rank_entry.to_string())
                        .collect::<Vec<String>>()
                        .join("; ");
                    send_privmsg(client, &channel, &ranking_txt);
                }
            }
        }
    }
}

impl crate::plugins::help::Help for FantasyHandler {
    fn name(&self) -> String {
        String::from("fantasy")
    }
    fn help(&self) -> Vec<crate::plugins::help::HelpEntry> {
        vec![crate::plugins::help::HelpEntry::new(
            "!fantasy",
            "Return ranking in the EURO 2020-1 fantasy competition",
        )]
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
                team = self.team_name,
            )
        } else {
            write!(
                f,
                "{rank}. {team} {points}pts",
                rank = self.rank,
                team = self.team_name,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stuff() {
        let c = super::super::config::Config::new();
        let f = FantasyHandler::new(&c);
        let ranking = f.fetch();
        println!("{:#?}", ranking);
        assert!(false);
    }
}
