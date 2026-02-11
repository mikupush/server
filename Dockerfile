FROM rust:1.88 AS build

WORKDIR /builder

COPY . .

RUN cargo build --release

FROM node:22.18.0 AS build-web

WORKDIR /builder

COPY . .

RUN npm install && npm run build

FROM debian:bookworm-slim

WORKDIR /srv

RUN apt update && apt install -y \
    libssl3 \
    ca-certificates \
    && update-ca-certificates

ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs
ENV REQUESTS_CA_BUNDLE=/etc/ssl/certs/ca-certificates.crt
ENV RUST_BACKTRACE=0
ENV RUST_LOG=info

COPY --from=build /builder/target/release/mikupush-server .
COPY --from=build-web /builder/dist dist
RUN chmod +x /srv/mikupush-server

ENTRYPOINT ["/srv/mikupush-server"]
