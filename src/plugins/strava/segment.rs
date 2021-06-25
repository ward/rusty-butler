use std::fmt;

#[derive(Deserialize, Debug)]
pub struct Segment {
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
    pub fn fetch(id: &str, access_token: &str) -> Result<Segment, reqwest::Error> {
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
        let state = self.state.as_deref().unwrap_or("-");
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
