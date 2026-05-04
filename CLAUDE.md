# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Rust + Axum + Chromiumoxide microservice that renders web pages with headless Chromium and returns PNG/JPEG/WebP screenshots. Endpoints: `GET /health`, `GET /screenshot` (query params), `POST /screenshot` (JSON body). HTTP contract is documented in `README.md` — preserve it.

## Commands

Required before handing off changes:

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

Run a single test: `cargo test --locked applies_defaults_like_the_go_service`.

Local run requires Chrome/Chromium on PATH; override with `CHROME_BIN` (or `CHROMIUM_BIN` / `CHROME_PATH`):

```bash
export CHROME_BIN=/path/to/chromium
cargo run
```

Container smoke test: `docker build -t screenshot-service:rust .` and `docker compose up -d --build` (clean up with `docker compose down`). Stay aligned with the pinned `Cargo.lock`.

## Architecture

Four-module split under `src/` — keep responsibilities separated:

- `main.rs` — Axum router, body-size limit, tracing init, graceful shutdown. `process_screenshot` is the shared post-validation pipeline that calls `take_screenshot`, then attaches `Content-Type` / `Content-Length` / `Content-Disposition` / `Cache-Control` headers to the raw image bytes.
- `request.rs` — `ScreenshotRequest` (POST JSON shape) and `ScreenshotQuery` (GET shape, where `headers` and `clip` arrive as JSON-encoded strings and are parsed into the request). All defaulting and clamping lives in `apply_defaults`; bounds checks live in `validate`. Both run in `process_screenshot` before any browser work.
- `screenshot.rs` — Chromium lifecycle. Each request launches a fresh `Browser` with a per-request `tempfile` user-data dir, spawns a handler task to drain CDP events, then runs `capture_page` under a `tokio::time::timeout` derived from `req.timeout`. Browser/page/handler are torn down in all paths (errors only logged at debug). Three capture modes: full-page (re-overrides device metrics to the layout content size, capped at 16384px, uses `capture_beyond_viewport`), `clip`, or plain viewport. `wait_for` polls `find_element` + `bounding_box` every 100 ms until visible or the request timeout elapses.
- `error.rs` — `AppError::BadRequest` → 400, `AppError::Screenshot` → 500. Always return the `{"error": "..."}` JSON shape.

## Conventions

- Do not reintroduce Go files (`go.mod`, `go.sum`, Gin/chromedp). The repo was rewritten from Go; old files are deleted in working tree but visible in `git log`.
- Keep abstractions small — the four-module layout is intentional.
- Image bytes flow through `axum::body::Body` directly; do not buffer through additional copies.

## Git Commits

Subject format: `[Type] Short description starting with capital letter`.

Allowed types: `[Feat]`, `[Fix]`, `[Chore]`, `[Docs]`.

Rules:

- Capital first letter, imperative mood (`Add`, not `Added`), no trailing period, ~70 chars max.
- Agent attribution uses the standard Git `Co-authored-by:` trailer on its own line, separated from the subject by a blank line — not a free-form `Agent:` line. Use `Co-authored-by: Claude Opus 4.7 <noreply@anthropic.com>` (substitute the actual model used).

Project examples:

```text
[Feat] Add WebP quality control to screenshot params
[Fix] Reset device metrics before full-page capture
[Chore] Pin chromiumoxide to 0.9.1
[Docs] Document CHROME_BIN fallback chain in README
```

Agent-authored example:

```text
[Docs] Add CLAUDE.md with architecture overview

Co-authored-by: Claude Opus 4.7 <noreply@anthropic.com>
```
