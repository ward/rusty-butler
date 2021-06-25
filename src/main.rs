use rusty_butler_lib::plugins;
use rusty_butler_lib::plugins::AsyncMutableHandler;
use rusty_butler_lib::plugins::Handler;
use rusty_butler_lib::plugins::MutableHandler;

use futures::prelude::*;
use irc::client::prelude::*;
use std::sync::Mutex;

use clap::{App, Arg};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("rusty-butler")
        .version("0.4.0")
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
    let plugin_config = plugins::config::Config::new();

    let mut client = Client::from_config(config.clone()).await?;
    // Asks server if it can do SASL, once acknowledged (see later, we can ask to authenticate with
    // it).
    client.send_cap_req(&[Capability::Sasl])?;
    // Identify with SASL instead of nickserv password sending
    // Need to set client_cert_path and client_cert_pass in bot.toml
    // The cert needs to be p12 format. Probably need to set use_ssl and use_tls to true too
    // client.identify().expect("Failed to identify");
    // .identify() would send these for us, so just emulate that
    client.send(Command::NICK(config.nickname()?.to_string()))?;
    client.send(Command::USER(
        config.username().to_string(),
        "0".to_owned(),
        config.real_name().to_string(),
    ))?;
    let mut stream = client.stream()?;

    // Non mutable handlers
    let mut help_handler = plugins::help::HelpHandler::new();
    let time_handler = plugins::time::TimeHandler::new();
    help_handler.add_help(&time_handler);
    let simple_reply_handler = plugins::simple_reply::SimpleReplyHandler::new(&plugin_config);
    help_handler.add_help(&simple_reply_handler);
    let mut handlers: Vec<Box<dyn Handler>> =
        vec![Box::new(time_handler), Box::new(simple_reply_handler)];

    // Mutable handlers
    let nickname_handler = plugins::nickname::NicknameHandler::new(&config_for_handlers);
    help_handler.add_help(&nickname_handler);
    let calc_handler = plugins::calc::CalcHandler::new();
    help_handler.add_help(&calc_handler);
    let last_seen_handler = plugins::lastseen::LastSeenHandler::new();
    help_handler.add_help(&last_seen_handler);
    let games_handler = plugins::games::GamesHandler::new();
    help_handler.add_help(&games_handler);
    let mutable_handlers: Vec<Mutex<Box<dyn MutableHandler>>> = vec![
        Mutex::new(Box::new(nickname_handler)),
        Mutex::new(Box::new(calc_handler)),
        Mutex::new(Box::new(last_seen_handler)),
        Mutex::new(Box::new(games_handler)),
    ];

    // Async mutable handlers
    let elo_handler = plugins::elo::EloHandler::new();
    help_handler.add_help(&elo_handler);
    let fantasy_handler = plugins::fantasy::FantasyHandler::new(&plugin_config);
    help_handler.add_help(&fantasy_handler);
    let ranking_handler = plugins::leagueranking::LeagueRankingHandler::new();
    help_handler.add_help(&ranking_handler);
    let strava_handler = plugins::strava::StravaHandler::new(&config_for_handlers);
    help_handler.add_help(&strava_handler);
    let untappd_handler = plugins::untappd::UntappdHandler::new(&config_for_handlers);
    help_handler.add_help(&untappd_handler);
    let async_mutable_handlers: Vec<Mutex<Box<dyn AsyncMutableHandler>>> = vec![
        Mutex::new(Box::new(elo_handler)),
        Mutex::new(Box::new(fantasy_handler)),
        Mutex::new(Box::new(ranking_handler)),
        Mutex::new(Box::new(strava_handler)),
        Mutex::new(Box::new(untappd_handler)),
    ];

    // Could not move help_handler before
    handlers.push(Box::new(help_handler));

    // TODO Should these handlers all become async? There should not be much intersection so
    // perhaps not worth the effort. Only one will _truly_ react to a message.
    while let Some(irc_msg) = stream.next().await.transpose()? {
        plugins::print_msg(&irc_msg);

        // Should I move this SASL stuff to its own module?
        // Cleaner still would be seeing how I can get it into upstream.
        match irc_msg.command {
            Command::CAP(_, ref subcommand, _, _) => {
                if subcommand.to_str() == "ACK" {
                    println!("Recieved ack for sasl");
                    // client.send_sasl_plain()?;
                    client.send_sasl_external()?;
                }
            }
            Command::AUTHENTICATE(_) => {
                println!("Got signal to continue authenticating");
                client.send(Command::AUTHENTICATE(String::from('+')))?;
                // client.send(Command::AUTHENTICATE(base64::encode(format!(
                //     "{}\x00{}\x00{}",
                //     config.nickname()?.to_string(),
                //     config.nickname()?.to_string(),
                //     config.password().to_string()
                // ))))?;
                client.send(Command::CAP(None, "END".parse()?, None, None))?;
            }
            Command::Response(code, _) => {
                if code == Response::RPL_SASLSUCCESS {
                    println!("Successfully authenticated");
                    client.send(Command::CAP(None, "END".parse()?, None, None))?;
                }
            }
            _ => {}
        };

        for handler in &handlers {
            handler.handle(&client, &irc_msg);
        }
        for mutable_handler in &mutable_handlers {
            // TODO Is there a possibility of this slowing things down in unforseen ways?
            let mut mutable_handler = mutable_handler.lock().expect("Likely fatal! Getting a lock failed which implies another thread holding the lock panicked");
            mutable_handler.handle(&client, &irc_msg);
        }
        for async_mutable_handler in &async_mutable_handlers {
            let mut async_mutable_handler = async_mutable_handler.lock().expect("Likely fatal! Getting a lock failed which implies another thread holding the lock panicked");
            async_mutable_handler.handle(&client, &irc_msg).await;
        }
    }

    Ok(())
}
