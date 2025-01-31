[[Japanese/日本語](README.ja.md)]

# electron-repl

A command-line REPL tool for Electron applications that allows you to execute JavaScript code in the main process.

## Features

- Connect to running Electron applications
- Execute JavaScript code in the main process
- Support for macOS and Linux (Windows support coming soon)
- Command history support
- Colored output for better readability

## Installation

```bash
cargo install electron-repl
```

## Usage

```bash
electron-repl <app-name> [port]
```

### Arguments

- `app-name`: Name of the Electron application (required)
- `port`: Port number for DevTools (default: 9222)

### Example

```bash
electron-repl Discord
```

## Supported Platforms

- macOS
- Linux
- Windows (coming soon)

## License

MIT License - see [LICENSE](LICENSE) for details
