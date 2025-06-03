---
title: CLI Commands
description: Complete reference for all Roselite CLI commands
---

This page provides a comprehensive reference for all Roselite CLI commands, their options, and usage examples.

## Commands Overview

| Command | Description |
|---------|-------------|
| [`bundle`](#bundle) | Bundle a static site into a .veilidpkg package |
| [`publish`](#publish) | Publish a package to the Veilid DHT for P2P hosting |
| [`access`](#access) | Access a site directly from a DHT key or gateway URL |

---

## bundle

Bundle a static site into a .veilidpkg package file for publishing to the DHT.

### Usage
```bash
roselite bundle [OPTIONS] [DIR]
```

### Arguments
| Argument | Description |
|----------|-------------|
| `[DIR]` | Source directory containing the static site (defaults to current directory) |

### Options
| Option | Description |
|--------|-------------|
| `-o, --output <FILE>` | Output package file path |
| `--name <NAME>` | Site name (will prompt if not provided) |
| `--version <VERSION>` | Site version (defaults to "1.0.0") |
| `--description <DESC>` | Site description |
| `--developer <DEVELOPER>` | Developer/author name |
| `--entry <ENTRY>` | Entry point file (defaults to "index.html") |
| `--tags <TAGS>` | Tags (comma-separated) |

### Examples

```bash
# Bundle current directory with interactive prompts
roselite bundle

# Bundle specific directory with metadata
roselite bundle ./my-site --name "My Portfolio" --description "Personal website"

# Bundle with custom output file
roselite bundle ./dist --output my-custom-package.veilidpkg

# Bundle with all metadata specified
roselite bundle ./site \
  --name "Documentation" \
  --version "2.1.0" \
  --description "Project documentation site" \
  --developer "My Organization" \
  --entry "index.html" \
  --tags "docs,api,reference"
```

### Output
```
🏗️  Bundling site from: /path/to/site
📦 Package: My Portfolio v1.0.0 by Developer Name
✅ Package created: my-portfolio.veilidpkg
📦 Size: 2,345 bytes
```

---

## publish

Publish a .veilidpkg package to the Veilid DHT for decentralized hosting.

### Usage
```bash
roselite publish [OPTIONS] <PACKAGE>
```

### Arguments
| Argument | Description |
|----------|-------------|
| `<PACKAGE>` | Package file (.veilidpkg) to publish |

### Options
| Option | Description |
|--------|-------------|
| `-g, --gateways` | Show all available gateway URLs |
| `--open` | Open the site in browser after publishing |

### Examples

```bash
# Publish a package
roselite publish my-site.veilidpkg

# Publish and show all gateways
roselite publish my-site.veilidpkg --gateways

# Publish and auto-open in browser
roselite publish my-site.veilidpkg --open

# Publish with both options
roselite publish my-site.veilidpkg --gateways --open
```

### Output
```
📤 Publishing package: my-site.veilidpkg
📦 Package: My Site v1.0.0 by Developer
🌐 Connecting to Veilid DHT...
✅ Successfully connected to Veilid network!
📡 Publishing to Veilid DHT...
✅ Package published successfully!

📊 DHT Record Details:
   📋 App ID: abc123def456789...
   📈 Version: 1.0.0
   🔗 DHT Record Key: abc123def456789...
   📡 Storage: Veilid distributed hash table

🚀 INSTANT WEB ACCESS:
   🌐 Primary URL: https://gateway.roselite.network/abc123def456789
   📱 Mobile-friendly HTTPS
   🔄 Real-time DHT resolution
   ✅ No setup required!

🎉 Your site is now live on the decentralized web!
```

With `--gateways` flag:
```
🌍 ALL AVAILABLE GATEWAYS:
   🔗 Primary Gateway: https://gateway.roselite.network/abc123def456789
   🔗 Backup Gateway 1: https://gateway2.roselite.network/abc123def456789
   🔗 Backup Gateway 2: https://gateway3.roselite.network/abc123def456789
```

### Custom Domain Setup
The publish command also provides instructions for setting up custom domains:

```
🔧 ADVANCED: Custom Domain Setup (Optional)
For your own domain (like example.com):
   1. Add DNS TXT record:
      example.com. IN TXT "veilid-app=abc123def456789"
      example.com. IN TXT "veilid-version=1.0.0"
   2. Deploy gateway code or use CNAME:
      example.com. CNAME gateway.roselite.network
   3. Access via: https://example.com
```

---

## access

Access a site directly from a DHT key or gateway URL. This command fetches site information from the Veilid DHT.

### Usage
```bash
roselite access <KEY_OR_URL>
```

### Arguments
| Argument | Description |
|----------|-------------|
| `<KEY_OR_URL>` | DHT key (App ID) or gateway URL of the site to access |

### Examples

```bash
# Access by DHT key
roselite access abc123def456789

# Access by gateway URL
roselite access https://gateway.roselite.network/abc123def456789

# Access by custom domain (extracts DHT key from DNS)
roselite access https://example.com
```

### Output
```
🌐 Accessing site: abc123def456789
🔍 DHT Lookup Key: abc123def456789
📡 Connecting to Veilid DHT...
✅ Found site in DHT!

📊 Site Information:
   📦 Name: My Portfolio
   📈 Version: 1.0.0
   👤 Developer: Developer Name
   📝 Description: Personal portfolio website
   🏷️ Tags: portfolio, web, personal
   📄 Entry Point: index.html
   📦 Package Size: 2.3 MB
   🕒 Last Updated: 2024-01-15 14:30:22

🌐 Gateway Access URLs:
   • https://gateway.roselite.network/abc123def456789
   • https://gateway2.roselite.network/abc123def456789
```

For URL inputs, the command will attempt to extract the DHT key from DNS TXT records or URL paths.

---

## Global Help

To see general help information:

```bash
# Show main help
roselite --help

# Show command-specific help
roselite bundle --help
roselite publish --help
roselite access --help

# Show version
roselite --version
```

## Error Handling

All commands include comprehensive error handling:

### Bundle Errors
- Missing or invalid source directory
- No valid web files found
- File permission issues
- Invalid metadata

### Publish Errors
- Invalid package file
- Veilid DHT connection failures
- Network connectivity issues
- Package format version mismatches

### Access Errors
- Invalid DHT key format
- Site not found in DHT
- Veilid connection issues
- DNS lookup failures (for URL inputs)

## Configuration

Roselite stores temporary data and cache in:
- **Linux/macOS**: `~/.cache/roselite/`
- **Windows**: `%LOCALAPPDATA%\roselite\cache\`

No persistent configuration file is currently required, as the CLI connects to the DHT on-demand. 