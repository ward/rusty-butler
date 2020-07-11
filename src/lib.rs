#[macro_use]
extern crate serde_derive;

pub mod plugins {
    use irc::client::prelude::*;

    pub trait Handler {
        fn handle(&self, client: &Client, msg: &Message);
    }
    pub trait MutableHandler {
        fn handle(&mut self, client: &Client, msg: &Message);
    }

    pub fn print_msg(msg: &Message) {
        match msg.command {
            Command::PING(_, _) | Command::PONG(_, _) => (),
            _ => print!("{}", msg),
        }
    }

    pub mod time;

    pub mod strava;

    pub mod calc;

    pub mod nickname;

    pub mod lastseen;

    pub mod elo;

    pub mod games;

    pub mod formatting {
        use std::fmt;

        pub enum IrcFormat {
            Bold,
            Normal,
            Underline,
            Italic,
            ForegroundColour(IrcColour),
            BackgroundColour(IrcColour, IrcColour),
        }
        pub enum IrcColour {
            White,
            Black,
            Navy,
            Green,
            Red,
            Brown,
            Purple,
            Olive,
            Yellow,
            LightGreen,
            Teal,
            Cyan,
            Blue,
            Pink,
            Gray,
            LightGray,
        }
        impl fmt::Display for IrcFormat {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    IrcFormat::Bold => write!(f, "\x02"),
                    IrcFormat::Normal => write!(f, "\x0F"),
                    IrcFormat::Underline => write!(f, "\x1F"),
                    IrcFormat::Italic => write!(f, "\x1D"),
                    IrcFormat::ForegroundColour(colour) => write!(f, "\x03{}", colour),
                    IrcFormat::BackgroundColour(text_colour, back_colour) => {
                        write!(f, "\x03{},{}", text_colour, back_colour)
                    }
                }
            }
        }
        impl fmt::Display for IrcColour {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    IrcColour::White => write!(f, "00"),
                    IrcColour::Black => write!(f, "01"),
                    IrcColour::Navy => write!(f, "02"),
                    IrcColour::Green => write!(f, "03"),
                    IrcColour::Red => write!(f, "04"),
                    IrcColour::Brown => write!(f, "05"),
                    IrcColour::Purple => write!(f, "06"),
                    IrcColour::Olive => write!(f, "07"),
                    IrcColour::Yellow => write!(f, "08"),
                    IrcColour::LightGreen => write!(f, "09"),
                    IrcColour::Teal => write!(f, "10"),
                    IrcColour::Cyan => write!(f, "11"),
                    IrcColour::Blue => write!(f, "12"),
                    IrcColour::Pink => write!(f, "13"),
                    IrcColour::Gray => write!(f, "14"),
                    IrcColour::LightGray => write!(f, "15"),
                }
            }
        }
    }
}
