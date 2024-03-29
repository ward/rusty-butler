use irc::client::prelude::*;
use regex::Regex;
use std::fmt;
use std::str::FromStr;
use unicode_segmentation::UnicodeSegmentation;

pub struct CalcHandler {
    ctx: rink_core::Context,
    shortcuts: Vec<CalcShortcut>,
    feet_to_cm_matcher: Regex,
    cm_to_feet_matcher: Regex,
    grade_matcher: Regex,
}
impl CalcHandler {
    pub fn new() -> CalcHandler {
        let mut ctx = rink_core::simple_context().expect("Could not create calculator core?");
        ctx.short_output = true;
        let shortcuts = vec![
            CalcShortcut {
                regex: Regex::new(r"!km +(\d.*)$").unwrap(),
                target_unit: "kilometre".to_owned(),
                default_unit: "miles".to_owned(),
            },
            CalcShortcut {
                regex: Regex::new(r"!mi(?:le)? +(\d.*)$").unwrap(),
                target_unit: "miles".to_owned(),
                default_unit: "kilometer".to_owned(),
            },
            CalcShortcut {
                regex: Regex::new(r"^!c +(-?\d.*)$").unwrap(),
                target_unit: "celsius".to_owned(),
                default_unit: "fahrenheit".to_owned(),
            },
            CalcShortcut {
                regex: Regex::new(r"^!f +(-?\d.*)$").unwrap(),
                target_unit: "fahrenheit".to_owned(),
                default_unit: "celsius".to_owned(),
            },
            CalcShortcut {
                regex: Regex::new(r"^!kg +(-?\d.*)$").unwrap(),
                target_unit: "kilogram".to_owned(),
                default_unit: "lbs".to_owned(),
            },
            CalcShortcut {
                regex: Regex::new(r"^!(?:lbs?|pound) +(-?\d.*)$").unwrap(),
                target_unit: "lbs".to_owned(),
                default_unit: "kilogram".to_owned(),
            },
        ];

        let feet_to_cm_matcher = Regex::new(r"^!cm +(\d+)\D+([0-9.]+)").unwrap();
        let cm_to_feet_matcher =
            Regex::new(r"^!(?:f(?:ee|oo)?t|in(?:ch|ches)?) +([0-9.]+) *(?:cm)?$").unwrap();

        // !grade <distance> <elevation>
        let grade_matcher = Regex::new(r"^(?i)!grade +(?P<distance>[0-9.]+) *(?P<distanceunit>[a-z]+)? +(?P<elevation>[0-9.]+) *(?P<elevationunit>[a-z]+)?$").unwrap();

        CalcHandler {
            ctx,
            shortcuts,
            feet_to_cm_matcher,
            cm_to_feet_matcher,
            grade_matcher,
        }
    }
    fn match_calc(msg: &str) -> bool {
        let first_six: String = msg.graphemes(true).take(6).collect();
        first_six.eq_ignore_ascii_case("!calc ")
    }

    fn get_calc_input(msg: &str) -> String {
        let input: String = msg.graphemes(true).skip(6).collect();
        input.trim().to_owned()
    }

    fn eval(&mut self, line: &str) -> Result<String, String> {
        rink_core::one_line(&mut self.ctx, line)
    }

    /// Checks incoming message to see whether it uses a calculation shortcut. If so, return
    /// Some(stringtoevaluate). Otherwise None
    fn handle_shortcut(&self, msg: &str) -> Option<String> {
        for shortcut in &self.shortcuts {
            // TODO Should this be part of a CalcShortcut method?
            if let Some(captures) = shortcut.regex.captures(msg) {
                if let Some(input) = captures.get(1) {
                    let input = input.as_str().trim();
                    if input
                        .chars()
                        .last()
                        .expect("Cannot be empty because of regex match")
                        .is_ascii_digit()
                    {
                        return Some(format!(
                            "{} {} to {}",
                            input, shortcut.default_unit, shortcut.target_unit
                        ));
                    } else {
                        return Some(format!("{} to {}", input, shortcut.target_unit));
                    }
                }
            }
        }
        None
    }

