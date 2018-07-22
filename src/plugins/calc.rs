use irc::client::prelude::*;
use rink;

pub fn handler(client: &IrcClient, msg: &Message) {
    if let Command::PRIVMSG(ref channel, ref message) = msg.command {
        if match_calc(message) {
            match eval(get_calc_input(message)) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_matches() {
        assert!(match_calc("!calc 5+5"));
        assert!(match_calc("!CALC 5+5"));
        assert!(match_calc("!cAlc 5+5"));
        assert!(!match_calc("!colc 5+5"));
        assert!(!match_calc("!calca 5+5"));
    }

    #[test]
    fn calc_input() {
        assert_eq!(get_calc_input("!calc 5+5"), "5+5");
    }

    #[test]
    fn rink_calcer() {
        assert_eq!(eval("5+5"), Ok("10 (dimensionless)".to_owned()));
    }

    #[test]
    fn rink_degree_conversion() {
        assert_eq!(
            eval("0 celsius in fahrenheit"),
            Ok("32 °F (temperature)".to_owned())
        );
        assert_eq!(
            eval("-40 fahrenheit in celsius"),
            Ok("-40 °C (temperature)".to_owned())
        );
    }
}
