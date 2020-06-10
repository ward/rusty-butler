# Makes it easier to compile for the Debian VPS from other platforms.
# docker build -t wardmuylaert/rusty-butler-builder .
# docker run --rm -it -v $(pwd):/project wardmuylaert/rusty-butler-builder

FROM debian:buster

RUN apt-get update
RUN apt-get install -y curl build-essential pkg-config libssl-dev libgmp-dev

# This is not very reproducible as it just gets the latest rustup and latest rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

WORKDIR /project

CMD ["/root/.cargo/bin/cargo", "build", "--target-dir", "target-vps", "--release"]
