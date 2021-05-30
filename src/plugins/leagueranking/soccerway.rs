use scraper::Html;
use scraper::Selector;
use std::collections::HashMap;

const CACHE_DURATION: std::time::Duration = std::time::Duration::from_secs(10 * 60);

// TODO Should League and Competition (Group) have one Trait as interface? Lots of code repetition atm

#[derive(Debug)]
pub struct League {
    ranking: Vec<RankingEntry>,
    url: String,
    last_updated: std::time::Instant,
}

impl League {
    pub fn new(url: String) -> Self {
        Self {
            url,
            ranking: vec![],
            last_updated: std::time::Instant::now()
                .checked_sub(CACHE_DURATION)
                .unwrap(),
        }
    }

    /// Updates if last update is older than CACHE_DURATION
    pub fn update(&mut self) {
        if self.needs_update() {
            println!("Fetching data from {}", self.url);
            self.last_updated = std::time::Instant::now();
            if let Ok(mut resp) = reqwest::get(&self.url) {
                if let Ok(content) = resp.text() {
                    self.ranking = parse_ranking(&content);
                }
            }
        }
    }

    /// True if last update is older than CACHE_DURATION
    fn needs_update(&self) -> bool {
        let now = std::time::Instant::now();
        let passed_time = now.duration_since(self.last_updated);
        passed_time > CACHE_DURATION
    }

    /// Gets up to 6 teams around a certain position
    pub fn get_ranking_around(&self, idx: usize) -> &[RankingEntry] {
        let length = self.ranking.len();
        let range = if length <= 6 {
            0..length
        } else if idx <= 3 {
            0..6
        } else if idx >= length - 2 {
            (length - 6)..length
        } else {
            (idx - 3)..(idx + 3)
        };
        &self.ranking[range]
    }

    /// Returns 0 indexed position.
    /// Defaults to 0 if nothing found.
    /// Yes that makes little sense but we're only using this in one place.
    pub fn find_team_position(&self, needle: &str) -> u8 {
        let needle = needle.to_lowercase();
        for rank in &self.ranking {
            let team_name = rank.team.to_lowercase();
            if team_name.matches(&needle).count() > 0 {
                return rank.rank - 1;
            }
        }
        0
    }
}

/// Extracted so both League and Group can use it. Should this be some trait?
/// Returns empty Vec if an error is encountered
///
/// TODO Proper Result returning?
fn parse_ranking(content: &str) -> Vec<RankingEntry> {
    let doc = Html::parse_document(content);
    match Selector::parse("table.leaguetable.sortable.table.detailed-table tbody tr") {
        Ok(selector) => {
            let mut ranking = vec![];
            for row in doc.select(&selector) {
                match RankingEntry::parse_from_row(row) {
                    Some(entry) => ranking.push(entry),
                    None => {
                        eprintln!("Failed to parse a ranking row, returning empty vector");
                        return vec![];
                    }
                }
            }
            ranking
        }
        Err(e) => {
            eprintln!("Failed to parse content. {:?}", e);
            vec![]
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

    pub fn get_group_mut(&mut self, group_id: &str) -> Option<&mut Group> {
        self.groups.get_mut(group_id)
    }
}

/// When created, only stores the source url. Will not fetch the rankings till asked to do so.
#[derive(Debug)]
pub struct Group {
    ranking: Vec<RankingEntry>,
    url: String,
    last_updated: std::time::Instant,
}

impl Group {
    fn new(url: String) -> Self {
        Self {
            url,
            ranking: vec![],
            last_updated: std::time::Instant::now()
                .checked_sub(CACHE_DURATION)
                .unwrap(),
        }
    }

    /// Updates if last update is older than CACHE_DURATION
    pub fn update(&mut self) {
        if self.needs_update() {
            println!("Fetching data from {}", self.url);
            self.last_updated = std::time::Instant::now();
            if let Ok(mut resp) = reqwest::get(&self.url) {
                if let Ok(content) = resp.text() {
                    self.ranking = parse_ranking(&content);
                }
            }
        }
    }

    /// True if last update is older than CACHE_DURATION
    fn needs_update(&self) -> bool {
        let now = std::time::Instant::now();
        let passed_time = now.duration_since(self.last_updated);
        passed_time > CACHE_DURATION
    }

    pub fn get_ranking(&self) -> &Vec<RankingEntry> {
        &self.ranking
    }
}

#[derive(Debug)]
pub struct RankingEntry {
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
    fn parse_from_row(row: scraper::ElementRef) -> Option<RankingEntry> {
        // TODO Make this return a Result<> instead of Option<>

        let cell_selector = Selector::parse("td").ok()?;
        let mut cells = row.select(&cell_selector);
        // Need to clean this up, too much repetition
        let rank = cells.next()?.text().next()?.parse().ok()?;
        let team = cells.nth(1)?.text().next()?.to_owned();
        let played = cells.next()?.text().next()?.parse().ok()?;
        let win = cells.next()?.text().next()?.parse().ok()?;
        let draw = cells.next()?.text().next()?.parse().ok()?;
        let lose = cells.next()?.text().next()?.parse().ok()?;
        let gf = cells.next()?.text().next()?.parse().ok()?;
        let ga = cells.next()?.text().next()?.parse().ok()?;
        let gd = cells.next()?.text().next()?.parse().ok()?;
        let points = cells.next()?.text().next()?.parse().ok()?;

        Some(RankingEntry {
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
        })
    }
}

impl std::fmt::Display for RankingEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{rank}. {team} {points}pts {win}-{draw}-{lose} {gf}-{ga}",
            rank = self.rank,
            team = self.team,
            points = self.points,
            win = self.win,
            draw = self.draw,
            lose = self.lose,
            gf = self.gf,
            ga = self.ga
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_euro_group_b() {
        let content = include_str!("euro2021-group-b.html");
        let ranking = parse_ranking(content);
        let belgium = RankingEntry {
            rank: 1,
            team: String::from("Belgium"),
            played: 0,
            win: 0,
            draw: 0,
            lose: 0,
            gf: 0,
            ga: 0,
            gd: 0,
            points: 0,
        };
        assert_eq!(ranking[0].rank, belgium.rank);
        assert_eq!(ranking[0].team, belgium.team);
        assert_eq!(ranking[0].gf, belgium.gf);
    }

    #[test]
    fn parse_belgian_playoff() {
        let content = include_str!("be2021-playoffs.html");
        let ranking = parse_ranking(content);
        let anderlecht = RankingEntry {
            rank: 4,
            team: String::from("Anderlecht"),
            played: 6,
            win: 0,
            draw: 4,
            lose: 2,
            gf: 9,
            ga: 11,
            gd: -2,
            points: 33,
        };
        assert_eq!(ranking[3].rank, anderlecht.rank);
        assert_eq!(ranking[3].team, anderlecht.team);
        assert_eq!(ranking[3].gf, anderlecht.gf);
    }
}
