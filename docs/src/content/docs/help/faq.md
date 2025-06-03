---
title: Frequently Asked Questions
description: Common questions and answers about Roselite
---

Here are answers to the most frequently asked questions about Roselite.

## General Questions

### What is Roselite?

Roselite is a peer-to-peer static site hosting platform built on the Veilid Distributed Hash Table (DHT). It allows you to deploy static websites to a decentralized network without relying on traditional hosting providers.

### How is Roselite different from traditional hosting?

| Traditional Hosting | Roselite |
|-------------------|----------|
| Centralized servers | Distributed network |
| Monthly hosting fees | No ongoing costs |
| Single point of failure | Fault tolerant |
| Geographic limitations | Global distribution |
| Censorship vulnerable | Censorship resistant |
| DNS dependency | Direct DHT addressing |

### Is Roselite really free?

Yes! Once you deploy your site to the DHT, there are no ongoing hosting costs. You only need:
- A one-time setup (installing the CLI)
- Minimal bandwidth during deployment
- No monthly fees or server maintenance

### What types of websites can I host on Roselite?

Roselite supports static websites including:
- **Personal blogs and portfolios**
- **Documentation sites**
- **Landing pages**
- **Static web apps** (React, Vue, Angular builds)
- **Image galleries**
- **Simple e-commerce catalogs**

**Not supported:**
- Server-side applications (PHP, Node.js, Python backends)
- Databases or dynamic content
- Real-time applications requiring WebSockets

## Technical Questions

### How does the DHT network work?

The Distributed Hash Table (DHT) is a decentralized storage system where:
1. Your content is bundled into a .veilidpkg package
2. The package gets a cryptographic DHT address
3. The package is distributed across network nodes
4. Gateway servers retrieve and serve your content
5. Multiple nodes store copies for redundancy

### What happens if nodes go offline?

Roselite uses redundancy to handle node failures:
- Content is stored on multiple nodes
- If some nodes go offline, others continue serving
- The network automatically redistributes content
- Gateway servers cache popular content locally

### How fast is site loading?

Performance depends on several factors:
- **Initial load**: 1-3 seconds (DHT lookup + content retrieval)
- **Cached content**: Near-instant (gateway caching)
- **Geographic location**: Closer gateways = faster loading
- **Content size**: Smaller sites load faster

Performance is generally comparable to traditional CDNs for static content.

### What are the file size limits?

Current limits:
- **Individual files**: 10MB maximum
- **Total site size**: 100MB maximum
- **File count**: 10,000 files maximum

For larger sites, consider:
- Compressing images and assets
- Splitting into multiple deployments
- Using external CDNs for large media

### Can I use a custom domain?

Currently, sites are accessed via:
- **DHT addresses**: Direct DHT key access
- **Gateway subdomains**: `http://your-app-slug.localhost:8080` (local development)
- **Custom domains**: Requires deploying your own gateway with wildcard DNS

**Universal gateway infrastructure is not yet deployed**, so custom domains require:
1. Running your own gateway server
2. Setting up wildcard DNS (`*.yourdomain.com`)
3. Configuring HTTPS certificates

## Security & Privacy

### How secure is my content?

Roselite provides several security features:
- **Content integrity**: Cryptographic verification prevents tampering
- **Immutable addresses**: Content can't be replaced without changing the address
- **Distributed storage**: No single point of attack
- **Encryption in transit**: All network communication is encrypted

### Can my site be censored or taken down?

Censorship resistance is a core feature:
- **No central authority** controls the network
- **Multiple nodes** store your content
- **Gateway diversity** prevents single points of control
- **Direct DHT access** bypasses traditional DNS

However, individual gateways might be blocked in some regions.

### Who can see my content?

Content on Roselite is:
- **Publicly accessible** to anyone with the DHT address
- **Not indexed** by default (no central directory)
- **Pseudonymous** (not tied to your identity unless you include it)

For private content, consider encryption before deployment.

