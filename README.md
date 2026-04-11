# chauve

An alert notification system for monitoring and visualizing alerts from various monitoring platforms. Built with Rust and Dioxus, chauve provides a clean, responsive interface for aggregating and displaying alerts from multiple sources.

## Quickstart

1. Install Rust (1.70+ recommended)
2. Clone this repository
3. Install [Dioxus CLI](https://dioxuslabs.com/docs/0.4/cli/index.html):
   ```bash
   cargo install dioxus-cli
   ```
4. Start the development app:
   ```bash
   dx serve --desktop
   ```
5. in another shell, sync tailwind css
   ```bash
   npx @tailwindcss/cli -i ./assets/source.css -o ./assets/main.css --watch
   ```


## Configuration

Configure your alert providers in `Settings.toml`. Currently, Grafana is supported as a provider:

```toml
[grafana]
url = "https://your-grafana-instance.com"
token = "your-api-token"
```

You can configure multiple Grafana instances by adding multiple entries.

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