    /// Checks incoming message for a !pace calculation.
    /// The input is some sort of time representation.
    /// We provide a conversion of t/km to t/mile and vice versa.
    fn handle_pace(&self, msg: &str) -> Option<String> {
        let first_six: String = msg.graphemes(true).take(6).collect();
        if !first_six.eq_ignore_ascii_case("!pace ") {
            return None;
        }
        let input: String = msg.graphemes(true).skip(6).collect();
        let input = input.trim();
        // TODO: Log failure to parse
        if let Ok(pace) = input.parse::<Pace>() {
            Some(format!(
                "{orig}/km = {miles}/mile || {orig}/mile = {km}/km",
                orig = pace,
                miles = pace.to_per_miles(),
                km = pace.to_per_kilometre()
            ))
        } else {
            None
        }
    }

    fn handle_grade(&mut self, msg: &str) -> Option<String> {
        if let Some(captures) = self.grade_matcher.captures(msg) {
            // Parsing input
            let distance: Result<f64, _> = captures.name("distance").unwrap().as_str().parse();
            let elevation: Result<f64, _> = captures.name("elevation").unwrap().as_str().parse();

            if distance.is_err() || elevation.is_err() {
                return None;
            }

            let distance = distance.unwrap();
            let elevation = elevation.unwrap();

            let distance_unit = if let Some(unit) = captures.name("distanceunit") {
                let unit = unit.as_str();
                match unit {
                    "k" => "km",
                    "mi" => "miles",
                    "ft" => "feet",
                    "m" => "meter",
                    _ => unit,
                }
            } else {
                "km"
            };
            let elevation_unit = if let Some(unit) = captures.name("elevationunit") {
                let unit = unit.as_str();
                match unit {
                    "k" => "km",
                    "mi" => "miles",
                    "ft" => "feet",
                    "m" => "meter",
                    _ => unit,
                }
            } else {
                "meter"
            };

            // Calculating grade
            let to_grade = format!(
                "{} {} per {} {}",
                elevation, elevation_unit, distance, distance_unit
            );
            let to_mkm = format!(
                "{} {} per {} {} to meter per kilometer",
                elevation, elevation_unit, distance, distance_unit
            );
            let to_ftmi = format!(
                "{} {} per {} {} to feet per mile",
                elevation, elevation_unit, distance, distance_unit
            );

            return match (
                self.eval(&to_grade),
                self.eval(&to_mkm),
                self.eval(&to_ftmi),
            ) {
                (Ok(grade), Ok(mkm), Ok(ftmi)) => {
                    let result = format!("{} -- {} -- {}", grade, mkm, ftmi);
                    Some(
                        result
                            .replace(" (dimensionless)", "")
                            .replace("approx. ", ""),
                    )
                }
                (_, _, _) => None,
            };
        }
        None
    }

    fn handle_feet_to_cm(&self, msg: &str) -> Option<String> {
        if let Some(captures) = self.feet_to_cm_matcher.captures(msg) {
            if let Some(feet) = captures.get(1) {
                if let Some(inches) = captures.get(2) {
                    return Some(format!(
                        "{} feet + {} inches to centimeter",
                        feet.as_str(),
                        inches.as_str()
                    ));
                }
            }
        }
        None
    }

