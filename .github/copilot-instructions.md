# Copilot Instructions

This repository is a Rust screenshot microservice built with Axum and Chromiumoxide.

## Development Guidance

- Keep routing and response handling in `src/main.rs`.
- Keep request structs, defaults, and validation in `src/request.rs`.
- Keep browser launch and CDP screenshot behavior in `src/screenshot.rs`.
- Return JSON error bodies in the existing `{"error":"..."}` shape.
- Preserve the documented `/health` and `/screenshot` behavior.
- Do not add Go code or restore the old Gin/chromedp implementation.

## Required Checks

Before suggesting a finished change, run or account for:

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

For container changes, also run:

```bash
docker build -t screenshot-service:rust .
```

## Style

- Prefer simple Rust modules and explicit error context.
- Avoid large abstractions unless they reduce real duplication.
- Keep Docker and CI behavior aligned with the checked-in `Cargo.lock`.

## Git Commits

Subject format: `[Type] Short description starting with capital letter`.

Allowed types: `[Feat]` (new capability), `[Fix]` (bug fix), `[Chore]` (maintenance, refactor, deps, build), `[Docs]` (documentation only).

Rules:

- Capital first letter, imperative mood (`Add`, not `Added`), no trailing period, ~70 chars max.
- Agent attribution uses the standard Git `Co-authored-by:` trailer (not a free-form `Agent:` line), on its own line separated from the subject by a blank line. Suggested values:
  - Claude (any 4.x): `Co-authored-by: Claude Opus 4.7 <noreply@anthropic.com>` (substitute the actual model)
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

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
```
