# Installation

Get Roselite up and running on your system with these installation options.

## Prerequisites

Before installing Roselite, ensure you have:

- **Rust** (1.70.0 or later) - [Install Rust](https://rustup.rs/)
- **Git** - For cloning repositories and version control
- **Operating System**: Linux, macOS, or Windows

!!! info "Veilid Dependency"
    Roselite uses the Veilid DHT network. The Veilid daemon will be automatically managed by Roselite, so no separate installation is required.

## Installation Methods

### Option 1: Install from Crates.io (Recommended)

The easiest way to install Roselite is using Cargo:

```bash
cargo install roselite
```

This will download, compile, and install the latest stable version of Roselite.

### Option 2: Install from Source

For the latest development version or to contribute:

```bash
# Clone the repository
git clone https://github.com/jdbohrman/roselite.git
cd roselite

# Build and install
cargo install --path .
```

### Option 3: Download Pre-built Binaries

Download pre-compiled binaries from the [GitHub Releases](https://github.com/jdbohrman/roselite/releases) page:

=== "Linux"
    ```bash
    # Download and extract (replace VERSION with actual version)
    wget https://github.com/jdbohrman/roselite/releases/download/vVERSION/roselite-linux-x86_64.tar.gz
    tar -xzf roselite-linux-x86_64.tar.gz
    
    # Move to PATH
    sudo mv roselite /usr/local/bin/
    ```

=== "macOS"
    ```bash
    # Download and extract
    curl -L https://github.com/jdbohrman/roselite/releases/download/vVERSION/roselite-macos-x86_64.tar.gz | tar -xz
    
    # Move to PATH
    sudo mv roselite /usr/local/bin/
    ```

=== "Windows"
    1. Download `roselite-windows-x86_64.zip`
    2. Extract the `roselite.exe` file
    3. Add the directory to your PATH

## Verify Installation

After installation, verify that Roselite is working correctly:

```bash
# Check version
roselite --version

# View help
roselite --help

# Test connectivity (this will start the Veilid daemon)
roselite status
```

Expected output:
```
Roselite v0.1.0
Veilid DHT: Connected
Gateway: Running on http://localhost:3000
Status: Ready
```

## Configuration

### Default Configuration

Roselite works out of the box with sensible defaults. The configuration file is located at:

- **Linux/macOS**: `~/.config/roselite/config.toml`
- **Windows**: `%APPDATA%\roselite\config.toml`

### Custom Configuration

To customize Roselite's behavior, create or edit the configuration file:

```toml
[network]
# Veilid DHT settings
bootstrap_nodes = ["bootstrap1.veilid.org", "bootstrap2.veilid.org"]
listen_port = 5150

[gateway]
# Gateway server settings
host = "localhost"
port = 3000
enable_cors = true

[storage]
# Local storage path for cached content
cache_dir = "~/.cache/roselite"
max_cache_size = "1GB"

[publishing]
# Default settings for publishing
compression = "gzip"
max_file_size = "10MB"
```

## Troubleshooting

### Common Issues

#### "command not found: roselite"

**Solution**: Make sure Cargo's bin directory is in your PATH:

```bash
# Add to your shell profile (.bashrc, .zshrc, etc.)
export PATH="$HOME/.cargo/bin:$PATH"

# Reload your shell
source ~/.bashrc  # or ~/.zshrc
```

#### "failed to connect to Veilid network"

**Solution**: Check your internet connection and firewall settings:

```bash
# Check if ports are blocked
telnet bootstrap1.veilid.org 5150

# Try with verbose logging
roselite -v status
```

#### "permission denied" on Linux/macOS

**Solution**: Ensure the binary has execute permissions:

```bash
chmod +x /usr/local/bin/roselite
```

### Getting Help

If you encounter issues:

1. Check the [troubleshooting guide](troubleshooting.md)
2. Search [existing issues](https://github.com/jdbohrman/roselite/issues)
3. Open a [new issue](https://github.com/jdbohrman/roselite/issues/new) with:
   - Your operating system
   - Roselite version (`roselite --version`)
   - Error messages
   - Steps to reproduce

## Next Steps

Now that you have Roselite installed, you're ready to:

1. Follow the [Quick Start Guide](quickstart.md) to publish your first site
2. Learn about [configuration options](configuration.md)
3. Explore the [CLI commands](cli.md)

!!! success "Installation Complete!"
    You're all set! Roselite is ready to help you publish sites to the decentralized web. 