    fn handle_cm_to_feet(&self, msg: &str) -> Option<String> {
        if let Some(captures) = self.cm_to_feet_matcher.captures(msg) {
            if let Some(cm) = captures.get(1) {
                if let Ok(cm) = cm.as_str().parse::<f64>() {
                    let feet = (cm * 0.032_808).floor();
                    let inches = (((cm * 0.393_701) % 12.0) * 1000.0).round() / 1000.0;
                    return Some(format!("{} ft {} in", feet, inches));
                }
            }
        }
        None
    }
}
impl Default for CalcHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl super::MutableHandler for CalcHandler {
    fn handle(&mut self, client: &Client, msg: &Message) {
        if let Command::PRIVMSG(ref channel, ref message) = msg.command {
            if CalcHandler::match_calc(message) {
                match self.eval(&CalcHandler::get_calc_input(message)) {
                    Ok(res) => client.send_privmsg(&channel, &res).unwrap(),
                    Err(e) => {
                        eprintln!("{}", e);
                        client
                            .send_privmsg(&channel, "I had some trouble with that :(")
                            .unwrap()
                    }
                }
            }

            // TODO Integrate with the above...
            if let Some(ref to_eval) = self.handle_shortcut(message) {
                match self.eval(to_eval) {
                    Ok(result) => client.send_privmsg(&channel, &result).unwrap(),
                    Err(e) => {
                        eprintln!("{}", e);
                        client
                            .send_privmsg(&channel, "I had some trouble with that :(")
                            .unwrap()
                    }
                }
            }
            if let Some(ref to_eval) = self.handle_feet_to_cm(message) {
                match self.eval(to_eval) {
                    Ok(result) => client.send_privmsg(&channel, &result).unwrap(),
                    Err(e) => eprintln!("{}", e),
                }
            }
            if let Some(ref paceresult) = self.handle_pace(message) {
                client.send_privmsg(&channel, paceresult).unwrap();
            }
            if let Some(ref cm_to_feet) = self.handle_cm_to_feet(message) {
                client.send_privmsg(&channel, cm_to_feet).unwrap();
            }
            if let Some(ref grade) = self.handle_grade(message) {
                client.send_privmsg(&channel, grade).unwrap();
            }
        }
    }
}

impl super::help::Help for CalcHandler {
    fn name(&self) -> String {
        String::from("calc")
    }

    fn help(&self) -> Vec<super::help::HelpEntry> {
        let result = vec![
            super::help::HelpEntry::new("!calc CALCULATION", "Performs given CALCULATION"),
            super::help::HelpEntry::new(
                "!c NUMBER / !f NUMBER",
                "Convert fahrenheit to celsius and vice versa",
            ),
            super::help::HelpEntry::new(
                "!km NUMBER / !mi NUMBER",
                "Convert miles to kilometre and vice versa",
            ),
            super::help::HelpEntry::new(
                "!kg NUMBER / !lbs NUMBER",
                "Convert pound to kilogram and vice versa",
            ),
            super::help::HelpEntry::new(
                "!cm NUMBER'NUMBER / !ft NUMBER",
                "Convert feet and inches to centimetre and vice versa",
            ),
            super::help::HelpEntry::new(
                "!pace NUMBER:NUMBER",
                "Converts pace per km to pace per mile and vice versa",
            ),
            super::help::HelpEntry::new(
                "!calc NUMBER UNIT to UNIT",
                "Converts number. No spaces in UNIT. 'to UNIT' optional. See https://github.com/tiffany352/rink-rs/blob/master/core/definitions.units for all units.",
            ),
        ];
        result
    }
}
/// There are some simple shortcuts that we want to handle in a generic way. Consider things like
/// !km 26 or !c 100 or !mi 10000 metre. To do so, we make the following assumptions about these
/// shortcuts:
///
/// - The `regex` is used to match user input. First capture of it is the number (+ optional unit)
/// input.
/// - The `target_unit` is the second part inclusion in a `"{} to {}"` format string. The first
/// parameter is the input.
/// - The `default_unit` is there in case the user did not provide a unit.
struct CalcShortcut {
    regex: Regex,
    target_unit: String,
    default_unit: String,
}

