//! Handles contacting the Untappd API and parsing its result.
//!
//! TODO Also fetch the rating for a beer (downside: another API call required)

const USER_AGENT: &str = "rusty-butler-untappd-plugin";

pub fn search(query: &str, client_id: &str, client_secret: &str) -> Vec<BeerResult> {
    let url = format!(
        "https://api.untappd.com/v4/search/beer?client_id={client_id}&client_secret={client_secret}&q={query}",
        client_id=client_id,
        client_secret=client_secret,
        query=sanitise_query(query)
    );
    let client = reqwest::Client::new();
    let req = client
        .get(&url)
        .header(reqwest::header::USER_AGENT, USER_AGENT);
    // TODO Keep track of failure information to pass it to the user
    match req.send() {
        Ok(mut resp) => match resp.json::<UntappdSearch>() {
            Ok(untappd_search) => untappd_search.response.beers.items,
            Err(e) => {
                eprintln!("{}", e);
                vec![]
            }
        },
        Err(e) => {
            eprintln!("{}", e);
            vec![]
        }
    }
}

fn sanitise_query(query: &str) -> String {
    // TODO Actual url encoding?
    query.replace(|ch: char| !ch.is_alphanumeric(), " ")
}

#[derive(Deserialize, Debug)]
struct UntappdSearch {
    response: UntappdSearchResponse,
}

#[derive(Deserialize, Debug)]
struct UntappdSearchResponse {
    beers: UntappdSearchResponseBeers,
}

#[derive(Deserialize, Debug)]
struct UntappdSearchResponseBeers {
    items: Vec<BeerResult>,
}

#[derive(Deserialize, Debug)]
pub struct BeerResult {
    checkin_count: u64,
    beer: Beer,
    brewery: Brewery,
}

#[derive(Deserialize, Debug)]
pub struct Beer {
    #[serde(rename = "bid")]
    id: u64,
    #[serde(rename = "beer_name")]
    name: String,
    #[serde(rename = "beer_abv")]
    abv: f32,
    #[serde(rename = "beer_slug")]
    slug: String,
    #[serde(rename = "beer_ibu")]
    ibu: u32,
    #[serde(rename = "beer_description")]
    description: String,
    #[serde(rename = "beer_style")]
    style: String,
}

#[derive(Deserialize, Debug)]
pub struct Brewery {
    #[serde(rename = "brewery_id")]
    id: u64,
    #[serde(rename = "brewery_name")]
    name: String,
    #[serde(rename = "brewery_slug")]
    slug: String,
    #[serde(rename = "country_name")]
    country: String,
    brewery_type: String,
}

impl BeerResult {
    pub fn to_irc(&self) -> String {
        format!(
            "[UNTAPPD] \"{beer}\" by {brewery} in {country}. {abv}%, {style}. ({checkins} checkins) {url}",
            beer = self.beer.name,
            brewery = self.brewery.name,
            country = self.brewery.country,
            abv = self.beer.abv,
            style = self.beer.style,
            checkins = self.checkin_count,
            url = self.beer.url(),
        )
    }
}

impl Beer {
    fn url(&self) -> String {
        // The slug does not matter, only the id does. So we make it something short
        format!(
            "https://untappd.com/b/eer/{id}",
            // slug = self.slug,
            id = self.id,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_untappd_response() {
        let response = include_str!("untappd.rochefort.json");
        let response: UntappdSearch = serde_json::from_str(&response).unwrap();
        println!("{:#?}", response);
        // TODO: Assert that it matches the data from the json
    }

    #[test]
    fn sanitising_queries() {
        let q = "hello&stuff=ha";
        let s = "hello stuff ha";
        assert_eq!(s, sanitise_query(q));
    }
}
