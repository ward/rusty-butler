use irc::client::prelude::*;
use rink;

pub struct CalcHandler {
    ctx: rink::Context,
}
impl CalcHandler {
    pub fn new() -> CalcHandler {
        let mut ctx = rink::load().expect("Could not create calculator core?");
        ctx.short_output = true;
        CalcHandler {
            ctx
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
        let mut calc = CalcHandler::new();
        assert_eq!(
            calc.eval("5+5"),
            Ok("10 (dimensionless)".to_owned())
        );
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
}