### Can I update or delete my site?

**Updates**: Currently, content is immutable. To update:
1. Bundle and publish a new version (gets a new DHT address)
2. Update links to point to the new address
3. Old version remains accessible unless abandoned

**Deletion**: Content may persist on the network even if you stop maintaining it, as other nodes may continue storing copies.

*Note: Update and deletion features are planned for future releases.*

## Usage Questions

### Do I need technical knowledge to use Roselite?

Basic command-line knowledge is helpful, but not extensive technical expertise:
- **Beginner**: Follow our Quick Start guide step-by-step
- **Intermediate**: Customize bundle options and gateway setup
- **Advanced**: Run your own gateway, contribute to development

### Can I migrate from my current hosting provider?

Yes! Migration is straightforward for static sites:

1. **Export your current site**:
   - Download all files via FTP/SFTP
   - Export from your CMS as static files
   - Use static site generators (Jekyll, Hugo, etc.)

2. **Prepare for Roselite**:
   - Ensure all resources are relative paths
   - Optimize images and assets
   - Test locally first

3. **Deploy with Roselite**:
   ```bash
   roselite bundle ./my-site --name "My Website"
   roselite publish my-website.veilidpkg
   ```

### How do I handle dynamic features like contact forms?

Since Roselite serves static content, dynamic features require external services:

- **Contact forms**: Use services like Formspree, Netlify Forms, or custom APIs
- **Comments**: Integrate Disqus, Utterances, or similar
- **Search**: Use client-side search (Lunr.js) or external search APIs
- **Analytics**: Use privacy-focused solutions like Plausible or Fathom

### Can I use Roselite for commercial websites?

Absolutely! Roselite is suitable for:
- **Business websites** and landing pages
- **E-commerce catalogs** (with external payment processing)
- **SaaS documentation** and marketing sites
- **Portfolio sites** for freelancers and agencies

Just ensure compliance with relevant laws and regulations in your jurisdiction.

## Network & Gateways

### What are gateway servers?

Gateway servers are HTTP/HTTPS bridges that:
- Retrieve content from the DHT network
- Serve it via standard web protocols (HTTP/HTTPS)
- Cache popular content for performance
- Provide subdomain-based routing (e.g., `app-name.localhost:8080`)

### Can I run my own gateway?

Yes! Running your own gateway provides:
- **Better performance** for your sites
- **Independence** from third-party gateways
- **Custom domain support**
- **Community contribution**

To run a gateway:
```bash
cd roselite-gateway
cargo run --release -- --domain "yourdomain.com" --port 80
```

### What if gateway servers go down?

Currently, gateway availability depends on:
- **Local development**: Run your own localhost gateway
- **Production**: Deploy your own gateway infrastructure
- **Future**: Universal gateway network (not yet deployed)

The DHT network continues operating even if gateways are temporarily unavailable - content remains stored and accessible.

### How do I access my site without a gateway?

You can access DHT content directly using the CLI:
```bash
roselite access your-dht-key
```

This fetches content directly from the DHT without requiring a gateway server.

## Development & Integration

### Can I use Roselite with static site generators?

Yes! Roselite works with any static site generator:

- **Jekyll**: Build with `jekyll build`, bundle `_site/`
- **Hugo**: Build with `hugo`, bundle `public/`
- **Gatsby**: Build with `gatsby build`, bundle `public/`
- **Next.js**: Export with `next export`, bundle `out/`
- **Nuxt.js**: Generate with `nuxt generate`, bundle `dist/`

### How do I integrate Roselite into CI/CD?

Example GitHub Actions workflow:
```yaml
name: Deploy to Roselite
on:
  push:
    branches: [main]
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Install Roselite CLI
        run: cargo install --path roselite-cli
      - name: Build site
        run: npm run build
      - name: Bundle for DHT
        run: roselite bundle ./dist --name "My Site"
      - name: Publish to DHT
        run: roselite publish my-site.veilidpkg
```

