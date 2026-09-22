# chauve

[![Build Desktop Release](https://github.com/K-Yo/chauve/actions/workflows/release.yml/badge.svg)](https://github.com/K-Yo/chauve/actions/workflows/release.yml)

An alert notification system for monitoring and visualizing alerts from various monitoring platforms. Built with Rust and Dioxus, chauve provides a clean, responsive interface for aggregating and displaying alerts from multiple sources.

## Install

Grab the latest assets from the [Releases page](https://github.com/K-Yo/chauve/releases).

### macOS (Apple Silicon)

1. Download `chauve-macos-arm64-vX.Y.Z.dmg`.
2. Open it and drag **Chauve** into the Applications folder.
3. The app is ad-hoc signed but not notarized — Apple charges $99/yr for that — so
   Gatekeeper blocks the first launch with *"Apple could not verify Chauve is free of
   malware"*. Clear the download quarantine flag once:
   ```bash
   xattr -dr com.apple.quarantine /Applications/Chauve.app
   ```
   Alternatively, try to open the app, then go to **System Settings → Privacy &
   Security** and click **Open Anyway**. (On macOS 15 and later, right-click → Open no
   longer bypasses this.)
4. Launch Chauve, then configure your providers — see [Configuration](#configuration).

### Linux / Windows

The release assets are plain executables (`chauve-linux-amd64-vX.Y.Z` and
`chauve-windows-amd64-vX.Y.Z.exe`); download, make executable if needed, and run.

## Quickstart

1. Install Rust (1.70+ recommended)
2. Clone this repository
3. Install the [Dioxus CLI](https://dioxuslabs.com/docs/0.4/cli/index.html). `dx` only works with
   the exact `dioxus` version this project pins, so use the script — it reads that version and
   installs the matching `dx`:
   ```bash
   ./scripts/install-dx.sh
   ```
   Re-run it whenever the `dioxus` dependency is bumped.
4. Start the development app:
   ```bash
   dx serve --desktop
   ```
5. in another shell, sync tailwind css
   ```bash
   npx @tailwindcss/cli -i ./assets/source.css -o ./assets/main.css --watch
   ```


## Configuration

Chauve reads `Settings.toml` from the OS config directory, creating it with defaults on
first launch:

| OS      | Path                                              |
| ------- | ------------------------------------------------- |
| macOS   | `~/Library/Application Support/chauve/Settings.toml` |
| Linux   | `~/.config/chauve/Settings.toml`                   |
| Windows | `%APPDATA%\chauve\Settings.toml`                   |

Currently, Grafana is supported as a provider:

```toml
poll_frequency = 5 # seconds

[notifications]
enabled = true
min_severity = "high" # critical, high, medium or low

[[providers.grafana]]
url = "https://your-grafana-instance.com"
token = "your-api-token"
```

You can configure multiple Grafana instances by adding multiple
`[[providers.grafana]]` entries. Every key can also be set through `APP_`-prefixed
environment variables.

## Code Organization

The project follows a clean architecture pattern with clear separation of concerns:

```
src/
├── entities/     # Data models and core entities
├── providers/    # External service integrations
├── ui/           # User interface components
├── poller.rs     # Alert polling logic
└── settings.rs   # Configuration management
```

The architecture is extensible to support multiple alert providers through a plugin system. Currently, Grafana integration is implemented, but the design allows for easy addition of other providers like Prometheus, Datadog, etc.

## Features

- Real-time alert monitoring with 5-second auto-refresh
- Manual refresh capability via "Update!" button
- Clean, responsive UI built with Dioxus and Tailwind CSS
- Support for multiple Grafana instances
- Extensible provider architecture

## Future Roadmap

- Add support for additional providers (Prometheus, Datadog, etc.)
- Implement configurable refresh intervals
- Add alert filtering and grouping by severity
- Enhanced error handling and notifications
- Improved settings validation

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -am 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a pull request

## License

Distributed under the MIT License. See `LICENSE` for more information.