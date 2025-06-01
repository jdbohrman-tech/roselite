# Roselite - Decentralized App Store for Veilid

A decentralized application store built on the [Veilid](https://veilid.com) peer-to-peer network. Roselite enables developers to package, distribute, and monetize applications without relying on centralized app stores.

## 🚀 Features

- **📦 Package Format**: `.veilidpkg` format with cryptographic signing
- **🌐 Decentralized**: Uses Veilid DHT for storage and discovery
- **🔍 Smart Search**: Fuzzy search with real-time filtering
- **🔐 Secure**: Cryptographically signed packages with identity verification
- **⚡ Fast**: TUI interface with keyboard-first navigation
- **🎯 Developer Friendly**: Simple CLI tools for packaging and publishing

## 🏗️ Architecture

```
┌─────────────────┐    ┌──────────────────┐
│   roselite-cli  │    │  roselite-store  │
│   (CLI Tools)   │    │   (TUI Browser)  │
└─────────┬───────┘    └────────┬─────────┘
          │                     │
          └──────┬──────────────┘
                 │
         ┌───────▼────────┐
         │ roselite-core  │
         │ (Core Library) │
         └───────┬────────┘
                 │
         ┌───────▼────────┐
         │  veilid-core   │
         │ (DHT Network)  │
         └────────────────┘
```

## 📁 Project Structure

- **`crates/roselite-core/`** - Core library with package management and Veilid integration
- **`crates/roselite-cli/`** - Command-line tools for developers
- **`crates/roselite-store/`** - Terminal UI app store for browsing and installing apps

## 🛠️ Quick Start

### Prerequisites

- Rust 1.70+ with Cargo
- Veilid node (for network access)

### Installation

```bash
git clone https://github.com/yourusername/roselite
cd roselite
cargo build --release
```

### CLI Usage

```bash
# Bundle an app
roselite bundle --name "My App" --version "1.0.0" ./my-app-dir

# Publish to Veilid DHT
roselite publish my-app.veilidpkg

# Search for apps
roselite search "game" --tags "puzzle,arcade"

# Install an app
roselite install veil://app/123abc.../1.0.0

# Launch TUI store
roselite-store
```

### TUI Store Controls

- **`/`** - Enter search mode
- **`↑↓`** - Navigate app list
- **`Enter`** - Install selected app
- **`q`** - Quit

## 📦 Package Format

Roselite packages (`.veilidpkg`) are signed, compressed tarballs containing:

```json
{
  "name": "MyApp",
  "version": "1.0.0", 
  "description": "A cool app",
  "developer": "Developer Name",
  "entry": "index.html",
  "tags": ["game", "puzzle"],
  "identity": "veilid:abc123...",
  "signature": "base64(signature)",
  "format_version": "1.0.0"
}
```

## 🔧 Development Status

> **⚠️ Early Development**: This is scaffolding code. Core Veilid integration is not yet implemented.

### Completed ✅
- Project structure and workspace setup
- Core type definitions and error handling
- CLI command parsing and structure
- TUI app with search interface
- Package manifest format specification

### TODO 🚧
- [ ] Veilid API integration (`veilid-core` bindings)
- [ ] Actual package creation (tarball generation)
- [ ] DHT storage and retrieval implementation
- [ ] Cryptographic signing with Veilid keys
- [ ] QR code generation for easy sharing
- [ ] Package installation and runtime
- [ ] App sandboxing and permissions
- [ ] Version management and updates

## 🌐 Veilid Integration

The project is designed to integrate with Veilid's:

- **Table Store**: For app metadata and package storage
- **Crypto System**: For package signing and verification  
- **Routing Context**: For peer-to-peer app discovery
- **Identity System**: For developer authentication

## 🔍 Search & Discovery

- **Real-time Search**: Filter as you type
- **Fuzzy Matching**: Find apps even with typos
- **Multi-field Search**: Name, description, developer, tags
- **DHT Scanning**: Discover new apps across the network

## 📱 Example Usage

```bash
# Create a simple web app
mkdir my-web-app
echo '<h1>Hello Veilid!</h1>' > my-web-app/index.html

# Bundle it
roselite bundle --name "Hello World" my-web-app/

# Publish to network
roselite publish hello-world.veilidpkg
# Output: ✅ Published! veil://app/abc123.../1.0.0

# Others can now install it
roselite install veil://app/abc123.../1.0.0
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📄 License

This project is licensed under MIT OR Apache-2.0.

## 🔗 Links

- [Veilid Project](https://veilid.com)
- [Veilid Developer Book](https://veilid.gitlab.io/developer-book/)
- [Issue Tracker](https://github.com/yourusername/roselite/issues) 