### Can I use analytics with Roselite sites?

Yes! Use privacy-focused, client-side analytics:
- **Plausible**: Lightweight, privacy-focused
- **Fathom**: Simple, GDPR compliant
- **Umami**: Self-hosted, open source
- **GoatCounter**: Simple, no-tracking

Avoid heavy analytics that might slow down loading.

### How do I handle SEO for Roselite sites?

SEO strategies for decentralized sites:
- **Content optimization**: Focus on quality, relevant content
- **Meta tags**: Include proper title, description, and Open Graph tags
- **Site structure**: Use semantic HTML and clear navigation
- **Performance**: Optimize loading speed and file sizes
- **Sitemap**: Generate and include sitemap.xml
- **Social sharing**: Include social media meta tags

Note: Search engines may take time to discover and index DHT-hosted content.

## Troubleshooting

### My bundle or publish is failing

Common solutions:
1. Check file sizes: `find . -type f -size +10M`
2. Verify directory structure and entry point (index.html)
3. Check network connectivity for DHT operations
4. Try with verbose output: `RUST_LOG=debug roselite publish my-site.veilidpkg`

### My site isn't loading through gateways

Troubleshooting steps:
1. Wait 1-5 minutes for DHT propagation
2. Check if content exists: `roselite access your-app-slug`
3. Verify gateway is running: `curl http://localhost:8080`
4. Check subdomain format: `http://your-app-slug.localhost:8080`

### Local gateway won't start

Try these solutions:
```bash
# Check if port is in use
lsof -i :8080

# Try different port
cd roselite-gateway && cargo run -- --port 3000

# Check for build errors
cargo build --release
```

### I'm getting permission errors during installation

Try these solutions:
```bash
# Install to user directory
cargo install --path roselite-cli --root ~/.local

# Ensure ~/.local/bin is in PATH
export PATH="$HOME/.local/bin:$PATH"

# Fix cargo permissions
sudo chown -R $USER ~/.cargo
```

### Content not found in DHT

Possible causes:
- DHT key is incorrect
- Content hasn't propagated yet (wait a few minutes)
- Network connectivity issues
- Package wasn't successfully published

Try:
```bash
# Verify the exact DHT key
roselite access your-dht-key

# Check recent publications
ls -la *.veilidpkg

# Re-publish if needed
roselite publish your-package.veilidpkg
```

## Community & Support

### How can I get help?

Multiple support channels are available:
- **Documentation**: Comprehensive guides and references
- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Community Q&A and general discussion
- **Email**: Direct contact with maintainers

### How can I contribute to Roselite?

We welcome contributions:
- **Code**: Features, bug fixes, optimizations
- **Documentation**: Guides, tutorials, translations
- **Testing**: Try new features, report bugs
- **Community**: Help other users, moderate discussions
- **Infrastructure**: Run gateways, provide bootstrap nodes

See our [Contributing Guide](../community/contributing/) for details.

### What's the roadmap for Roselite?

Planned features include:
- **Universal gateways**: Public gateway infrastructure deployment
- **Content updates**: Modify deployed sites without changing addresses
- **Custom domains**: CNAME support for branded URLs
- **Enhanced CLI**: Better progress reporting and management
- **Gateway improvements**: Performance and feature enhancements
- **Mobile apps**: iOS and Android deployment tools

### Is Roselite production-ready?

Roselite is currently in **early development**:
- **Core DHT functionality** is stable and tested
- **Gateway infrastructure** requires manual setup
- **Universal gateways** are not yet deployed
- **API may change** between versions
- **Production use** is possible but requires technical setup

We're working toward deploying universal gateway infrastructure and a stable 1.0 release.

---

**Have a question not covered here?** 

Join our [community discussions](https://github.com/jdbohrman-tech/roselite/discussions) or [open an issue](https://github.com/jdbohrman-tech/roselite/issues) on GitHub. 