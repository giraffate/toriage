FROM rust:1.49.0-slim-buster

RUN mkdir /toriage
COPY ./Cargo.toml /toriage/Cargo.toml
COPY ./src /toriage/src
COPY ./templates /toriage/templates
WORKDIR /toriage

CMD cargo run --release
