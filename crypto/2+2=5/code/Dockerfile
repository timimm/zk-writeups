FROM rust:1.67

WORKDIR /usr/src/app

COPY ./jolt ./jolt
COPY ./server ./server

WORKDIR /usr/src/app/server

RUN cargo build --release

# run the server once just to download the toolchain, this will crash but we do not care
# god has already forgiven me for this
# (also this has nothing to do with the challenge itself)
RUN (echo "" | ./target/release/challenge); true

RUN apt update && apt install -y socat

ENTRYPOINT ["socat", "-t", "900", "-T", "120", "TCP4-LISTEN:5555,reuseaddr,fork", "EXEC:./target/release/challenge"]
