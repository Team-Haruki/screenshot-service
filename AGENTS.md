# AGENTS.md

## Project

This service is a Rust + Axum + Chromiumoxide screenshot API.

Important endpoints:

- `GET /health`
- `GET /screenshot`
- `POST /screenshot`

The implementation lives in `src/`. Keep request parsing and validation in `src/request.rs`, HTTP wiring in `src/main.rs`, and Chromium work in `src/screenshot.rs`.

## Commands

Run these before handing off changes:

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
docker build -t screenshot-service:rust .
```

Use `docker compose up -d --build` for a full local container smoke test, then clean it up with `docker compose down`.

## Conventions

- Preserve the existing HTTP contract documented in `README.md`.
- Keep the service compatible with the pinned Docker Rust toolchain.
- Do not reintroduce Go files, `go.mod`, or `go.sum`.
- Keep generated artifacts out of git: `target/`, `.idea/`, local screenshots, and logs.
- Prefer small, focused changes over broad rewrites.

## Runtime Notes

Local non-Docker runs require Chrome or Chromium. Set `CHROME_BIN` when auto-detection is not enough.

The Docker image installs Alpine Chromium and runs as a non-root user behind `dumb-init`.

## Git Commits

Subject format: `[Type] Short description starting with capital letter`.

Allowed types:

| Type      | Usage                                                 |
|-----------|-------------------------------------------------------|
| `[Feat]`  | New feature or capability                             |
| `[Fix]`   | Bug fix                                               |
| `[Chore]` | Maintenance, refactoring, dependency or build changes |
| `[Docs]`  | Documentation-only changes                            |

Rules:

- Description starts with a capital letter, imperative mood (`Add`, not `Added`).
- No trailing period; keep the subject at or below ~70 characters.
- Agent attribution uses the standard Git `Co-authored-by:` trailer in the commit body, not a free-form `Agent:` line. Place it on its own line, separated from the subject by a blank line. Suggested values:
  - Claude (any 4.x): `Co-authored-by: Claude Opus 4.7 <noreply@anthropic.com>` (substitute the actual model, e.g. `Claude Sonnet 4.6`)
  - Codex: `Co-authored-by: Codex <noreply@openai.com>`
  - Copilot: `Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>`

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
