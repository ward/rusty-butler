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
