use irc::client::prelude::*;
use rink;

pub struct CalcHandler {}
impl CalcHandler {
    pub fn new() -> CalcHandler {
        CalcHandler {}
    }
    fn match_calc(msg: &str) -> bool {
        msg.len() > 5 && msg[..6].eq_ignore_ascii_case("!calc ")
    }

    fn get_calc_input(msg: &str) -> &str {
        msg[5..].trim()
    }

    fn eval(line: &str) -> Result<String, String> {
        // is load() heavy, documentation says it opens definition files
        let mut ctx = rink::load()?;
        ctx.short_output = true;
        rink::one_line(&mut ctx, line)
    }
}

impl super::Handler for CalcHandler {
    fn handle(&self, client: &IrcClient, msg: &Message) {
        if let Command::PRIVMSG(ref channel, ref message) = msg.command {
            if CalcHandler::match_calc(message) {
                match CalcHandler::eval(CalcHandler::get_calc_input(message)) {
                    Ok(res) => client.send_privmsg(&channel, &res).unwrap(),
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
        assert_eq!(
            CalcHandler::eval("5+5"),
            Ok("10 (dimensionless)".to_owned())
        );
    }

    #[test]
    fn rink_degree_conversion() {
        assert_eq!(
            CalcHandler::eval("0 celsius in fahrenheit"),
            Ok("32 °F (temperature)".to_owned())
        );
        assert_eq!(
            CalcHandler::eval("-40 fahrenheit in celsius"),
            Ok("-40 °C (temperature)".to_owned())
        );
    }
}
