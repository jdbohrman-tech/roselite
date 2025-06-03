---
title: Gateway API Reference
description: Gateway API and hosting setup (Coming Soon)
---

# Gateway API Reference - Coming Soon

The Roselite gateway infrastructure is currently in development. This page will contain the complete API reference once universal gateways are deployed.

## Current Status

**Universal Gateways**: Not yet deployed  
**Local Development**: Available  
**Custom Domain Setup**: Manual configuration required  

## What's Available Now

### Local Gateway Development

You can run a local gateway for development and testing:

```bash
# Clone and build the gateway
git clone https://github.com/jdbohrman-tech/roselite.git
cd roselite/roselite-gateway
cargo run

# Your local gateway will be available at:
# http://localhost:8080
```

### How the Gateway Actually Works

The Roselite gateway uses **subdomain-based routing** to serve DHT content:

1. **Publish your content** to DHT using the CLI
2. **Extract the app slug** from the DHT key
3. **Access via subdomain**: `http://your-app-slug.localhost:8080`

**No configuration files needed** - the gateway automatically:
- Connects to Veilid DHT on startup
- Resolves subdomains to DHT keys  
- Fetches and caches content from DHT
- Serves static files via HTTP

### Publishing to DHT

The core functionality works today:

```bash
# Bundle and publish your site
roselite bundle ./my-site --name "My Site"
roselite publish my-site.veilidpkg

# CLI will show you the app slug
# Access via: http://my-site.localhost:8080
```

## Local Development Setup

### 1. Start the Gateway

```bash
# Basic usage (HTTP only, port 8080)
cd roselite-gateway
cargo run

# With custom port
cargo run -- --port 3000

# With custom domain for production
cargo run -- --domain "yourdomain.com" --port 80

# With HTTPS (requires certificates)
cargo run -- --enable-https --cert-file cert.pem --key-file key.pem
```

**Available command line options:**
- `--port <PORT>` - HTTP port (default: 8080)
- `--https-port <PORT>` - HTTPS port (default: 8443)  
- `--domain <DOMAIN>` - Domain to serve (default: "localhost:8080")
- `--cert-file <FILE>` - Path to TLS certificate file
- `--key-file <FILE>` - Path to TLS private key file
- `--enable-https` - Enable HTTPS (requires cert and key files)
- `--cache-dir <DIR>` - Cache directory for apps (default: ".cache")

### 2. Access Your Apps

Once your gateway is running and you've published content:

```bash
# Your app is automatically available at:
http://your-app-slug.localhost:8080

# No DNS setup needed for *.localhost subdomains!
```

### 3. Manual Testing

For quick testing without DHT publishing:

```bash
# Create test app directory
mkdir -p .cache/my-test-app
echo '<h1>Hello World!</h1>' > .cache/my-test-app/index.html

# Access immediately:
# http://my-test-app.localhost:8080
```

## Production Setup with Custom Domain

For production use with your own domain:

### 1. Deploy Gateway to Your Server

```bash
# On your server (Ubuntu/Debian example)
git clone https://github.com/jdbohrman-tech/roselite.git
cd roselite/roselite-gateway

# Build for production
cargo build --release

# Run with your domain
./target/release/roselite-gateway --domain "example.com" --port 80

# Or with HTTPS
./target/release/roselite-gateway \
  --domain "example.com" \
  --port 80 \
  --enable-https \
  --cert-file /etc/letsencrypt/live/example.com/fullchain.pem \
  --key-file /etc/letsencrypt/live/example.com/privkey.pem
```

### 2. Configure DNS Wildcard

Set up wildcard DNS for subdomains:

```dns
# Wildcard A record for all subdomains
*.example.com. IN A 192.168.1.100

# Or wildcard CNAME
*.example.com. IN CNAME gateway-server.example.com.
```

### 3. Test Your Setup

```bash
# Publish an app
roselite bundle ./my-site --name "My Portfolio"
roselite publish my-portfolio.veilidpkg

# Access via your domain
curl http://my-portfolio.example.com
```

