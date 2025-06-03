---
title: Installation
description: How to install the Roselite CLI on your system
---

This guide covers how to install the Roselite CLI on your system. The CLI is the primary tool for deploying and managing your decentralized sites.

## Prerequisites

Before installing Roselite, ensure you have:

- **Rust 1.70 or later** - [Install Rust](https://rustup.rs/)
- **Git** (for source installation)
- **Internet connection** for DHT communication

## Installation Methods

### Method 1: Build from Source (Current)

Currently, Roselite must be built from source:

```bash
# Clone the repository
git clone https://github.com/jdbohrman-tech/roselite.git
cd roselite

# Build and install the CLI
cargo install --path roselite-cli
```

### Method 2: Cargo Install (Future)

Once published to crates.io, you'll be able to install via:

```bash
cargo install roselite-cli
```

### Method 3: Pre-built Binaries (Future)

Pre-built binaries will be available from our [GitHub releases page](https://github.com/jdbohrman-tech/roselite/releases) in future releases.

## Verify Installation

Check that Roselite is installed correctly:

```bash
roselite --version
```

You should see output similar to:
```
roselite 0.1.0
```

## No Configuration Required

Roselite is designed to work out-of-the-box with no configuration required:

- **No initialization** - Just run commands directly
- **No config files** - Connects to Veilid DHT on-demand
- **No persistent state** - Each command is independent

The CLI will automatically:
- Connect to the Veilid DHT network when needed
- Use appropriate gateway servers for web access
- Handle all networking transparently

## Network Requirements

### Firewall Settings

Roselite needs to communicate with the Veilid DHT network. Ensure these ports are accessible:

- **Various TCP/UDP ports** - DHT communication (handled automatically by Veilid)
- **TCP 443** - Gateway communication (HTTPS)

### Behind Corporate Firewalls

Roselite includes fallback modes that should work in most corporate environments. If you experience connection issues, the CLI will attempt to continue with limited functionality.

## Testing Your Installation

Test your installation by checking the help:

```bash
# Show all available commands
roselite --help

# Show help for specific commands
roselite bundle --help
roselite publish --help
roselite access --help
```

You should see the three main commands: `bundle`, `publish`, and `access`.

## Updating

### From Source

```bash
cd roselite
git pull
cargo install --path roselite-cli --force
```

### Future: Cargo Install

Once available on crates.io:

```bash
cargo install roselite-cli --force
```

## Troubleshooting

### Permission Denied

If you get permission errors during installation:

```bash
# Install to user directory instead of system-wide
cargo install --path roselite-cli --root ~/.local

# Make sure ~/.local/bin is in your PATH
export PATH="$HOME/.local/bin:$PATH"
```

### Build Failures

If compilation fails:

1. **Update Rust**: `rustup update`
2. **Clear cargo cache**: `cargo clean`
3. **Check dependencies**: Ensure you have required system libraries
4. **Check GitHub issues**: See if others have reported similar problems

### Network Connection Issues

If Roselite can't connect to the DHT during publish:

1. Check your internet connection
2. Verify firewall settings
3. The CLI will show detailed error messages and attempt fallback modes
4. Connection issues during publish are reported but won't prevent the command from completing

### Platform-Specific Issues

#### Linux
- Install build dependencies: `sudo apt install build-essential pkg-config libssl-dev`

#### macOS
- Install Xcode command line tools: `xcode-select --install`

#### Windows
- Install Visual Studio Build Tools or Visual Studio Community
- Ensure the Rust toolchain is properly configured

## Next Steps

Now that you have Roselite installed, continue with the [Quick Start guide](./quick-start/) to deploy your first decentralized site! 