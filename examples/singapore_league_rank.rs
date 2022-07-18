use rusty_butler_lib::plugins::leagueranking::soccerway;

/// Mainly adding this example to debug league rank fetching stuff without starting IRC. Probably a
/// sign I need to extract it to a library.

#[tokio::main]
async fn main() {
    // Run with RUST_LOG=trace cargo run --example singapore_league_rank
    env_logger::init();

    let mut singapore = soccerway::League::new(String::from(
        "https://us.soccerway.com/national/singapore/sleague/2022/regular-season/r66125/",
    ));
    singapore.update().await;
    println!("{:#?}", singapore);
}
