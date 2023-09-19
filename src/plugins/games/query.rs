//! Attempt at putting some structure into the queries that people can use. Will parse messages
//! into a Query object. Query is then what is used to decide what games to show.

use regex::Regex;

#[derive(Debug, PartialEq)]
pub struct Query {
    pub query: Vec<String>,
    pub country: Option<String>,
    pub competition: Option<String>,
    pub time: QueryTime,
    pub display_order: DisplayOrder,
}

impl Query {
    pub fn just_query_string(&self) -> String {
        self.query.join(" ")
    }
}

// TODO Part of these should probably be put into a QueryStatus instead of a QueryTime. Querying
// finished might not expect to get everything from yesterday too, currently it would.
#[derive(Debug, Clone, PartialEq)]
pub enum QueryTime {
    SlidingWindow,
    Today,
    Tomorrow,
    Yesterday,
    Finished,
    Live,
    Upcoming,
}

#[derive(Debug, PartialEq, Clone)]
pub enum DisplayOrder {
    CountryCompetition,
    Time,
}

/// Sometimes people like to be lazy and not type too much. "epl" will always mean english premier
/// league, "mls" major league soccer, "cl" champions league, ... Shortcuts are defined through
/// this struct.
struct Shortcut {
    regex: Regex,
    country: Option<String>,
    competition: Option<String>,
    replace_by: Vec<String>,
    display_order: Option<DisplayOrder>,
}

/// Parse creates a Query based on a given piece of text. Requires its own struct so we can
/// organise the Regexs in it. A Parser object should in turn be owned by whoever wants to parse
/// queries.
pub struct Parser {
    shortcuts: Vec<Shortcut>,
}

impl Parser {
    pub fn new() -> Self {
        let shortcuts = vec![
            // (?i) is this crate's syntax to turn on case insensitivity
            Shortcut {
                regex: Regex::new(r"^(?i)[eb]pl$").unwrap(),
                country: Some(String::from("England")),
                competition: Some(String::from("Premier League")),
                replace_by: vec![],
                display_order: None,
            },
            Shortcut {
                regex: Regex::new(r"^(?i)(?:la?)?liga$").unwrap(),
                country: Some(String::from("Spain")),
                competition: Some(String::from("LaLiga")),
                replace_by: vec![],
                display_order: None,
            },
            Shortcut {
                regex: Regex::new(r"^(?i)u?cl$").unwrap(),
                country: Some(String::from("Champions League")),
                competition: None,
                replace_by: vec![],
                display_order: None,
            },
            Shortcut {
                regex: Regex::new(r"^(?i)u?el$").unwrap(),
                country: Some(String::from("Europa League")),
                competition: None,
                replace_by: vec![],
                display_order: None,
            },
            Shortcut {
                regex: Regex::new(r"^(?i)ecl$").unwrap(),
                country: Some(String::from("Europa Conference League")),
                competition: None,
                replace_by: vec![],
                display_order: None,
            },
            Shortcut {
                regex: Regex::new(r"^(?i)bundes(?:liga)?$").unwrap(),
                country: Some(String::from("Germany")),
                competition: Some(String::from("Bundesliga")),
                replace_by: vec![],
                display_order: None,
            },
            Shortcut {
                regex: Regex::new(r"^(?i)serie[ -]?a$").unwrap(),
                country: Some(String::from("Italy")),
                competition: Some(String::from("Serie A")),
                replace_by: vec![],
                display_order: None,
            },
            Shortcut {
                regex: Regex::new(r"^(?i)mls$").unwrap(),
                country: Some(String::from("USA")),
                competition: Some(String::from("MLS")),
                replace_by: vec![],
                display_order: None,
            },
            Shortcut {
                regex: Regex::new(r"^(?i)w(?:orld)?c(?:up)?$").unwrap(),
                country: Some(String::from("World Cup 2022")),
                competition: None,
                replace_by: vec![],
                display_order: Some(DisplayOrder::Time),
            },
            Shortcut {
                regex: Regex::new(r"^(?i)w(?:omen'?s?)?-*w(?:orld)?-*c(?:up)?$").unwrap(),
                country: Some(String::from("Women's World Cup")),
                competition: None,
                replace_by: vec![],
                display_order: Some(DisplayOrder::Time),
            },
            Shortcut {
                regex: Regex::new(r"^(?i)psg$").unwrap(),
                country: None,
                competition: None,
                replace_by: vec![
                    String::from("Paris"),
                    String::from("Saint-Germain"),
                ],
                display_order: None,
            },
        ];
        Self { shortcuts }
    }

