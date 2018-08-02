extern crate chrono;
extern crate irc;
extern crate regex;
extern crate reqwest;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
// Calculations
extern crate rink;

pub mod plugins {
    use irc::client::prelude::*;

    pub trait Handler {
        fn handle(&self, client: &IrcClient, msg: &Message);
    }
    pub trait MutableHandler {
        fn handle(&mut self, client: &IrcClient, msg: &Message);
    }

    pub fn print_msg(msg: &Message) {
        print!("{}", msg);
    }

    pub mod time;

    pub mod strava;

    pub mod calc;

    pub mod nickname;
}
