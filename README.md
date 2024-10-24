# Running

You need a `bot.toml` file that follows the format as specified by the [irc
crate](https://github.com/aatxe/irc). Compile with the usual rustup (to install
Rust and friends) and cargo (to actually do the building) commands. Generally
aiming for whatever is the latest version of Rust.

Things I was required to install with `apt` on a fresh Lubuntu installation:

- `build-essential`
- `pkg-config` for openssl-sys crate
- `libssl-dev` for openssl-sys crate

On Arch: `pacman -S openssl gcc pkgconf` though the last two would already be
installed if you installed the `base-devel` group.

## OpenSSL 3

Arch was already ahead of Debian in openssl versions. Notably, openssl 3
removed support for the encryption I had used on debian to have an SASL key.
Got an error:

```
[2022-11-20T12:39:12Z INFO  irc::client::conn] Connecting via TLS to irc.libera.chat.
Error: Tls(Normal(ErrorStack([Error { code: 50856204, library: "digital envelope routines", function: "inner_evp_generic_fetch", reason: "unsupported", file: "crypto/evp/evp_fetch.c", line: 373, data: "Global default library context, Algorithm (RC2-40-CBC : 0), Properties ()" }])))
```

Resolved it by following https://stackoverflow.com/a/72600724/411495.

```
openssl pkcs12 -in cert/keyStore.p12 -nodes -legacy >temp
openssl pkcs12 -in ./temp -export -out cert/keyStore.openssl3.p12
```

# Maintenance

Follow some best practices to keep the code clean:

- Run `cargo fmt` on the code. Ensure it is installed by issuing `rustup
  component add rustfmt-preview` (at the time of writing).
- Run `cargo clippy` on the code. Ensure it is installed by issuing `rustup
  component add clippy` (at the time of writing).
- Run `cargo outdated` to check for outdated dependencies. See
  https://github.com/kbknapp/cargo-outdated. Currently install is via `cargo
  install cargo-outdated`.
- Run `cargo test`.

# To Do/Ideas

- Basic query parsing should probably be centralised, always the same idea.
- Related to that, an alias plugin sounds useful (see also the growing mess in
  games shortshortcut parsing)
- `plugins.toml` should let you decide which plugins to enable.
- Need to filter out a bunch of lesser competitions that nobody cares about.
    - Have an include and exclude list
    - Option to exclude all and only allow explicit includes?
    - Have the list be a configuration, not hardcoded
    - Have an --all or similar option when you _do_ want to look through
      everything

## From Old Gitea Issues

- Add weather via merrysky / its backend api
- Match random strava club links? Not sure I really want that, giving the
  leaderboard doesn't help anyone. Maybe just name / type / location / member
  count. Does not occur often enough to matter though.
- Consistent logging. Use the `log` (I think) crate everywhere instead of
  `println`.
- FIFA and Elo ranking for a country (see `!elo` club command)
- Fantasy Premier League ranking.
- Does it make sense to maybe use a lexer or whatever to analyse user input vs
  adhoc regex?
- Better error messages for `!calc`. Help with conversions / unit names would
  be handy too.
- Anti spam. See if the IRC library offers anything here. Maybe combine with
  more streamlined parsing of the input. But would every plugin then need to
  "register" its catches? Not sure how to best go about that.
- If anti spam works, then see about making the bot reply in private messages.
- Can this be easily turned into a bot for other platforms? Slack, Telegram,
  WhatsApp, idk, something else to make it useful for me.
- Recent IRC activity check for `!strava` command? Meaning if people don't talk
  in the channel, then don't show them in the ranking. Combine `!seen`
  information with the strava-ircnick connections we have.
- Also enable showing last week's strava ranking. Useful the _x_ hours after
  the clock changes.
- Some games incorrectly show up as `@live` games in `!game`. Status: `Aband.`,
  `AAW`, `AHW` (awarded home/away win). Hard to test those last two, the first
  one might appear somewhat regularly.
- Consider a `!strava me` (or `!strava someperson`) option that shows the stats
  for a person and their ranking for each.
