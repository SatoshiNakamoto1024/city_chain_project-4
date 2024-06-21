FROM rust:latest

WORKDIR /usr/src/app
COPY . .

RUN apt-get update && apt-get install -y pkg-config libssl-dev
RUN cargo build --release

CMD ["./target/release/municipal_chain"]
