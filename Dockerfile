FROM rust:latest

RUN apt update && apt upgrade -y
RUN apt install -y libgmp-dev
