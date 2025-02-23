FROM rust:bookworm AS build

WORKDIR /app

RUN apt-get update -y

RUN rustup upgrade

COPY . .

RUN cargo build --release

FROM rust:bookworm AS run

WORKDIR /app

COPY --from=build /app/target/release/blag blag
RUN git clone --depth=1 https://github.com/whynotavailable/my-site.git sitecode

ENTRYPOINT ["./blag", "sitecode"]
