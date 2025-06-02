# CLI Commands

Complete reference for all Roselite command-line tools.

## roselite

The main CLI tool for bundling and publishing static sites.

### Global Options

| Option | Description | Default |
|--------|-------------|---------|
| `-h, --help` | Show help message | |
| `-V, --version` | Show version information | |
| `-v, --verbose` | Enable verbose logging | |
| `--quiet` | Suppress output except errors | |

### bundle

Package static site files into a Veilid package.

```bash
roselite bundle <PATH> --name <NAME> --version <VERSION> [OPTIONS]
```

#### Arguments

- `<PATH>` - Directory containing static site files

#### Required Options

- `--name <NAME>` - Package name
- `--version <VERSION>` - Package version (semver format)

#### Optional Arguments

| Option | Description | Default |
|--------|-------------|---------|
| `--description <DESC>` | Package description | |
| `--developer <DEV>` | Developer name | |
| `--entry <FILE>` | Entry point file | `index.html` |
| `--category <CAT>` | Package category | `website` |
| `--output <FILE>` | Output package file | `<name>.veilidpkg` |

#### Examples

```bash
# Basic bundling
roselite bundle ./my-site --name "My Portfolio" --version "1.0.0"

# With metadata
roselite bundle ./docs \
  --name "Project Docs" \
  --version "2.1.0" \
  --description "API documentation" \
  --developer "My Company" \
  --entry "index.html"

# Custom output location
roselite bundle ./site --name "Blog" --version "1.0.0" --output ./packages/blog.veilidpkg
```

### publish

Deploy a Veilid package to the DHT.

```bash
roselite publish <PACKAGE> [OPTIONS]
```

#### Arguments

- `<PACKAGE>` - Path to `.veilidpkg` file

#### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--slug <SLUG>` | Custom slug for access | Auto-generated |
| `--force` | Overwrite existing content | `false` |
| `--timeout <SECS>` | Network timeout in seconds | `30` |

#### Examples

```bash
# Basic publishing
roselite publish my-site.veilidpkg

# Custom slug
roselite publish docs.veilidpkg --slug project-documentation

# Force overwrite
roselite publish blog.veilidpkg --force
```

### status

Check Veilid network connectivity and system status.

```bash
roselite status [OPTIONS]
```

#### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--json` | Output in JSON format | `false` |
| `--detailed` | Show detailed network information | `false` |

#### Examples

```bash
# Basic status
roselite status

# Detailed information
roselite status --detailed

# JSON output
roselite status --json
```

### list

List published packages.

```bash
roselite list [OPTIONS]
```

#### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--format <FORMAT>` | Output format (table, json, yaml) | `table` |
| `--filter <PATTERN>` | Filter by package name pattern | |

#### Examples

```bash
# List all packages
roselite list

# JSON format
roselite list --format json

# Filter by name
roselite list --filter "docs*"
```

### access

Retrieve content from DHT (for testing).

```bash
roselite access <SLUG> [OPTIONS]
```

#### Arguments

- `<SLUG>` - Package slug to retrieve

#### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--output <DIR>` | Extract to directory | |
| `--raw` | Show raw DHT data | `false` |
| `--file <PATH>` | Retrieve specific file | |

#### Examples

```bash
# Access package
roselite access my-site

# Extract to directory
roselite access docs --output ./extracted

# Show raw data
roselite access blog --raw

# Get specific file
roselite access my-site --file style.css
```

## roselite-gateway

The HTTP gateway server for web access.

### Usage

```bash
roselite-gateway [OPTIONS]
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `-p, --port <PORT>` | HTTP port to listen on | `8080` |
| `-d, --domain <DOMAIN>` | Domain for subdomain routing | `localhost:8080` |
| `--cache-dir <DIR>` | Cache directory path | `./cache` |
| `--cache-size <SIZE>` | Max cache size in MB | `1024` |
| `--timeout <SECS>` | DHT request timeout | `30` |
| `--workers <NUM>` | Number of worker threads | CPU cores |
| `--log-level <LEVEL>` | Logging level | `info` |
| `-h, --help` | Show help message | |
| `-V, --version` | Show version | |

### Examples

```bash
# Basic gateway
roselite-gateway

# Custom port and domain
roselite-gateway --port 3000 --domain localhost:3000

