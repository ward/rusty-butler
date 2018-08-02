use irc::client::prelude::*;
use std::time::{Duration, Instant};

pub struct NicknameHandler {
    nick: Option<String>,
    last_attempt: Instant,
    waiting_time: Duration,
}

impl NicknameHandler {
    pub fn new(config: &Config) -> NicknameHandler {
        let nick = match &config.nickname {
            Some(nickname) => Some(nickname.clone()),
            None => None,
        };
        NicknameHandler {
            nick,
            last_attempt: Instant::now(),
            waiting_time: Duration::new(5*60, 0),
        }
    }
    fn is_it_time(&self) -> bool {
        let now = Instant::now();
        let diff = now - self.last_attempt;
        diff > self.waiting_time
    }
    fn reset_time(&mut self) {
        self.last_attempt = Instant::now();
    }
}

impl super::MutableHandler for NicknameHandler {
    fn handle(&mut self, client: &IrcClient, _msg: &Message) {
        if self.is_it_time() {
            self.reset_time();
            match self.nick {
                Some(ref nick) => {
                    if client.current_nickname() != nick {
                        client.send(Command::NICK(nick.to_string())).unwrap();
                    }
                },
                None => (),
            }
        }
    }
}