    pub fn from_message(&self, msg: &str) -> Query {
        let msg_parts = msg.split(' ');

        // Ugly parsing
        // Parse out @time ones
        // If encountering a --country or --competition, everything that follows (except another
        // special thing) will be added as country or competition query.
        let mut query = vec![];
        let mut country: Option<String> = None;
        let mut competition: Option<String> = None;
        let mut parsing_country = false;
        let mut parsing_competition = false;
        let mut time = QueryTime::SlidingWindow;
        let mut display_order = DisplayOrder::CountryCompetition;
        for part in msg_parts {
            if part.eq_ignore_ascii_case("--country") {
                parsing_country = true;
                parsing_competition = false;
                country = Some(String::new());
            } else if part.eq_ignore_ascii_case("--competition") {
                parsing_country = false;
                parsing_competition = true;
                competition = Some(String::new());
            } else if part.eq_ignore_ascii_case("@today") {
                time = QueryTime::Today;
            } else if part.eq_ignore_ascii_case("@now") || part.eq_ignore_ascii_case("@live") {
                time = QueryTime::Live;
            } else if part.eq_ignore_ascii_case("@tomorrow") {
                time = QueryTime::Tomorrow;
            } else if part.eq_ignore_ascii_case("@yesterday") || part.eq_ignore_ascii_case("@yday")
            {
                time = QueryTime::Yesterday;
            } else if part.eq_ignore_ascii_case("@finished")
                || part.eq_ignore_ascii_case("@past")
                || part.eq_ignore_ascii_case("@done")
            {
                time = QueryTime::Finished;
            } else if part.eq_ignore_ascii_case("@upcoming") || part.eq_ignore_ascii_case("@soon") {
                time = QueryTime::Upcoming;
            } else if part.eq_ignore_ascii_case("@bytime") {
                display_order = DisplayOrder::Time;
            } else if parsing_country {
                let mut curr_country = country.expect("Should not be possible to be None");
                if !curr_country.is_empty() {
                    curr_country.push(' ');
                }
                curr_country.push_str(part);
                country = Some(curr_country);
            } else if parsing_competition {
                let mut curr_competition = competition.expect("Should not be possible to be None");
                if !curr_competition.is_empty() {
                    curr_competition.push(' ');
                }
                curr_competition.push_str(part);
                competition = Some(curr_competition);
            } else if part.is_empty() {
                continue;
            } else {
                // Query is not an obvious special entity, but maybe it is one of the shortcuts?
                let mut encountered_shortcut = false;
                for shortcut in &self.shortcuts {
                    if shortcut.regex.is_match(part) {
                        for piece in &shortcut.replace_by {
                            query.push(piece.to_owned());
                        }
                        country = shortcut.country.clone();
                        competition = shortcut.competition.clone();
                        if let Some(order) = shortcut.display_order.clone() {
                            display_order = order;
                        }
                        encountered_shortcut = true;
                        break;
                    }
                }
                if !encountered_shortcut {
                    query.push(part.to_owned());
                }
            }
        }
        Query {
            query,
            country,
            competition,
            time,
            display_order,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_query() {
        let desired = Query {
            query: vec![String::from("anderlecht"), String::from("brugge")],
            country: None,
            competition: None,
            time: QueryTime::SlidingWindow,
            display_order: DisplayOrder::CountryCompetition,
        };
        let parser = Parser::new();
        let query = parser.from_message("anderlecht brugge");
        assert_eq!(desired, query);
    }

    #[test]
    fn test_country_query() {
        let desired = Query {
            query: vec![],
            country: Some(String::from("Belgium")),
            competition: None,
            time: QueryTime::SlidingWindow,
            display_order: DisplayOrder::CountryCompetition,
        };
        let parser = Parser::new();
        let query = parser.from_message("--country Belgium");
        assert_eq!(desired, query);
    }

    #[test]
    fn test_query_with_many_spaces() {
        let desired = Query {
            query: vec![String::from("anderlecht"), String::from("brugge")],
            country: None,
            competition: None,
            time: QueryTime::SlidingWindow,
            display_order: DisplayOrder::CountryCompetition,
        };
        let parser = Parser::new();
        let query = parser.from_message("anderlecht    brugge");
        assert_eq!(desired, query);
    }

    #[test]
    fn test_country_with_spaces() {
        let desired = Query {
            query: vec![],
            country: Some(String::from("San Marino")),
            competition: None,
            time: QueryTime::SlidingWindow,
            display_order: DisplayOrder::CountryCompetition,
        };
        let parser = Parser::new();
        let query = parser.from_message("--country San Marino");
        assert_eq!(desired, query);
    }

    #[test]
    fn test_competition() {
        let desired = Query {
            query: vec![],
            country: Some(String::from("Europa League")),
            competition: Some(String::from("Group K")),
            time: QueryTime::SlidingWindow,
            display_order: DisplayOrder::CountryCompetition,
        };
        let parser = Parser::new();
        let query = parser.from_message("--country Europa League --competition Group K");
        assert_eq!(desired, query);
    }

    #[test]
    fn test_competition_before_country() {
        let desired = Query {
            query: vec![],
            country: Some(String::from("Europa League")),
            competition: Some(String::from("Group K")),
            time: QueryTime::SlidingWindow,
            display_order: DisplayOrder::CountryCompetition,
        };
        let parser = Parser::new();
        let query = parser.from_message("--competition Group K --country Europa League");
        assert_eq!(desired, query);
    }

    #[test]
    fn test_competition_no_country() {
        let desired = Query {
            query: vec![],
            country: None,
            competition: Some(String::from("Group K")),
            time: QueryTime::SlidingWindow,
            display_order: DisplayOrder::CountryCompetition,
        };
        let parser = Parser::new();
        let query = parser.from_message("--competition Group K");
        assert_eq!(desired, query);
    }

    #[test]
    fn test_shortcut_and_competition() {
        let desired = Query {
            query: vec![],
            country: Some(String::from("Europa League")),
            competition: Some(String::from("Group K")),
            time: QueryTime::SlidingWindow,
            display_order: DisplayOrder::CountryCompetition,
        };
        let parser = Parser::new();
        let query = parser.from_message("el --competition Group K");
        assert_eq!(desired, query);
    }

    #[test]
    fn test_epl_shortcut() {
        let desired = Query {
            query: vec![],
            country: Some(String::from("England")),
            competition: Some(String::from("Premier League")),
            time: QueryTime::SlidingWindow,
            display_order: DisplayOrder::CountryCompetition,
        };
        let parser = Parser::new();
        let query = parser.from_message("epl");
        assert_eq!(desired, query);
    }

    #[test]
    fn test_epl_caps_shortcut() {
        let desired = Query {
            query: vec![],
            country: Some(String::from("England")),
            competition: Some(String::from("Premier League")),
            time: QueryTime::SlidingWindow,
            display_order: DisplayOrder::CountryCompetition,
        };
        let parser = Parser::new();
        let query = parser.from_message("EPL");
        assert_eq!(desired, query);
    }

    #[test]
    fn test_epl_shortcut_with_time() {
        let desired = Query {
            query: vec![],
            country: Some(String::from("England")),
            competition: Some(String::from("Premier League")),
            time: QueryTime::Yesterday,
            display_order: DisplayOrder::CountryCompetition,
        };
        let parser = Parser::new();
        let query = parser.from_message("@yday epl");
        assert_eq!(desired, query);
        let query = parser.from_message("epl @yday");
        assert_eq!(desired, query);
    }

    #[test]
    fn test_world_cup_shortcut() {
        let desired = Query {
            query: vec![],
            country: Some(String::from("World Cup 2022")),
            competition: None,
            time: QueryTime::SlidingWindow,
            display_order: DisplayOrder::Time,
        };
        let parser = Parser::new();
        let query = parser.from_message("wc");
        assert_eq!(desired, query);
    }

    #[test]
    fn test_women_world_cup_shortcut() {
        let desired = Query {
            query: vec![],
            country: Some(String::from("Women's World Cup")),
            competition: None,
            time: QueryTime::SlidingWindow,
            display_order: DisplayOrder::Time,
        };
        let parser = Parser::new();
        let query = parser.from_message("wwc");
        assert_eq!(desired, query);
    }

    #[test]
    fn test_shortcut_with_dashes() {
        let desired = Query {
            query: vec![],
            country: Some(String::from("Women's World Cup")),
            competition: None,
            time: QueryTime::SlidingWindow,
            display_order: DisplayOrder::Time,
        };
        let parser = Parser::new();
        let query = parser.from_message("women-world-cup");
        assert_eq!(desired, query);
    }
}
