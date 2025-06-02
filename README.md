# Roselite - P2P Static Site Hosting via Veilid DHT

Deploy static content like JAMStack sites to the [Veilid](https://veilid.com) DHT with instant web access through gateway servers. **Zero censorship, zero single points of failure** - your content lives forever in the decentralized network.

## 🚀 Features

- **📦 Simple Packaging**: Bundle static sites into `.veilidpkg` format
- **🌐 Instant Deployment**: Publish to Veilid DHT with one command
- **🌍 Gateway Server**: Full HTTP server for web browser access via subdomains
- **🔗 Universal Access**: Automatic gateway URLs with SSL termination
- **🔐 Decentralized**: Content stored in Veilid DHT, served through gateways
- **🛡️ Fallback System**: Works even when Veilid network has connectivity issues
- **⚡ High Performance**: In-memory and filesystem caching for fast loading
- **🚫 Zero Censorship**: No central authority can remove your content
- **🌍 Unstoppable**: Multiple gateway redundancy prevents takedowns
- **🎯 Developer Friendly**: Four commands: bundle, publish, gateway, access

## 🏗️ How It Works

```
Static Site → Bundle → Veilid DHT → Gateway Server → Web Browser
(HTML/CSS/JS) (.veilidpkg) (Distributed) (subdomain.domain.com) (HTTPS)
                            ↑
                    CENSORSHIP-RESISTANT
                    No single point of control
```

1. **Bundle** your static site files into a package
2. **Publish** the package to Veilid's distributed hash table  
3. **Gateway** serves content via subdomain routing (e.g., `my-site.localhost:8080`)
4. **Share** the gateway URL - fully accessible in any web browser
5. **Survive** - Content remains accessible even if gateways are blocked

## 🚫 Zero Censorship Architecture

### **Distributed Content Storage**
- **No Central Servers**: Content lives across hundreds of Veilid nodes
- **Cryptographic Integrity**: Content cannot be modified without keys
- **Permanent Storage**: Once published, content persists in the network
- **Global Replication**: Automatic distribution across geographic regions

### **Gateway Independence** 
- **Multiple Gateways**: Any number of gateway servers can serve your content
- **Gateway Redundancy**: If one gateway is blocked, others continue working
- **Self-Hosting**: Anyone can run a gateway - no gatekeeping
- **Domain Flexibility**: Same content accessible via different domains

### **Unstoppable Access**
- **No Single Point of Failure**: Cannot be taken down by targeting one server
- **Resistant to Blocking**: Content accessible through multiple routes
- **International Resilience**: Gateways can operate in any jurisdiction
- **Emergency Access**: Direct Veilid DHT access if all gateways fail

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
./target/release/roselite-gateway --port 443 --domain localhost:8080
# Sites accessible at: https://my-site.localhost:8080
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
roselite-gateway --port 443 --domain localhost:8080
```

### Gateway URL Structure

```
https://[slug].[domain]/[path]
       ↓       ↓      ↓
   DHT lookup  |   File path
            Your domain
```

Examples:
- `https://my-portfolio.localhost:8080/` → serves `index.html`
- `https://my-blog.localhost:8080/about.html` → serves `about.html`
- `https://docs.localhost:8080/guide/` → serves `guide/index.html`

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
# 🌐 Gateway URL: https://my-site.localhost:8080 (when gateway is running)
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
  --domain localhost:8080 \
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
*.localhost:8080. IN A 1.2.3.4

# Root domain (optional)  
localhost:8080. IN A 1.2.3.4
```

### 3. SSL/TLS Setup

```bash
# With Let's Encrypt
certbot certonly --standalone -d "*.localhost:8080" -d "localhost:8080"

# Start gateway with TLS (requires TLS proxy like nginx)
# Or use a reverse proxy like Cloudflare
```

### 4. Gateway Deployment

```bash
# Production command
./target/release/roselite-gateway \
  --port 8080 \
  --domain localhost:8080
  
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
- **Custom Domains**: DNS-based custom domain mapping

## 🎯 Advanced Use Cases

### **Censorship-Resistant Publishing**
- **Journalism**: Publish investigative content without fear of takedown
- **Activism**: Share information in restrictive environments
- **Whistleblowing**: Leak documents with persistent availability
- **Free Speech**: Express ideas without platform dependency
- **Post-Tor Content Hosting**: Build on a network that picks up where Tor left off

### **Emergency & Crisis Communication**
- **Disaster Response**: Information sharing when infrastructure fails
- **Political Unrest**: Communication channels that can't be shut down
- **Network Outages**: Content remains accessible during ISP issues
- **Government Blocking**: Bypass national internet restrictions

### Personal & Business
- **Portfolio Sites**: Developer portfolios with instant deployment
- **Documentation**: Project docs with P2P hosting
- **Landing Pages**: Marketing sites without hosting costs
- **Blogs**: Static site generators → Veilid deployment

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
- **🚫 Uncensorable**: No central authority can remove content
- **🌐 Global**: Accessible from anywhere via multiple routes
- **🔑 User-Controlled**: Only you control your content keys

## 🌍 Censorship Resistance

### **Scenario 1: Gateway Blocking**
```
Government blocks localhost:8080 domain
→ Content still accessible via:
  - Alternative gateways (roselite.net, veilid.org, etc.)
  - Self-hosted gateways (your-gateway.com)
  - Direct Veilid DHT access
  - Tor hidden service gateways
```

### **Scenario 2: ISP Censorship**
```
ISP blocks gateway domains
→ Content still accessible via:
  - VPN to different region
  - Tor browser access
  - Local gateway deployment
  - Mobile network routing
```

### **Scenario 3: Legal Pressure**
```
Legal action against gateway operator
→ Content still accessible via:
  - Other gateway operators in different jurisdictions
  - Community-run gateways
  - Personal gateway deployment
  - Direct P2P network access
```

### **Scenario 4: Technical Attacks**
```
DDoS attacks on gateway servers
→ Content still accessible via:
  - Load balancing across multiple gateways
  - Automatic failover to backup gateways
  - CDN protection (if enabled)
  - Direct Veilid node access
```

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

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

**Why Apache 2.0?** This license supports our zero censorship mission by:
- Encouraging maximum adoption and community gateway deployment
- Protecting against patent trolls who might target decentralized systems
- Allowing commercial gateway services while keeping core technology open
- Building trust through permissive, well-understood terms

## 🔗 Links

- [📚 Complete Documentation](https://yourusername.github.io/roselite) - Full documentation site with getting started guide
- [Veilid Project](https://veilid.com)
- [Veilid Developer Book](https://veilid.gitlab.io/developer-book/)
- [CLI Reference](https://yourusername.github.io/roselite/reference/cli-commands/) - Complete command reference
- [Issue Tracker](https://github.com/yourusername/roselite/issues)
- [Discussions](https://github.com/yourusername/roselite/discussions)
- [💖 Sponsor the Project](https://yourusername.github.io/roselite/sponsor/) - Help fund development 