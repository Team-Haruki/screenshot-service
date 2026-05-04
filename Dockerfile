FROM rust:1.95.0-alpine AS builder

WORKDIR /app

RUN apk add --no-cache musl-dev pkgconfig

COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && \
    printf 'fn main() {}\n' > src/main.rs && \
    cargo fetch --locked && \
    rm -rf src

COPY src ./src
RUN cargo build --locked --profile release

FROM alpine:3.22

RUN apk add --no-cache \
    ca-certificates \
    chromium \
    dumb-init \
    freetype \
    font-noto-cjk \
    font-noto-emoji \
    harfbuzz \
    nss \
    ttf-freefont \
    wget \
    && rm -rf /var/cache/apk/*

ENV CHROME_BIN=/usr/bin/chromium-browser \
    RUST_LOG=screenshot_service=info,tower_http=info

RUN addgroup -g 1000 appgroup && \
    adduser -u 1000 -G appgroup -s /bin/sh -D appuser

WORKDIR /app
COPY --from=builder /app/target/release/screenshot-service ./screenshot-service
RUN chown -R appuser:appgroup /app

USER appuser

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

ENTRYPOINT ["/usr/bin/dumb-init", "--"]
CMD ["./screenshot-service"]
