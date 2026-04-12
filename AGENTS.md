# AGENTS.md

chauve is an alert display app. it is built using rust 1.94.1 and dioxus 0.7.1.
it is targeted for linux, windows and macOS desktops.

## Configuration
- Alert providers configured in `Settings.toml` (Grafana format example in README)

## Tests
- from the package root, run `cargo test`

## Code Structure
- code is layered and should respect separation of concerns. use dependency injection if necessary
- `src/entities/`: Data models/core entities
- `src/providers/`: External service integrations (Grafana implemented)
- `src/ui/`: Dioxus components with Tailwind CSS
- `src/poller.rs`: Alert polling logic
- `src/settings.rs`: Configuration management

## Code guidelines
- write small understandable functions, code should be boring and easy to understand
- after doing changes: search for bugs, and ensure `cargo check` passes
- only change what is needed, do not refactor without being asked