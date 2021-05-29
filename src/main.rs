use rusty_butler_lib::plugins;
use rusty_butler_lib::plugins::Handler;
use rusty_butler_lib::plugins::MutableHandler;

use futures::prelude::*;
use irc::client::prelude::*;
use std::sync::Mutex;

use clap::{App, Arg};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("rusty-butler")
        .version("0.3.0")
        .author("Ward Muylaert")
        .about("An IRC bot. The world needed more of those.")
        .arg(
            Arg::with_name("config")
                .long("config")
                .value_name("FILE")
                .help("Use a different configuration file")
                .default_value("bot.toml"),
        )
        .get_matches();

    let config_file_name = matches.value_of("config").unwrap();
    let config = Config::load(config_file_name).expect("Failed to load config");
    let config_for_handlers = Config::load(config_file_name).expect("Failed to load config");

    let mut client = Client::from_config(config).await?;
    client.identify().expect("Failed to identify");
    let mut stream = client.stream()?;

    // Non mutable handlers
    let mut help_handler = plugins::help::HelpHandler::new();
    let strava_handler = plugins::strava::StravaHandler::new(&config_for_handlers);
    help_handler.add_help(&strava_handler);
    let time_handler = plugins::time::TimeHandler::new();
    help_handler.add_help(&time_handler);
    let untappd_handler = plugins::untappd::UntappdHandler::new(&config_for_handlers);
    help_handler.add_help(&untappd_handler);
    let simple_reply_handler = plugins::simple_reply::SimpleReplyHandler::new();
    help_handler.add_help(&simple_reply_handler);
    let mut handlers: Vec<Box<dyn Handler>> = vec![];
    handlers.push(Box::new(strava_handler));
    handlers.push(Box::new(time_handler));
    handlers.push(Box::new(untappd_handler));
    handlers.push(Box::new(simple_reply_handler));

    // Mutable handlers
    let nickname_handler = plugins::nickname::NicknameHandler::new(&config_for_handlers);
    help_handler.add_help(&nickname_handler);
    let calc_handler = plugins::calc::CalcHandler::new();
    help_handler.add_help(&calc_handler);
    let last_seen_handler = plugins::lastseen::LastSeenHandler::new();
    help_handler.add_help(&last_seen_handler);
    let elo_handler = plugins::elo::EloHandler::new();
    help_handler.add_help(&elo_handler);
    let games_handler = plugins::games::GamesHandler::new();
    help_handler.add_help(&games_handler);
    let ranking_handler = plugins::leagueranking::LeagueRankingHandler::new();
    help_handler.add_help(&ranking_handler);
    let mut mutable_handlers: Vec<Mutex<Box<dyn MutableHandler>>> = vec![];
    mutable_handlers.push(Mutex::new(Box::new(nickname_handler)));
    mutable_handlers.push(Mutex::new(Box::new(calc_handler)));
    mutable_handlers.push(Mutex::new(Box::new(last_seen_handler)));
    mutable_handlers.push(Mutex::new(Box::new(elo_handler)));
    mutable_handlers.push(Mutex::new(Box::new(games_handler)));
    mutable_handlers.push(Mutex::new(Box::new(ranking_handler)));

    // Could not move help_handler before
    handlers.push(Box::new(help_handler));

    // TODO Should these handlers all become async? There should not be much intersection so
    // perhaps not worth the effort. Only one will _truly_ react to a message.
    while let Some(irc_msg) = stream.next().await.transpose()? {
        plugins::print_msg(&irc_msg);
        for handler in &handlers {
            handler.handle(&client, &irc_msg);
        }
        for mutable_handler in &mutable_handlers {
            // TODO Is there a possibility of this slowing things down in unforseen ways?
            let mut mutable_handler = mutable_handler.lock().expect("Likely fatal! Getting a lock failed which implies another thread holding the lock panicked");
            mutable_handler.handle(&client, &irc_msg);
        }
    }

    Ok(())
}
