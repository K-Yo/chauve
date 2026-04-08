# Codebase Overview: chauve Alert Notification System

## Project Structure

The `chauve` project is a clean architecture Rust application that serves as an alert notification system using Dioxus for the frontend. The project structure is organized as follows:

- **Root Directory**
  - `Cargo.toml`: Project dependencies and metadata
  - `Dioxus.toml`: Dioxus-specific configuration
  - `Settings.toml`: Configuration file for providers (currently only Grafana)
  - `README.md`: Project documentation
  - `assets/`: Static assets including CSS files

- **src/**: Main source code directory
  - `main.rs`: Entry point of the application
  - `entities/`: Data models and entity definitions
  - `providers/`: Integration with external alert providers
  - `ui/`: User interface components
  - `poller.rs`: Handles polling of alerts from providers
  - `settings.rs`: Configuration management

## Core Architecture

### 1. Main Application Flow

The application starts in `src/main.rs` which simply launches the Dioxus application component. The `App` component in `src/ui/app.rs` serves as the root component that initializes the application context and settings.

### 2. Settings System

The application uses a configuration system based on the `config` crate to load settings from `Settings.toml`. The `get_settings()` function in `src/settings.rs` reads and deserializes the configuration into a strongly-typed `Settings` structure.

Current configuration supports multiple Grafana instances defined in the `Settings.toml` file with URL and authentication token.

### 3. Provider Architecture

The system follows a plugin architecture for different alert providers through traits defined in `src/entities/provider.rs`:

- `Provider` trait: Defines the interface for alert providers with an async `alerts()` method
- `ProviderAlert` trait: Converts provider-specific alerts to the normalized `Alert` structure

Currently, only Grafana is implemented as a provider in `src/providers/grafana/`.

### 4. Grafana Integration

The Grafana provider (`src/providers/grafana/provider.rs`) connects to Grafana's Alertmanager API to fetch active alerts. It uses:

- `reqwest` for HTTP requests
- Bearer token authentication
- Async/Await for non-blocking API calls

The provider maps Grafana's alert format to the application's normalized `Alert` structure using the `convert_alert` function.

### 5. Data Models

#### Alert Structure

The core `Alert` structure (in `src/entities/alert.rs`) contains:
- Title
- Severity level
- Link to the source

#### Provider-Specific Models

Grafana alerts (`src/providers/grafana/alert.rs`) are deserialized from the API response and include:
- Annotations and labels
- Timestamps (start, update, end)
- Status information
- Receivers
- Fingerprint and generator URL

The `ProviderAlert` trait implementation extracts the relevant information (title, severity, link) from Grafana alerts to populate the normalized `Alert` structure.

## Frontend Implementation

The frontend is built with Dioxus, a React-like UI library for Rust. Key components:

- `App` component (`src/ui/app.rs`): Root component that provides global state
- `Header` component (`src/ui/header.rs`): Application header
- `AlertList` component (`src/ui/alert.rs`): Displays the list of alerts

The UI uses Tailwind CSS for styling and automatically refreshes alerts every 5 seconds using a coroutine. Users can also manually trigger updates with the "Update!" button.

## Polling System

The `Poller` struct (`src/poller.rs`) manages the retrieval of alerts from configured providers. It:

- Maintains a list of provider instances
- Tracks the last poll time
- Polls providers on demand or at regular intervals

## Current Limitations and Issues

1. **Error Handling**: The Grafana provider currently ignores API errors and returns an empty vector, which could hide connectivity issues or authentication problems.

2. **Single Provider Type**: Currently only supports Grafana, though the architecture is designed to support multiple provider types.

3. **Hardcoded Refresh Interval**: The UI refreshes every 5 seconds, which may be too frequent for some use cases.

## Future Development Opportunities

1. **Add More Providers**: Implement additional providers like Prometheus, Datadog, or custom webhook-based providers.

2. **Enhanced Error Handling**: Implement proper error handling and user notifications for provider connectivity issues.

3. **Configurable Refresh Rate**: Make the polling interval configurable through settings.

4. **Alert Filtering and Grouping**: Add UI capabilities to filter alerts by severity or provider.

6. **Improved Settings Management**: Add validation for provider configurations and better error messages for invalid settings.

The codebase demonstrates a well-structured clean architecture with clear separation of concerns between data models, providers, UI, and business logic, making it extensible for additional features and providers.