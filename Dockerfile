FROM rust:stable-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY plugin.json ./plugin.json
COPY .well-known ./.well-known
COPY extensions ./extensions
RUN cargo build --release --bin jev-seo

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/jev-seo /usr/local/bin/jev-seo
EXPOSE 8080
ENV PORT=8080
CMD ["sh","-c","jev-seo serve --addr 0.0.0.0:${PORT:-8080}"]
