use scraper::Html;
use scraper::Selector;
use std::collections::HashMap;

// TODO Should League and Competition have one Trait as interface?

#[derive(Debug)]
pub struct League {
    ranking: Vec<RankingEntry>,
    url: String,
    // TODO: Some caching timer
}

impl League {
    pub fn new(url: String) -> Self {
        Self {
            url,
            ranking: vec![],
        }
    }
}

#[derive(Debug)]
pub struct Competition {
    groups: HashMap<String, Group>,
}

impl Competition {
    pub fn new(group_config: &HashMap<String, String>) -> Self {
        let mut groups = HashMap::new();
        for (group_name, group_url) in group_config {
            groups.insert(group_name.clone(), Group::new(group_url.clone()));
        }
        Self { groups }
    }
}

/// When created, only stores the source url. Will not fetch the rankings till asked to do so.
#[derive(Debug)]
struct Group {
    ranking: Vec<RankingEntry>,
    url: String,
    // TODO: Some caching timer
}

impl Group {
    fn new(url: String) -> Self {
        Self {
            url,
            ranking: vec![],
        }
    }

    fn update(&mut self) {
        if let Ok(mut resp) = reqwest::get(&self.url) {
            if let Ok(content) = resp.text() {
                self.ranking = Group::parse_ranking(&content);
            }
        }
    }

    fn parse_ranking(content: &str) -> Vec<RankingEntry> {
        let doc = Html::parse_document(content);
        let selector =
            Selector::parse("table.leaguetable.sortable.table.detailed-table tbody tr").unwrap();
        let mut ranking = vec![];
        for row in doc.select(&selector) {
            ranking.push(RankingEntry::parse_from_row(row));
        }
        ranking
    }
}

#[derive(Debug)]
struct RankingEntry {
    rank: u8,
    team: String,
    played: u8,
    win: u8,
    draw: u8,
    lose: u8,
    gf: u8,
    ga: u8,
    gd: i8,
    points: u8,
}

impl RankingEntry {
    fn parse_from_row(row: scraper::ElementRef) -> RankingEntry {
        let cell_selector = Selector::parse("td").unwrap();
        let mut cells = row.select(&cell_selector);
        // Need to clean this up still... evidently.
        let rank = cells
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let team = cells.nth(1).unwrap().text().next().unwrap().to_owned();
        let played = cells
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let win = cells
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let draw = cells
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let lose = cells
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let gf = cells
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let ga = cells
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let gd = cells
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let points = cells
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap()
            .parse()
            .unwrap();

        RankingEntry {
            rank,
            team,
            played,
            win,
            draw,
            lose,
            gf,
            ga,
            gd,
            points,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_euro_group_b() {
        let content = include_str!("euro2021-group-b.html");
        println!("{:#?}", Group::parse_ranking(content));
        assert!(false);
    }

    #[test]
    fn parse_belgian_playoff() {
        let content = include_str!("be2021-playoffs.html");
        println!("{:#?}", Group::parse_ranking(content));
        assert!(false);
    }
}
