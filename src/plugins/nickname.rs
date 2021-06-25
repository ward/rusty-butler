use irc::client::prelude::*;
use std::time::{Duration, Instant};

pub struct NicknameHandler {
    nick: Option<String>,
    nickserv_password: Option<String>,
    last_attempt: Instant,
    waiting_time: Duration,
}

impl NicknameHandler {
    pub fn new(config: &Config) -> NicknameHandler {
        let nick = config.nickname.as_ref().cloned();
        let nickserv_password = config.nick_password.as_ref().cloned();
        NicknameHandler {
            nick,
            nickserv_password,
            last_attempt: Instant::now(),
            waiting_time: Duration::new(5 * 60, 0),
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
    fn retake_nick(&self, client: &Client) {
        if let Some(ref nick) = self.nick {
            if client.current_nickname() != nick {
                client.send(Command::NICK(nick.to_string())).unwrap();
            }
        }
    }
    fn handle_nickserv(&self, client: &Client, msg: &Message) {
        // NOTE The irc library we use already has some logic surrounding logging in. See fn
        // `ClientState::send_nick_password(&self)`. That function gets called automatically at the
        // end of the MOTD (or when the notice is sent that there is no MOTD). Effectively, this
        // means that when logging in, both that function and this one will trigger. As far as I
        // can tell though, the other one will not trigger at any point afterwards, while we *do*
        // want ours to trigger in case we retake a name.
        if let Some(ref pass) = self.nickserv_password {
            if let Command::NICKSERV(ref text) = msg.command {
                if text.contains(&"This nickname is registered.".to_owned()) {
                    client
                        .send(Command::NICKSERV(vec![
                            "IDENTIFY".to_string(),
                            pass.to_owned(),
                        ]))
                        .unwrap();
                }
            }
        }
    }
}

impl super::MutableHandler for NicknameHandler {
    fn handle(&mut self, client: &Client, msg: &Message) {
        if self.is_it_time() {
            self.reset_time();
            self.retake_nick(client);
        }
        self.handle_nickserv(client, msg);
    }
}

impl super::help::Help for NicknameHandler {
    fn name(&self) -> String {
        String::from("nickinternal")
    }

    fn help(&self) -> Vec<super::help::HelpEntry> {
        vec![]
    }
}
