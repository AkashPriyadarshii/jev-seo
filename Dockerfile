FROM rust:1-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY plugin.json ./plugin.json
COPY .well-known ./.well-known
COPY mcp.json ./mcp.json
COPY .app.json ./.app.json
COPY skills ./skills
COPY extensions ./extensions
RUN cargo build --release --bin jev-seo

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /srv
COPY --from=build /app/target/release/jev-seo /usr/local/bin/jev-seo
COPY --from=build /app/plugin.json ./plugin.json
COPY --from=build /app/.well-known ./.well-known
COPY --from=build /app/mcp.json ./mcp.json
COPY --from=build /app/.app.json ./.app.json
COPY --from=build /app/skills ./skills
COPY --from=build /app/extensions ./extensions
EXPOSE 8080
ENV PORT=8080
CMD ["sh","-c","jev-seo serve --addr 0.0.0.0:${PORT:-8080}"]
