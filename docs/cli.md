# CLI Reference

Complete reference for all Roselite command-line interface commands.

## Global Options

These options can be used with any Roselite command:

```bash
roselite [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-h, --help` | Show help information |
| `-V, --version` | Show version information |
| `-v, --verbose` | Enable verbose output |
| `-q, --quiet` | Suppress non-error output |
| `--config <FILE>` | Use custom configuration file |

## Commands Overview

| Command | Purpose |
|---------|---------|
| [`bundle`](#bundle) | Create a site bundle |
| [`publish`](#publish) | Publish a bundle to DHT |
| [`gateway`](#gateway) | Manage gateway server |
| [`status`](#status) | Check system status |
| [`list`](#list) | List published sites |
| [`info`](#info) | Get site information |
| [`update`](#update) | Update an existing site |
| [`delete`](#delete) | Delete a site from DHT |
| [`open`](#open) | Open a site in browser |
| [`serve`](#serve) | Serve local files |
| [`config`](#config) | Manage configuration |

---

## `bundle`

Create a compressed bundle from a directory of static files.

### Usage
```bash
roselite bundle [OPTIONS] <SOURCE>
```

### Arguments
- `<SOURCE>` - Directory containing static files to bundle

### Options
| Option | Description | Default |
|--------|-------------|---------|
| `-o, --output <FILE>` | Output bundle filename | `site.tar.gz` |
| `--compression <TYPE>` | Compression algorithm | `gzip` |
| `--exclude <PATTERN>` | Exclude files matching pattern | - |
| `--include-hidden` | Include hidden files | `false` |
| `--max-size <SIZE>` | Maximum total bundle size | `100MB` |

### Examples
```bash
# Bundle current directory
roselite bundle .

# Bundle with custom output name
roselite bundle ./public -o my-site.tar.gz

# Bundle with exclusions
roselite bundle . --exclude "*.log" --exclude "node_modules/*"

# Bundle with different compression
roselite bundle . --compression zstd
```

### Supported Compression Types
- `gzip` (default) - Good balance of speed and compression
- `zstd` - Faster compression/decompression
- `bzip2` - Better compression ratio
- `none` - No compression

---

## `publish`

Publish a site bundle to the Veilid DHT network.

### Usage
```bash
roselite publish [OPTIONS] <BUNDLE>
```

### Arguments
- `<BUNDLE>` - Path to the bundle file to publish

### Options
| Option | Description | Default |
|--------|-------------|---------|
| `--name <NAME>` | Human-readable site name | - |
| `--description <DESC>` | Site description | - |
| `--tags <TAGS>` | Comma-separated tags | - |
| `--private` | Publish as private site | `false` |
| `--ttl <SECONDS>` | Time-to-live in seconds | `86400` |
| `--replicas <NUM>` | Number of replicas | `3` |

### Examples
```bash
# Basic publish
roselite publish site.tar.gz

# Publish with metadata
roselite publish site.tar.gz \
  --name "My Personal Blog" \
  --description "A blog about tech and life" \
  --tags "blog,personal,tech"

# Publish private site
roselite publish site.tar.gz --private

# Publish with custom TTL
roselite publish site.tar.gz --ttl 604800  # 7 days
```

### Output
```
🚀 Publishing site to Veilid DHT...
📡 Uploading to network... ✅
✅ Site published successfully!

📍 DHT Address: VLD0:abc123def456789...
🌐 Gateway URL: http://localhost:3000/site/abc123def456789
🔗 Share URL: https://gateway.roselite.org/site/abc123def456789
📊 Size: 1.2 KB compressed (2.1 KB original)
🕐 TTL: 24 hours
```

---

## `gateway`

Manage the Roselite gateway server.

### Usage
```bash
roselite gateway <SUBCOMMAND>
```

### Subcommands

#### `start`
Start the gateway server.

```bash
roselite gateway start [OPTIONS]
```

| Option | Description | Default |
|--------|-------------|---------|
| `--host <HOST>` | Bind to host address | `localhost` |
| `--port <PORT>` | Listen on port | `3000` |
| `--cors` | Enable CORS headers | `false` |
| `--daemon` | Run as background daemon | `false` |

#### `stop`
Stop the gateway server.

```bash
roselite gateway stop
```

#### `status`
Check gateway server status.

```bash
roselite gateway status
```

#### `restart`
Restart the gateway server.

```bash
roselite gateway restart [OPTIONS]
```

### Examples
```bash
# Start gateway on default port
roselite gateway start

# Start on custom host/port
roselite gateway start --host 0.0.0.0 --port 8080

# Start with CORS enabled
roselite gateway start --cors

# Start as daemon
roselite gateway start --daemon

# Check status
roselite gateway status

# Stop gateway
roselite gateway stop
```

---

## `status`

Check the status of Roselite components or specific sites.

### Usage
```bash
roselite status [SITE_ID]
```

### Arguments
- `[SITE_ID]` - Optional site ID to check specific site status

### Examples
```bash
# Check overall system status
roselite status

# Check specific site status
roselite status abc123def456789
```

### Output (System Status)
```
🌹 Roselite Status
✅ Veilid DHT: Connected (5 peers)
✅ Gateway: Running on http://localhost:3000
✅ Local Cache: 45.2 MB used (500 MB available)
📊 Published Sites: 3 active
🕐 Uptime: 2h 15m
```

### Output (Site Status)
```
🌐 Site Status: abc123def456789
✅ Available on DHT
📊 Size: 1.2 KB
🕐 Published: 2 hours ago
🔄 Last accessed: 5 minutes ago
📍 Replicas: 3/3 healthy
```

---

## `list`

List all published sites.

### Usage
```bash
roselite list [OPTIONS]
```

### Options
| Option | Description | Default |
|--------|-------------|---------|
| `--format <FORMAT>` | Output format | `table` |
| `--sort <FIELD>` | Sort by field | `published` |
| `--reverse` | Reverse sort order | `false` |
| `--filter <PATTERN>` | Filter by name pattern | - |

### Output Formats
- `table` - Human-readable table
- `json` - JSON output
- `csv` - CSV format

### Examples
```bash
# List all sites
roselite list

# List with JSON output
roselite list --format json

# Sort by name
roselite list --sort name

# Filter by pattern
roselite list --filter "blog*"
```

---

## `info`

Get detailed information about a specific site.

### Usage
```bash
roselite info <SITE_ID>
```

### Arguments
- `<SITE_ID>` - Site ID to get information about

### Example
```bash
roselite info abc123def456789
```

### Output
```
🌐 Site Information

📍 ID: abc123def456789
📛 Name: My Personal Blog
📝 Description: A blog about tech and life
🏷️  Tags: blog, personal, tech
📊 Size: 1.2 KB compressed (2.1 KB original)
🕐 Published: 2023-12-01 14:30:00 UTC
🔄 Last Modified: 2023-12-01 14:30:00 UTC
⏱️  TTL: 23h 45m remaining
🔗 Gateway URL: http://localhost:3000/site/abc123def456789
📡 DHT Address: VLD0:abc123def456789...

📊 Network Status:
  ✅ Available: Yes
  🔄 Replicas: 3/3 healthy
  📈 Access Count: 42 (last 24h)
  🌍 Geographic Distribution: 3 regions

📁 Contents:
  index.html (1.1 KB)
  about.html (0.8 KB)
  style.css (0.3 KB)
```

---

## `update`

Update an existing site with new content.

### Usage
```bash
roselite update <SITE_ID> <BUNDLE>
```

### Arguments
- `<SITE_ID>` - ID of the site to update
- `<BUNDLE>` - Path to the new bundle file

### Options
| Option | Description |
|--------|-------------|
| `--name <NAME>` | Update site name |
| `--description <DESC>` | Update description |
| `--tags <TAGS>` | Update tags |

### Example
```bash
# Update with new bundle
roselite update abc123def456789 new-site.tar.gz

# Update with new metadata
roselite update abc123def456789 new-site.tar.gz \
  --name "Updated Blog" \
  --description "New and improved!"
```

---

## `delete`

Delete a site from the DHT network.

### Usage
```bash
roselite delete <SITE_ID>
```

### Arguments
- `<SITE_ID>` - ID of the site to delete

### Options
| Option | Description |
|--------|-------------|
| `--force` | Skip confirmation prompt |

### Example
```bash
# Delete with confirmation
roselite delete abc123def456789

# Force delete without confirmation
roselite delete abc123def456789 --force
```

---

## `open`

Open a site in the default web browser.

### Usage
```bash
roselite open <SITE_ID>
```

### Arguments
- `<SITE_ID>` - ID of the site to open

### Example
```bash
roselite open abc123def456789
```

---

## `serve`

Serve local files through the gateway (for development).

### Usage
```bash
roselite serve [OPTIONS] <DIRECTORY>
```

### Arguments
- `<DIRECTORY>` - Directory to serve

### Options
| Option | Description | Default |
|--------|-------------|---------|
| `--port <PORT>` | Port to serve on | `3001` |
| `--host <HOST>` | Host to bind to | `localhost` |
| `--watch` | Auto-reload on changes | `false` |

### Example
```bash
# Serve current directory
roselite serve .

# Serve with auto-reload
roselite serve . --watch

# Serve on custom port
roselite serve ./public --port 8080
```

---

## `config`

Manage Roselite configuration.

### Usage
```bash
roselite config <SUBCOMMAND>
```

### Subcommands

#### `show`
Show current configuration.

```bash
roselite config show
```

#### `set`
Set a configuration value.

```bash
roselite config set <KEY> <VALUE>
```

#### `get`
Get a configuration value.

```bash
roselite config get <KEY>
```

#### `reset`
Reset configuration to defaults.

```bash
roselite config reset
```

### Examples
```bash
# Show all config
roselite config show

# Set gateway port
roselite config set gateway.port 8080

# Get a specific value
roselite config get network.bootstrap_nodes

# Reset to defaults
roselite config reset
```

---

## Exit Codes

Roselite uses standard exit codes:

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General error |
| `2` | Invalid command line arguments |
| `3` | Network error (DHT connection failed) |
| `4` | File system error |
| `5` | Permission denied |
| `6` | Site not found |

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `ROSELITE_CONFIG` | Path to config file | `~/.config/roselite/config.toml` |
| `ROSELITE_CACHE_DIR` | Cache directory | `~/.cache/roselite` |
| `ROSELITE_LOG_LEVEL` | Log level | `info` |
| `VEILID_BOOTSTRAP` | Bootstrap nodes | Built-in list |

### Example
```bash
export ROSELITE_LOG_LEVEL=debug
roselite status
``` 