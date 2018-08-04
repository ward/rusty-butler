use irc::client::prelude::*;
use regex::Regex;
use rink;

pub struct CalcHandler {
    ctx: rink::Context,
    shortcuts: Vec<CalcShortcut>,
}
impl CalcHandler {
    pub fn new() -> CalcHandler {
        let mut ctx = rink::load().expect("Could not create calculator core?");
        ctx.short_output = true;
        let mut shortcuts = vec![];
        shortcuts.push(CalcShortcut {
            regex: Regex::new(r"!km +(\d.*)$").unwrap(),
            target_unit: "kilometre".to_owned(),
            default_unit: "miles".to_owned(),
        });
        shortcuts.push(CalcShortcut {
            regex: Regex::new(r"!mi(?:le)? +(\d.*)$").unwrap(),
            target_unit: "miles".to_owned(),
            default_unit: "kilometer".to_owned(),
        });
        shortcuts.push(CalcShortcut {
            regex: Regex::new(r"^!c +(-?\d.*)$").unwrap(),
            target_unit: "celsius".to_owned(),
            default_unit: "fahrenheit".to_owned(),
        });
        shortcuts.push(CalcShortcut {
            regex: Regex::new(r"^!f +(-?\d.*)$").unwrap(),
            target_unit: "fahrenheit".to_owned(),
            default_unit: "celsius".to_owned(),
        });
        shortcuts.push(CalcShortcut {
            regex: Regex::new(r"^!kg +(-?\d.*)$").unwrap(),
            target_unit: "kilogram".to_owned(),
            default_unit: "lbs".to_owned(),
        });
        shortcuts.push(CalcShortcut {
            regex: Regex::new(r"^!(?:lbs?|pound) +(-?\d.*)$").unwrap(),
            target_unit: "lbs".to_owned(),
            default_unit: "kilogram".to_owned(),
        });
        CalcHandler {
            ctx,
            shortcuts,
        }
    }
    fn match_calc(msg: &str) -> bool {
        msg.len() > 5 && msg[..6].eq_ignore_ascii_case("!calc ")
    }

    fn get_calc_input(msg: &str) -> &str {
        msg[5..].trim()
    }

    fn eval(&mut self, line: &str) -> Result<String, String> {
        rink::one_line(&mut self.ctx, line)
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
}

impl super::MutableHandler for CalcHandler {
    fn handle(&mut self, client: &IrcClient, msg: &Message) {
        if let Command::PRIVMSG(ref channel, ref message) = msg.command {
            if CalcHandler::match_calc(message) {
                match self.eval(CalcHandler::get_calc_input(message)) {
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
        }
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
}