# Production setup
roselite-gateway \
  --port 443 \
  --domain roselite.example.com \
  --cache-size 4096 \
  --workers 8

# Development with debug logging
RUST_LOG=debug roselite-gateway --port 8080 --domain localhost:8080
```

### URL Format

Gateway serves content via subdomain routing:

```
http://[slug].[domain]/[path]
```

Examples:
- `http://my-site.localhost:8080/` → serves `index.html`
- `http://docs.localhost:8080/api/` → serves `api/index.html`
- `http://blog.localhost:8080/posts/hello.html` → serves `posts/hello.html`

## Environment Variables

Configure Roselite behavior via environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Logging level | `info` |
| `ROSELITE_FALLBACK` | Enable fallback mode | `false` |
| `ROSELITE_CONFIG` | Config file path | |
| `VEILID_CONFIG` | Veilid config path | |
| `ROSELITE_CACHE_DIR` | Global cache directory | `~/.roselite/cache` |

### Examples

```bash
# Debug logging
RUST_LOG=debug roselite publish site.veilidpkg

# Fallback mode (local storage)
ROSELITE_FALLBACK=true roselite-gateway

# Custom cache directory
ROSELITE_CACHE_DIR=/tmp/roselite-cache roselite-gateway
```

## Exit Codes

Roselite commands return standard exit codes:

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General error |
| `2` | Network error (DHT unreachable) |
| `3` | File system error |
| `4` | Invalid arguments |
| `5` | Package format error |

## Configuration Files

### Global Config

Location: `~/.roselite/config.toml`

```toml
[network]
timeout = 30
fallback = false

[cache]
directory = "~/.roselite/cache"
max_size_mb = 1024

[gateway]
default_port = 8080
default_domain = "localhost:8080"
```

### Project Config

Location: `.roselite.toml` in project root

```toml
[package]
name = "My Site"
version = "1.0.0"
description = "My personal website"
entry = "index.html"

[build]
exclude = ["*.md", "src/", ".git/"]
include = ["dist/"]
```

## Common Workflows

### Deploy Static Site

```bash
# 1. Build your static site (example with Next.js)
npm run build

# 2. Bundle for Roselite
roselite bundle ./out --name "My App" --version "1.0.0"

# 3. Publish to DHT
roselite publish my-app.veilidpkg

# 4. Start gateway (in another terminal)
roselite-gateway --port 3000 --domain localhost:3000

# 5. Access in browser
open http://my-app.localhost:3000
```

### Update Existing Site

```bash
# Make changes to your site
# ...

# Increment version and re-bundle
roselite bundle ./site --name "My Site" --version "1.0.1"

# Re-publish (will update existing content)
roselite publish my-site.veilidpkg
```

### Production Deployment

```bash
# Build optimized gateway
cargo build --release

# Deploy with production settings
./target/release/roselite-gateway \
  --port 443 \
  --domain your-domain.com \
  --cache-size 8192 \
  --workers 16 \
  --log-level warn
```

## Troubleshooting

### Common Issues

#### "Network unreachable"
```bash
# Check network status
roselite status

# Try with debug logging
RUST_LOG=debug roselite publish site.veilidpkg

# Use fallback mode
ROSELITE_FALLBACK=true roselite publish site.veilidpkg
```

#### "Package not found"
```bash
# List published packages
roselite list

# Check slug spelling
roselite access correct-slug-name

# Verify publication succeeded
roselite status --detailed
```

#### "Gateway not responding"
```bash
# Check if gateway is running
ps aux | grep roselite-gateway

# Test with curl
curl -H "Host: my-site.localhost:3000" http://localhost:3000/

# Check gateway logs for errors
```

#### "Permission denied"
```bash
# Check file permissions
ls -la site.veilidpkg

# Ensure cache directory is writable
mkdir -p ~/.roselite/cache
chmod 755 ~/.roselite/cache
```

### Debug Mode

Enable detailed logging for troubleshooting:

```bash
# CLI debug
RUST_LOG=debug roselite bundle ./site --name "Test" --version "1.0.0"

# Gateway debug
RUST_LOG=debug roselite-gateway --port 3000 --domain localhost:3000

# Trace level (very verbose)
RUST_LOG=trace roselite publish site.veilidpkg
``` 