//! Handles contacting the Untappd API and parsing its result.
//!
//! TODO Also fetch the rating for a beer (downside: another API call required)

const USER_AGENT: &str = "rusty-butler-untappd-plugin";

pub async fn search(query: &str, client_id: &str, client_secret: &str) -> Vec<BeerResult> {
    let url = "https://api.untappd.com/v4/search/beer";
    let client = reqwest::Client::new();
    let req = client
        .get(url)
        .query(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("q", query), // Encodes it for us
        ])
        .header(reqwest::header::USER_AGENT, USER_AGENT);
    // TODO Keep track of failure information to pass it to the user
    match req.send().await {
        Ok(resp) => match resp.json::<UntappdApiReply>().await {
            Ok(untappd_search) => match untappd_search.response {
                Some(response) => response.beers.items,
                None => {
                    eprintln!("Received error from Untappd API: {:?}", untappd_search);
                    vec![]
                }
            },
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

/// Every API call results in the same root structure
#[derive(Deserialize, Debug, PartialEq)]
struct UntappdApiReply {
    meta: UntappdApiMeta,
    /// A response is only present when the API call is a success
    response: Option<UntappdSearchResponse>,
}

#[derive(Deserialize, Debug, PartialEq)]
struct UntappdApiMeta {
    code: u16,
    error_detail: Option<String>,
    error_type: Option<String>,
    developer_friendly: Option<String>,
}

#[derive(Deserialize, Debug, PartialEq)]
struct UntappdSearchResponse {
    beers: UntappdSearchResponseBeers,
}

#[derive(Deserialize, Debug, PartialEq)]
struct UntappdSearchResponseBeers {
    items: Vec<BeerResult>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct BeerResult {
    checkin_count: u64,
    beer: Beer,
    brewery: Brewery,
}

#[derive(Deserialize, Debug, PartialEq)]
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

#[derive(Deserialize, Debug, PartialEq)]
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
        let parsed_response = UntappdApiReply {
            meta: UntappdApiMeta {
                code: 200,
                error_detail: None,
                error_type: None,
                developer_friendly: None,
            },
            response: Some(
                          UntappdSearchResponse {
                              beers: UntappdSearchResponseBeers {
                                  items: vec![
                                      BeerResult {
                                          checkin_count: 295058,
                                          beer: Beer {
                                              id: 6766,
                                              name: String::from("Trappistes Rochefort 10"),
                                              abv: 11.3,
                                              slug: String::from("abbaye-notredame-de-saintremy-trappistes-rochefort-10"),
                                              ibu: 27,
                                              description: String::from("Dominant impressions of latte coffee with powerful chocolate aromas in the nose. The alcohol esters are enveloped with hints of autumn wood, citrus zest (orange, lemon) and freshly baked biscuits. The initial taste is sweetly sinful. Beer and chocolate trapped into one single glass, a liquid milky draught with a backbone of bitter malt. The alcohol warms the throat and, in the finish, you will pick up traces of cloves, citrus, orange and mocha.\r\nThe heaviest of the Rochefort beers, the 10 is a quadrupel style beer and can be recognized by its blue label."),
                                              style: String::from("Belgian Quadrupel"),
                                          },
                                          brewery: Brewery {
                                              id: 1650,
                                              name: String::from("Abbaye Notre-Dame de Saint-Rémy"),
                                              slug: String::from("abbaye-notre-dame-de-saint-remy"),
                                              country: String::from("Belgium"),
                                              brewery_type: String::from("Regional Brewery"),
                                          },
                                      },
                                  ]
                              }
                          }
            )
        };
        let response = include_str!("untappd.rochefort.json");
        let response: UntappdApiReply = serde_json::from_str(&response).unwrap();
        println!("{:#?}", response);
        assert_eq!(
            response.response.unwrap().beers.items[0],
            parsed_response.response.unwrap().beers.items[0]
        );
        assert_eq!(response.meta, parsed_response.meta);
    }

    #[test]
    fn failed_untappd_api_reply() {
        let parsed_reponse = UntappdApiReply {
            meta: UntappdApiMeta {
                code: 500,
                error_detail: Some(String::from(
                    "The user has not authorized this application or the token is invalid.",
                )),
                error_type: Some(String::from("invalid_auth")),
                developer_friendly: Some(String::from(
                    "The user has not authorized this application or the token is invalid.",
                )),
            },
            response: None,
        };
        let response = include_str!("untappd.api.failure.json");
        let response: UntappdApiReply = serde_json::from_str(&response).unwrap();
        println!("{:#?}", response);
        assert_eq!(parsed_reponse, response);
    }

    #[test]
    fn sanitising_queries() {
        let q = "hello&stuff=ha";
        let s = "hello stuff ha";
        assert_eq!(s, sanitise_query(q));
    }
}
