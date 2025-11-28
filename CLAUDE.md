# Repository Guidelines

## Project Structure & Module Organization
- `src/` hosts the backend entrypoint (`main.rs`) built with Tokio and Axum.
- `crates/todoapp-model/` defines data models; `crates/todoapp-transfer/` holds DTOs shared across the stack.
- `crates/todoapp-frontend/` is the Dioxus web client; assets live in `crates/todoapp-frontend/assets/`, with styling in `tailwind.css`.
- `target/` is build output; avoid committing it. Workspace-level `Cargo.toml` centralizes dependencies.

## Build, Test, and Development Commands
- `cargo build` compiles the entire workspace.
- `cargo test` runs all Rust tests across crates.
- `cargo run` launches the backend from `src/main.rs`.
- From `crates/todoapp-frontend/`: `dx serve` starts the Dioxus dev server (hot reload, Tailwind auto-build); `dx build --release` produces a production bundle.
- Add `--package <crate>` to scope commands to a specific crate (e.g., `cargo test --package todoapp-model`).

## Coding Style & Naming Conventions
- Rust code follows `rustfmt` defaults (4-space indent; snake_case for functions/vars, CamelCase for types). Run `cargo fmt` before committing.
- Prefer `clippy`-clean code (`cargo clippy --all-targets --all-features`) and handle warnings deliberately.
- Keep Dioxus components small and typed; colocate per-page styles/assets under `crates/todoapp-frontend/`.
- Favor descriptive module names aligned with domain concepts (e.g., `model`, `transfer`, `routes`).

## Testing Guidelines
- Use Rust unit tests in the same module when possible; integration tests can live under `tests/` per crate.
- Name tests for behavior (`add_calculates_sum`, `blog_route_navigates`).
- Aim to cover DTO serialization/deserialization and any stateful sled interactions; add regression tests for bugs.
- Frontend: prefer logic extracted into testable Rust functions; keep snapshots stable.

## Commit & Pull Request Guidelines
- Write concise commits in imperative mood (`Add blog route`, `Refactor transfer types`); group related changes.
- Include context in PR descriptions: purpose, key changes, testing performed (`cargo test`, `dx serve` smoke-checked), and any follow-ups.
- Link issues when applicable; add screenshots/gifs for frontend-visible changes.
- Keep branches focused; rebase instead of merging from main when cleaning up history.

## Security & Configuration Tips
- Do not commit secrets; use env vars or local config files ignored by Git.
- Validate new dependencies for maintenance and license compatibility; prefer workspace versions to avoid drift.
