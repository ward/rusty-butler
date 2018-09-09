# Running

You need a `bot.toml` file that follows the format as specified by the [irc
crate](https://github.com/aatxe/irc). Compile with the usual rustup (to install
Rust and friends) and cargo (to actually do the building) commands. Generally
aiming for whatever is the latest version of Rust.

Things I was required to install with `apt` on a fresh Lubuntu installation:

- `build-essential`
- `pkg-config` for openssl-sys crate
- `libssl-dev` for openssl-sys crate
- `libgmp-dev` (possibly) for the rink crate

# Contributing

Started this on a private gitea instance so the issue list is still there too.
Guess I should bring them over to GitHub at some point so people can actually
contribute.  For now just drop ideas on IRC (freenode -> ward), sorry.