### 4. Set Up HTTPS with Let's Encrypt

```bash
# Install certbot
sudo apt install certbot

# Get wildcard certificate
sudo certbot certonly --manual -d "*.example.com" -d "example.com"

# Certificates will be in:
# /etc/letsencrypt/live/example.com/fullchain.pem
# /etc/letsencrypt/live/example.com/privkey.pem
```

## How DHT Resolution Works

The gateway uses a simple resolution process:

1. **Extract subdomain**: `my-app.localhost:8080` → `my-app`
2. **Convert to DHT key**: The subdomain IS the app slug/DHT key
3. **Fetch from DHT**: Use VeilidStore to download the package
4. **Cache locally**: Extract files to `.cache/my-app/`
5. **Serve files**: Standard HTTP file serving

**No DNS TXT records needed** - the subdomain directly maps to the DHT key.

## Gateway Server Features

### Automatic Caching
- Downloaded apps are cached locally
- Subsequent requests served instantly
- Cache invalidation when needed

### Security
- Path traversal protection
- Safe file serving only within app directories
- CORS headers for browser compatibility

### Development Features
- Automatic *.localhost subdomain support
- Hot reload when cache directories change
- Detailed logging and debugging

### Production Features
- HTTPS/TLS support with custom certificates
- Gzip compression for static assets
- Custom domain support

## Coming Soon: Universal Gateway Network

We're working on deploying a network of public gateways that will provide:

### Planned Features

- **Public Gateway URLs**: `https://gateway.roselite.network/{dht-key}`
- **Global Load Balancing**: Multiple gateway locations worldwide
- **Automatic Failover**: Redundant gateway infrastructure
- **HTTPS Support**: SSL certificates for all gateway domains
- **Custom Domain Support**: Automatic wildcard DNS resolution
- **API Endpoints**: Full REST API for gateway management

### Planned API Structure

The future API will include:

```bash
# Content access via subdomain
GET https://my-app.gateway.roselite.network/

# Direct DHT access
GET https://gateway.roselite.network/{dht-key}

# API endpoints
GET https://gateway.roselite.network/api/v1/status
GET https://gateway.roselite.network/api/v1/info/{dht-key}
GET https://gateway.roselite.network/api/v1/network/status
```

### Timeline

- **Phase 1** (Current): Local gateway development, DHT publishing
- **Phase 2** (Next): Single public gateway deployment with wildcard DNS
- **Phase 3** (Future): Multi-region gateway network with load balancing
- **Phase 4** (Future): Complete API and monitoring dashboard

## Current Workarounds

### For Development

```bash
# 1. Run local gateway
cd roselite-gateway && cargo run

# 2. Publish your site
roselite publish my-site.veilidpkg

# 3. Access via localhost subdomain
open http://my-site.localhost:8080
```

### For Production

```bash
# 1. Deploy gateway to your server with custom domain
# 2. Configure wildcard DNS (*.yourdomain.com)
# 3. Set up HTTPS with wildcard certificate
# 4. Share your gateway domain with users
```

## Troubleshooting

### Gateway Won't Start
```bash
# Check if port is already in use
lsof -i :8080

# Try different port
cargo run -- --port 3000
```

### App Not Loading
```bash
# Check if app exists in DHT
roselite access your-app-slug

# Check gateway logs for errors
RUST_LOG=debug cargo run

# Try manual cache
mkdir -p .cache/your-app-slug
# Copy files manually for testing
```

### DNS Issues
```bash
# Test wildcard DNS
dig my-app.example.com

# Test from different DNS servers
dig @8.8.8.8 my-app.example.com
```

## Stay Updated

- **GitHub**: Watch the [Roselite repository](https://github.com/jdbohrman-tech/roselite) for updates
- **Discussions**: Join our [GitHub Discussions](https://github.com/jdbohrman-tech/roselite/discussions)
- **Issues**: Report problems or request features in [Issues](https://github.com/jdbohrman-tech/roselite/issues)

---

**This page will be updated with the complete API reference once universal gateways are deployed. Check back soon!** 