struct Pace {
    secs: u32,
}
impl Pace {
    /// Assumes self is in time per km and creates a new Pace with the time
    /// per mile
    pub fn to_per_miles(&self) -> Pace {
        let secs = (self.secs as f32 * 1.6093) as u32;
        Pace { secs }
    }
    /// Assumes self is in time per mile and creates a new Pace with the time
    /// per kilometre
    pub fn to_per_kilometre(&self) -> Pace {
        let secs = (self.secs as f32 / 1.6093) as u32;
        Pace { secs }
    }
}
impl fmt::Display for Pace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mins = self.secs / 60;
        let secs = self.secs % 60;
        write!(f, "{}:{:02}", mins, secs)
    }
}
impl FromStr for Pace {
    type Err = PaceParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        match s.parse::<u32>() {
            Ok(secs) => Ok(Pace { secs }),
            Err(_e) => {
                // Split by non numbers and assume first and second are
                // the numbers representing mins and seconds
                let mut parts = s.split(|c: char| !c.is_digit(10)).take(2);
                // This cannot be the best way to do this...
                // Can't use ? for my error type without rust nightly,
                // which I am trying to avoid.
                if let Some(mins) = parts.next() {
                    if let Some(secs) = parts.next() {
                        // Parse both
                        if let Ok(mins) = mins.parse::<u32>() {
                            if let Ok(secs) = secs.parse::<u32>() {
                                return Ok(Pace {
                                    secs: mins * 60 + secs,
                                });
                            }
                        }
                    }
                }

                Err(PaceParseError {})
            }
        }
    }
}
struct PaceParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_matches() {
        assert!(CalcHandler::match_calc("!calc 5+5"));
        assert!(CalcHandler::match_calc("!CALC 5+5"));
        assert!(CalcHandler::match_calc("!cAlc 5+5"));
        assert!(!CalcHandler::match_calc("!colc 5+5"));
        assert!(!CalcHandler::match_calc("!calca 5+5"));
    }

    #[test]
    fn calc_input() {
        assert_eq!(CalcHandler::get_calc_input("!calc 5+5"), "5+5");
    }

    #[test]
    fn rink_calcer() {
        let mut calc = CalcHandler::new();
        assert_eq!(calc.eval("5+5"), Ok("10 (dimensionless)".to_owned()));
    }

    #[test]
    fn rink_degree_conversion() {
        let mut calc = CalcHandler::new();
        assert_eq!(
            calc.eval("0 celsius in fahrenheit"),
            Ok("32 °F (temperature)".to_owned())
        );
        assert_eq!(
            calc.eval("-40 fahrenheit in celsius"),
            Ok("-40 °C (temperature)".to_owned())
        );
    }

    #[test]
    fn shortcut() {
        let calc = CalcHandler::new();

        assert_eq!(
            calc.handle_shortcut("!km 26"),
            Some("26 miles to kilometre".to_owned())
        );
    }

    #[test]
    fn cm_to_feet() {
        let calc = CalcHandler::new();

        assert_eq!(
            calc.handle_cm_to_feet("!feet 188"),
            Some("6 ft 2.016 in".to_owned())
        );
    }

    #[test]
    fn unicode_line() {
        CalcHandler::match_calc("🤓🤓🤓🤓");
        let calc = CalcHandler::new();
        calc.handle_pace("🤓🤓🤓🤓");
    }

    #[test]
    fn pace_conversion() {
        let calc = CalcHandler::new();
        let res = calc.handle_pace("!pace 5:00");
        assert_eq!(
            res,
            Some("5:00/km = 8:02/mile || 5:00/mile = 3:06/km".to_owned())
        );
    }

    #[test]
    fn grade_calculation() {
        let mut calc = CalcHandler::new();

        let input_output = vec![
            (
                "!grade 10 130",
                "0.013 -- 13 meter / kilometer -- 68.64 foot / mile",
            ),
            ("!grade 23.04 329", "0.01427951 -- 8225/576, 14.27951 meter / kilometer -- 3619/48, 75.39583 foot / mile"),
            ("!grade 101 mile 96 feet", "0.0001800180 -- 0.1800180 meter / kilometer -- 96/101, 0.9504950 foot / mile"),
            ("!grade 101 mi 96 ft", "0.0001800180 -- 0.1800180 meter / kilometer -- 96/101, 0.9504950 foot / mile"),
            ("!grade 101mi 96ft", "0.0001800180 -- 0.1800180 meter / kilometer -- 96/101, 0.9504950 foot / mile"),
            ("!grade 101 k 96ft", "0.0002897108 -- 0.2897108 meter / kilometer -- 1.529673 foot / mile"),
        ];
        for (input, output) in input_output {
            println!("{}", input);
            let res = calc.handle_grade(input);
            assert_eq!(res, Some(output.to_owned()));
        }
    }
}
