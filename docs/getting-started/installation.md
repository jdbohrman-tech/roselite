# Installation

This guide covers installing Roselite on your system. Roselite consists of two main components: the CLI tool for bundling and publishing, and the gateway server for web access.

## Prerequisites

Before installing Roselite, make sure you have:

=== "All Platforms"

    - **Rust 1.70+** with Cargo
    - **Git** for cloning the repository
    - **Internet connection** for Veilid network access

=== "Linux/macOS"

    ```bash
    # Check Rust version
    rustc --version
    # Should show 1.70.0 or higher
    
    # Install Rust if needed
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

=== "Windows"

    ```powershell
    # Check Rust version
    rustc --version
    
    # Install Rust if needed - visit https://rustup.rs/
    # Or use winget
    winget install Rustlang.Rustup
    ```

## Install from Source

The recommended way to install Roselite is building from source:

### 1. Clone the Repository

```bash
git clone https://github.com/jdbohrman/roselite.git
cd roselite
```

### 2. Build Release Version

```bash
# Build both CLI and gateway
cargo build --release

# This creates binaries in target/release/:
# - roselite (CLI tool)
# - roselite-gateway (Gateway server)
```

### 3. Verify Installation

```bash
# Test CLI
./target/release/roselite --version

# Test gateway
./target/release/roselite-gateway --help
```

### 4. Add to PATH (Optional)

=== "Linux/macOS"

    ```bash
    # Copy to local bin
    cp target/release/roselite ~/.local/bin/
    cp target/release/roselite-gateway ~/.local/bin/
    
    # Or create symlinks
    ln -s $(pwd)/target/release/roselite ~/.local/bin/roselite
    ln -s $(pwd)/target/release/roselite-gateway ~/.local/bin/roselite-gateway
    ```

=== "Windows"

    ```powershell
    # Copy to a directory in PATH
    copy target\release\roselite.exe C:\tools\
    copy target\release\roselite-gateway.exe C:\tools\
    
    # Add C:\tools to PATH if needed
    ```

## Development Installation

For development or contributing:

### 1. Clone and Setup

```bash
git clone https://github.com/jdbohrman/roselite.git
cd roselite

# Install in development mode
cargo build
```

### 2. Run Tests

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture
```

### 3. Development Commands

```bash
# Run CLI directly
cargo run --bin roselite -- --help

# Run gateway directly
cargo run --bin roselite-gateway -- --help

# Check code
cargo check
cargo clippy
```

## Docker Installation

For containerized deployment:

### 1. Build Docker Image

```bash
# Build gateway image
docker build -f roselite-gateway/Dockerfile -t roselite-gateway .
```

### 2. Run Gateway Container

```bash
# Run on port 8080
docker run -p 8080:8080 roselite-gateway --port 8080 --domain localhost:8080

# With volume for cache
docker run -p 8080:8080 -v ./cache:/app/cache roselite-gateway
```

## Verification

After installation, verify everything works:

### 1. Check CLI Installation

```bash
roselite --version
# Should output version information

roselite --help
# Should show available commands
```

### 2. Check Gateway Installation

```bash
roselite-gateway --help
# Should show gateway options
```

### 3. Test Basic Functionality

```bash
# Check Veilid network status
roselite status

# Should show network connectivity information
```

## Troubleshooting

### Common Issues

!!! warning "Build Errors"

    **Issue**: Compilation fails with dependency errors
    
    **Solution**: 
    ```bash
    # Update Rust toolchain
    rustup update
    
    # Clean and rebuild
    cargo clean
    cargo build --release
    ```

!!! warning "Network Issues"

    **Issue**: Can't connect to Veilid network
    
    **Solution**:
    ```bash
    # Check internet connection
    # Ensure firewall allows Veilid connections
    # Try with debug logging
    RUST_LOG=debug roselite status
    ```

!!! warning "Permission Errors"

    **Issue**: Permission denied when copying binaries
    
    **Solution**:
    ```bash
    # Use sudo if needed (Linux/macOS)
    sudo cp target/release/roselite /usr/local/bin/
    
    # Or install to user directory
    cp target/release/roselite ~/.local/bin/
    ```

### Getting Help

If you encounter issues:

1. Check the [troubleshooting guide](../reference/configuration.md#troubleshooting)
2. Search [existing issues](https://github.com/jdbohrman/roselite/issues)
3. Create a new issue with:
   - Your operating system
   - Rust version (`rustc --version`)
   - Complete error message
   - Steps to reproduce

## Next Steps

Now that Roselite is installed:

- [Quick Start Guide](quick-start.md) - Deploy your first site
- [First Deployment](first-deployment.md) - Complete tutorial with gateway
- [CLI Reference](../reference/cli-commands.md) - Detailed command documentation

!!! success "Installation Complete"

    You're ready to start deploying static sites to the Veilid DHT! Continue with the [Quick Start](quick-start.md) guide. 