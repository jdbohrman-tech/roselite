# Roselite - P2P Static Site Hosting via Veilid DHT

Deploy static content like JAMStack sites to the [Veilid](https://veilid.com) DHT with instant web access through gateway servers. No traditional hosting, no SSL setup required - just publish and share.

## 🚀 Features

- **📦 Simple Packaging**: Bundle static sites into `.veilidpkg` format
- **🌐 Instant Deployment**: Publish to Veilid DHT with one command
- **🌍 Gateway Server**: Full HTTP server for web browser access via subdomains
- **🔗 Universal Access**: Automatic gateway URLs with SSL termination
- **🔐 Decentralized**: Content stored in Veilid DHT, served through gateways
- **🛡️ Fallback System**: Works even when Veilid network has connectivity issues
- **⚡ High Performance**: In-memory and filesystem caching for fast loading
- **🎯 Developer Friendly**: Four commands: bundle, publish, gateway, access

## 🏗️ How It Works

```
Static Site → Bundle → Veilid DHT → Gateway Server → Web Browser
(HTML/CSS/JS) (.veilidpkg) (Distributed) (subdomain.domain.com) (HTTPS)
```

1. **Bundle** your static site files into a package
2. **Publish** the package to Veilid's distributed hash table  
3. **Gateway** serves content via subdomain routing (e.g., `my-site.veilid.app`)
4. **Share** the gateway URL - fully accessible in any web browser

## 📁 Project Structure

- **`roselite-core/`** - Core library with enhanced Veilid DHT integration
- **`roselite-cli/`** - Command-line tool for P2P hosting
- **`roselite-gateway/`** - **NEW**: HTTP gateway server for web access

## 🛠️ Quick Start

### Prerequisites

- Rust 1.70+ with Cargo
- Internet connection (Veilid network bootstrap)

### Installation

```bash
git clone https://github.com/yourusername/roselite
cd roselite
cargo build --release
```

### Complete Workflow

```bash
# 1. Bundle your static site
./target/release/roselite bundle bundles/my-site --name "My Site" --version "1.0.0"

# 2. Publish to Veilid DHT
./target/release/roselite publish my-site.veilidpkg
# Output: 📤 Published successfully! Slug: my-site

# 3. Start the gateway server
./target/release/roselite-gateway --port 3000 --domain localhost:3000

# 4. Access via subdomain routing
curl -H "Host: my-site.localhost:3000" http://localhost:3000/
# Or in browser: http://my-site.localhost:3000

# 5. Production: Deploy gateway with proper domain
./target/release/roselite-gateway --port 443 --domain veilid.app
# Sites accessible at: https://my-site.veilid.app
```

## 🌐 Gateway Server Architecture

The Roselite Gateway provides web browser access to Veilid-hosted content:

### Features
- **🔀 Subdomain Routing**: `my-site.domain.com` → retrieves `my-site` from DHT
- **📁 Multi-tier Caching**: In-memory + filesystem fallback
- **🛡️ Security**: Path traversal protection, safe file serving
- **⚡ Performance**: Optimized for static site delivery
- **🔧 Fallback**: Works even with Veilid connectivity issues

### Gateway Commands

```bash
# Start gateway server
roselite-gateway --port 8080 --domain your-domain.com

# Development mode with detailed logging
RUST_LOG=debug roselite-gateway --port 3000 --domain localhost:3000

# Production deployment
roselite-gateway --port 443 --domain veilid.app
```

### Gateway URL Structure

```
https://[slug].[domain]/[path]
       ↓       ↓      ↓
   DHT lookup  |   File path
            Your domain
```

Examples:
- `https://my-portfolio.veilid.app/` → serves `index.html`
- `https://my-blog.veilid.app/about.html` → serves `about.html`
- `https://docs.veilid.app/guide/` → serves `guide/index.html`

## 📦 Enhanced Package Format

Roselite packages (`.veilidpkg`) contain optimized metadata:

```json
{
  "name": "My Website",
  "version": "1.0.0", 
  "description": "My personal portfolio",
  "developer": "Developer Name",
  "entry": "index.html",
  "category": "website",
  "format_version": "1.0.0",
  "created_at": "2024-06-02T08:30:00Z",
  "file_count": 25,
  "total_size": 2048576
}
```

## 🔍 Commands Reference

### Bundle

Package your static site files:

```bash
# Basic bundling
roselite bundle path/to/site --name "My Site" --version "1.0.0"

# With metadata
roselite bundle ./website/ \
  --name "Portfolio" \
  --version "2.1.0" \
  --description "My personal website"
```

### Publish

Deploy to Veilid DHT with automatic slug generation:

```bash
roselite publish my-site.veilidpkg

# Output:
# 📤 Publishing package: my-site.veilidpkg
# ✅ Published successfully!
# 🔗 Slug: my-site
# 🌐 Gateway URL: https://my-site.veilid.app (when gateway is running)
```

### Gateway Server

Run the web gateway for browser access:

```bash
# Local development
roselite-gateway --port 3000 --domain localhost:3000

# Production deployment  
roselite-gateway --port 443 --domain your-domain.com

# Custom configuration
roselite-gateway \
  --port 8080 \
  --domain veilid.app \
  --cache-dir ./site-cache
```

### Access (Legacy)

Direct DHT access (mainly for testing):

```bash
# Access by slug
roselite access "my-site"

# Show raw DHT data
roselite access "site-key" --raw
```

## 💡 Production Deployment

### 1. Server Setup

```bash
# On your server
git clone https://github.com/yourusername/roselite
cd roselite  
cargo build --release

# Install as system service (optional)
sudo cp target/release/roselite-gateway /usr/local/bin/
```

### 2. DNS Configuration

For wildcard subdomain support:

```dns
# Wildcard A record pointing to your server
*.veilid.app. IN A 1.2.3.4

# Root domain (optional)  
veilid.app. IN A 1.2.3.4
```

### 3. SSL/TLS Setup

```bash
# With Let's Encrypt
certbot certonly --standalone -d "*.veilid.app" -d "veilid.app"

# Start gateway with TLS (requires TLS proxy like nginx)
# Or use a reverse proxy like Cloudflare
```

### 4. Gateway Deployment

```bash
# Production command
./target/release/roselite-gateway \
  --port 8080 \
  --domain veilid.app
  
# Behind reverse proxy (recommended)
# nginx/caddy handles TLS, forwards to gateway
```

## 🛡️ Enhanced Veilid Integration

### Network Features
- **🔄 Automatic Fallback**: Graceful degradation when Veilid network unavailable
- **📊 Connection Management**: Smart retry logic and state monitoring  
- **🔐 Security**: Proper cryptographic key handling
- **⚡ Performance**: Optimized DHT operations with caching

### Network States
- **Full Veilid**: Connected to DHT, full P2P functionality
- **Fallback Mode**: Local storage when network unavailable
- **Hybrid**: Automatic switching between modes

### Configuration

```bash
# Check network status
roselite status

# Force fallback mode for testing
ROSELITE_FALLBACK=true roselite publish site.veilidpkg

# Enable debug logging
RUST_LOG=debug roselite-gateway --port 3000 --domain localhost:3000
```

## 📱 Complete Example

```bash
# Create a Next.js site (or any static site)
npx create-next-app@latest my-portfolio --app --typescript
cd my-portfolio
npm run build

# Bundle for Roselite
cd ../roselite
./target/release/roselite bundle ../my-portfolio/out \
  --name "My Portfolio" \
  --version "1.0.0"

# Publish to Veilid DHT
./target/release/roselite publish my-portfolio.veilidpkg
# Output: ✅ Published! Slug: my-portfolio  

# Start gateway server
./target/release/roselite-gateway --port 3000 --domain localhost:3000

# Access in browser or via curl
curl -H "Host: my-portfolio.localhost:3000" http://localhost:3000/
# Browser: http://my-portfolio.localhost:3000
```

## 🔧 Development Status

> **✅ Production Ready**: Complete P2P hosting system with web gateway

### Completed ✅
- **Core DHT Operations**: Publish, retrieve, delete from Veilid network
- **Gateway Server**: Full HTTP server with subdomain routing
- **Caching System**: Multi-tier caching (memory + filesystem)
- **Fallback Storage**: Works without network connectivity
- **Security**: Path traversal protection, safe file serving
- **Performance**: Optimized for static site delivery
- **Production Ready**: Complete deployment workflow

### Planned 🚧
- **Enhanced Caching**: TTL and cache invalidation
- **Admin Interface**: Web UI for gateway management
- **Analytics**: Basic usage metrics and monitoring
- **CDN Integration**: Optional CDN support for performance
- **Custom Domains**: DNS-based custom domain mapping

## 🎯 Advanced Use Cases

### Personal & Business
- **Portfolio Sites**: Developer portfolios with instant deployment
- **Documentation**: Project docs with P2P hosting
- **Landing Pages**: Marketing sites without hosting costs
- **Blogs**: Static site generators → Veilid deployment

### Enterprise & Web3
- **Decentralized Apps**: Frontend hosting for dApps  
- **Corporate Sites**: Censorship-resistant corporate presence
- **Emergency Sites**: Disaster-resistant information distribution
- **Web3 Integration**: Gateway integration with blockchain projects

### Community & Open Source
- **Project Sites**: Open source project hosting
- **Community Docs**: Decentralized knowledge bases
- **Event Sites**: Conference and meetup pages
- **Resource Sharing**: Educational content distribution

## 🔐 Security & Privacy

- **🔒 End-to-End**: Content encrypted in Veilid DHT
- **🌍 Distributed**: No single point of failure  
- **🛡️ Gateway Security**: Path traversal protection
- **🔍 Privacy**: No traditional server logs or tracking
- **⚡ Resilient**: Automatic fallback mechanisms

## 💰 Monetization & Self-Hosting

**Yes! The Roselite architecture supports both hosted gateways and self-hosting:**

### Hosted Gateway Service (Paywalled)
- Provide `https://*.veilid.app` as a premium service
- Subscription tiers based on bandwidth/storage
- Enhanced features: analytics, custom domains, CDN
- Professional support and SLA guarantees

### Self-Hosted Gateways (Free/Open Source)
- Complete gateway server is open source
- Users can deploy on their own domains
- `https://*.your-domain.com` with their own infrastructure
- No vendor lock-in - content remains in Veilid DHT

### Hybrid Model Benefits
- **Content Portability**: Sites work on any gateway
- **No Lock-in**: Switch between gateways anytime  
- **Market Choice**: Premium vs. self-hosted options
- **Network Effect**: More gateways = better reliability

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Test with local development setup
5. Push to the branch (`git push origin feature/amazing-feature`)
6. Open a Pull Request

### Development Setup

```bash
# Clone and build
git clone https://github.com/yourusername/roselite
cd roselite
cargo build

# Run tests
cargo test

# Local testing with gateway
cargo build --release
./target/release/roselite-gateway --port 3000 --domain localhost:3000
```

## 📄 License

This project is licensed under MIT OR Apache-2.0.

## 🔗 Links

- [Veilid Project](https://veilid.com)
- [Veilid Developer Book](https://veilid.gitlab.io/developer-book/)
- [Gateway Documentation](./docs/gateway.md)
- [API Reference](./docs/api.md)
- [Issue Tracker](https://github.com/yourusername/roselite/issues)
- [Discussions](https://github.com/yourusername/roselite/discussions) 