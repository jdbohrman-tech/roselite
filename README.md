# Roselite - P2P Static Site Hosting via Veilid DHT

Deploy static websites to the [Veilid](https://veilid.com) decentralized network. No servers, no DNS setup required - just publish and share.

## 🚀 Features

- **📦 Simple Packaging**: Bundle static sites into `.veilidpkg` format
- **🌐 Instant Deployment**: Publish to Veilid DHT with one command
- **🔗 Universal Access**: Automatic gateway URLs for web browser access
- **🔐 Decentralized**: No central servers or hosting providers needed
- **⚡ Real-time**: Content served from DHT with instant updates
- **🎯 Developer Friendly**: Three simple commands: bundle, publish, access

## 🏗️ How It Works

```
Your Static Site  →  Bundle  →  Veilid DHT  →  Gateway URLs
     (HTML/CSS/JS)    (.veilidpkg)  (Distributed)  (Web Access)
```

1. **Bundle** your static site files into a package
2. **Publish** the package to Veilid's distributed hash table
3. **Access** via auto-generated gateway URLs or DHT lookup
4. **Share** the gateway URL - no DNS setup required

## 📁 Project Structure

- **`roselite-core/`** - Core library with Veilid DHT integration
- **`roselite-cli/`** - Command-line tool for P2P hosting

## 🛠️ Quick Start

### Prerequisites

- Rust 1.70+ with Cargo
- Veilid node (for DHT access)

### Installation

```bash
git clone https://github.com/yourusername/roselite
cd roselite
cargo build --release
```

### Basic Usage

```bash
# 1. Bundle your static site
roselite bundle --name "My Site" --version "1.0.0" ./my-site/

# 2. Publish to Veilid DHT
roselite publish my-site.veilidpkg

# 3. Get instant web access URLs
# Output: ✅ Live at: https://my-site.veilid.app

# 4. Access any published site
roselite access "my-site-dht-key"
```

## 📦 Package Format

Roselite packages (`.veilidpkg`) are compressed bundles containing:

```json
{
  "name": "My Website",
  "version": "1.0.0", 
  "description": "My personal portfolio",
  "developer": "Developer Name",
  "entry": "index.html",
  "category": "website",
  "format_version": "1.0.0"
}
```

## 🌐 Gateway System

Published sites get automatic web access through universal gateways:

- **Primary**: `https://your-site.veilid.app`
- **Alternative**: `https://your-site.roselite.app`
- **Local Dev**: `http://dht-your-site.localhost:3000`

### Custom Domains (Optional)

For your own domain, add DNS records:

```dns
yoursite.com. IN TXT "veilid-app=your-dht-key"
yoursite.com. IN CNAME your-site.veilid.app
```

## 🔍 Commands

### Bundle

Package your static site files:

```bash
roselite bundle --name "Portfolio" --version "1.0.0" ./website/
```

### Publish

Deploy to Veilid DHT:

```bash
roselite publish my-site.veilidpkg
```

Output includes:
- DHT storage confirmation
- Instant gateway URLs
- Sharing instructions
- Custom domain setup

### Access

View any published site:

```bash
# By DHT key
roselite access "site-dht-key"

# By gateway URL  
roselite access "https://site.veilid.app"
```

## 📱 Example Workflow

```bash
# Create a simple website
mkdir my-portfolio
echo '<h1>Hello World!</h1>' > my-portfolio/index.html
echo 'body { font-family: Arial; }' > my-portfolio/style.css

# Bundle and publish
roselite bundle --name "My Portfolio" my-portfolio/
roselite publish my-portfolio.veilidpkg

# Share the auto-generated URL
# ✅ Live at: https://my-portfolio.veilid.app
```

## 🔧 Development Status

> **⚠️ Active Development**: Core P2P hosting functionality is implemented.

### Completed ✅
- Static site bundling and packaging
- Veilid DHT publishing and retrieval
- Universal gateway URL generation
- Real-time DHT content resolution
- Web browser access via gateways
- Custom domain DNS integration

### In Progress 🚧
- Enhanced DHT propagation and caching
- Advanced gateway server implementation
- Content versioning and updates
- Performance optimizations

## 🌐 Veilid Integration

Roselite uses Veilid's distributed hash table for:

- **Content Storage**: Static site files stored across DHT nodes
- **Decentralized Access**: No single point of failure
- **Real-time Resolution**: Gateway servers resolve DHT keys instantly
- **P2P Network**: Content distributed across peer nodes

## 🎯 Use Cases

- **Personal Websites**: Deploy portfolios and blogs
- **Project Sites**: Share documentation and demos  
- **Decentralized Web**: Build censorship-resistant sites
- **P2P Hosting**: Eliminate hosting costs and dependencies
- **Web3 Applications**: Static frontends for dApps

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test with local sites
5. Submit a pull request

## 📄 License

This project is licensed under MIT OR Apache-2.0.

## 🔗 Links

- [Veilid Project](https://veilid.com)
- [Veilid Developer Book](https://veilid.gitlab.io/developer-book/)
- [Issue Tracker](https://github.com/yourusername/roselite/issues) 