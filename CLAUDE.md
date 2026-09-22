# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

Chauve is a cross-platform desktop alert notification system built with Rust and Dioxus. It polls alert providers (currently Grafana) and displays them in a native desktop UI with real-time refresh.

## Commands

```bash
# Format + lint (run before every commit)
cargo fmt && cargo clippy -- -D warnings

# Run tests
cargo test

# Verify code compiles (no test run)
cargo check

# Dev server (desktop)
dx serve --desktop

# Watch and build Tailwind CSS (run in a separate shell alongside dx serve)
npx @tailwindcss/cli -i ./assets/source.css -o ./assets/main.css --watch

# Release build
dx build --release --platform desktop
npx @tailwindcss/cli -i ./assets/source.css -o ./assets/main.css --minify
```

Dioxus CLI must be installed: run `./scripts/install-dx.sh`. It installs `dx` at exactly the resolved `dioxus` version — `dx` refuses to build against any other version.

## Architecture

Dioxus version: **0.7.10**, pinned exactly (`=0.7.10`) in `Cargo.toml`. Check `Cargo.toml` before using APIs — Dioxus has breaking changes between versions. The pin is exact because `dx` must match the crate version; both CI (`.github/workflows/release.yml`) and `scripts/install-dx.sh` derive the `dx` version from `cargo pkgid dioxus`, so after a bump re-run the script.

The codebase is layered with strict separation of concerns:

**`src/entities/`** — Core domain types. `Alert` is the normalized alert structure used throughout the app. `Provider` and `ProviderAlert` are traits that define the extension points for adding new alert sources. New providers must implement these.

**`src/providers/`** — External service integrations. Each provider lives in its own subdirectory (e.g., `providers/grafana/`). The provider fetches raw data from the API and converts it to `Alert` via `ProviderAlert`. Currently only Grafana is implemented, connecting to Grafana's Alertmanager API with bearer token auth.

**`src/poller.rs`** — Orchestration layer. `Poller` holds a list of `Provider` instances and calls them all in parallel via `join_all`. Returns `PollerData` with aggregated alerts, poll time, and per-provider stats. This is the main unit-tested file (mock providers are defined in its test module).

**`src/settings.rs`** — Loads `Settings.toml` (or `APP_` prefixed env vars as fallback) into strongly-typed structs via `config` + `serde`. Settings are passed to the `Poller` at startup. Poll frequency is configurable via `poll_frequency` (default: 5s); see `src/entities/settings.rs`.

**`src/ui/`** — Dioxus components. `app.rs` is the root: it loads settings, creates the `Poller`, and provides it via Dioxus context. `AlertsApp` runs a coroutine that polls on the configured interval and stores results in a signal. Components consume that signal reactively. Tailwind CSS is used for all styling; severity-specific classes (`.severity-critical`, etc.) are defined in `assets/source.css`.

## Key Patterns

- **Adding a provider**: Implement `Provider` (async `alerts()` returning `Vec<Alert>`) and `ProviderAlert` (convert raw type → `Alert`). Register in the provider list built from `Settings` in `ui/app.rs`.
- **State management**: Dioxus signals. Avoid shared mutable state outside of signals.
- **Error handling**: `anyhow::Result` in providers; `thiserror`-derived enums for `ProviderError`. Fail with context rather than silently returning empty results.
- **Testing**: Tests live in `poller.rs` using in-module mock providers. No external services needed.

## Configuration

`Settings.toml` at the project root configures provider instances. Multiple Grafana instances are supported:

```toml
poll_frequency = 5  # seconds

[[grafana]]
url = "https://grafana.example.com"
token = "glsa_..."
```
