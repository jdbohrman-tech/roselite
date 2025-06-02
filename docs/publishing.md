# Publishing Guide

Master the art of publishing sites to the decentralized web with Roselite.

## Overview

Publishing with Roselite is a two-step process:

1. **Bundle** - Package your static files into a compressed archive
2. **Publish** - Upload the bundle to the Veilid DHT network

This guide covers advanced publishing techniques, optimization strategies, and best practices.

## Basic Publishing Workflow

### 1. Prepare Your Site

Ensure your static site is ready for publication:

```bash
# Example with a Hugo site
hugo build

# Example with a Jekyll site  
jekyll build

# Example with a React app
npm run build

# Example with plain HTML
# No build step needed - just organize your files
```

### 2. Bundle Your Site

Create an optimized bundle:

```bash
# Basic bundling
roselite bundle ./public

# Advanced bundling with options
roselite bundle ./public \
  --output my-site-v1.0.tar.gz \
  --compression zstd \
  --exclude "*.map" \
  --exclude "*.log"
```

### 3. Publish to DHT

Upload your bundle to the network:

```bash
# Basic publish
roselite publish my-site-v1.0.tar.gz

# Publish with metadata
roselite publish my-site-v1.0.tar.gz \
  --name "My Awesome Site" \
  --description "A showcase of decentralized web hosting" \
  --tags "portfolio,tech,demo"
```

## Advanced Publishing Techniques

### Custom Bundle Configuration

Create a `.roselite.toml` file in your project root for automatic configuration:

```toml
[bundle]
# Output filename pattern
output = "site-{version}.tar.gz"

# Compression settings
compression = "zstd"
compression_level = 9

# File exclusions
exclude = [
    "*.log",
    "*.map", 
    "node_modules/**",
    ".git/**",
    "*.tmp"
]

# Include hidden files
include_hidden = false

# Maximum bundle size
max_size = "50MB"

[publish]
# Default metadata
name = "My Site"
description = "Powered by Roselite"
tags = ["static", "decentralized"]

# Publishing options
ttl = 604800  # 7 days
replicas = 5
private = false
```

With this configuration, you can simply run:

```bash
roselite bundle .
roselite publish
```

### Site Versioning

Manage multiple versions of your site:

```bash
# Publish version 1.0
roselite bundle ./v1.0
roselite publish site.tar.gz --name "My Site v1.0"

# Publish version 1.1 (new site ID)
roselite bundle ./v1.1  
roselite publish site.tar.gz --name "My Site v1.1"

# Update existing site (same site ID)
roselite bundle ./v1.1
roselite update <SITE_ID> site.tar.gz
```

### Private Sites

Publish private sites that require authentication:

```bash
# Publish as private
roselite publish site.tar.gz --private

# Generate access token
roselite token generate <SITE_ID>

# Share private URL
# https://gateway.roselite.org/private/<SITE_ID>?token=<TOKEN>
```

## Optimization Strategies

### Bundle Size Optimization

Keep your bundles small for faster distribution:

#### 1. Image Optimization
```bash
# Before bundling, optimize images
find ./public -name "*.jpg" -exec jpegoptim --max=85 {} \;
find ./public -name "*.png" -exec optipng -o7 {} \;

# Use modern formats
find ./public -name "*.png" -exec cwebp -q 80 {} -o {}.webp \;
```

#### 2. Asset Minification
```bash
# Minify CSS
find ./public -name "*.css" -exec cleancss -o {} {} \;

# Minify JavaScript  
find ./public -name "*.js" -exec terser {} --compress --mangle -o {} \;

# Minify HTML
find ./public -name "*.html" -exec html-minifier --collapse-whitespace --remove-comments --output {} {} \;
```

#### 3. Compression Comparison

Choose the best compression for your use case:

| Algorithm | Speed | Ratio | Best For |
|-----------|-------|-------|----------|
| `gzip` | Fast | Good | General use |
| `zstd` | Very Fast | Good | Real-time sites |
| `bzip2` | Slow | Best | Archival content |

### Content Delivery Optimization

#### Use a CDN Pattern
Deploy multiple gateways for better performance:

```bash
# Primary gateway
https://gateway.roselite.org/site/<SITE_ID>

# Regional gateways  
https://eu-gateway.roselite.org/site/<SITE_ID>
https://asia-gateway.roselite.org/site/<SITE_ID>
```

#### Implement Caching Headers
Include cache-friendly metadata:

```html
<!-- In your HTML pages -->
<meta http-equiv="Cache-Control" content="public, max-age=3600">
<meta http-equiv="ETag" content="v1.0">
```

## Static Site Generator Integration

### Hugo Integration

Create a Hugo deployment script:

```bash
#!/bin/bash
# deploy.sh

set -e

echo "🏗️  Building Hugo site..."
hugo --minify

echo "📦 Creating Roselite bundle..."
roselite bundle ./public --output "site-$(date +%Y%m%d-%H%M%S).tar.gz"

echo "🚀 Publishing to DHT..."
SITE_ID=$(roselite publish site-*.tar.gz --format json | jq -r '.site_id')

echo "✅ Site published!"
echo "🌐 URL: https://gateway.roselite.org/site/$SITE_ID"

# Clean up
rm site-*.tar.gz
```

### Jekyll Integration

Add to your `_config.yml`:

```yaml
# _config.yml
plugins:
  - jekyll-roselite

roselite:
  bundle:
    exclude:
      - "*.scss"
      - "*.sass"
      - "_site/*.map"
  publish:
    name: "My Jekyll Blog"
    tags: ["blog", "jekyll"]
```

### Next.js Integration

Create a custom deployment command:

```json
{
  "scripts": {
    "deploy": "next build && next export && roselite bundle ./out && roselite publish site.tar.gz"
  }
}
```

## Content Management

### Updating Sites

Update existing sites efficiently:

```bash
# Method 1: Update in place (preserves site ID)
roselite update <SITE_ID> new-bundle.tar.gz

# Method 2: Publish new version (new site ID)  
roselite publish new-bundle.tar.gz

# Method 3: Atomic updates (advanced)
roselite atomic-update <SITE_ID> new-bundle.tar.gz
```

### Content Staging

Set up staging environments:

```bash
# Publish to staging
roselite publish site.tar.gz \
  --name "My Site (Staging)" \
  --tags "staging" \
  --ttl 3600  # 1 hour TTL

# Promote to production
roselite promote <STAGING_ID> --name "My Site"
```

### Rollback Strategy

Prepare for rollbacks:

```bash
# Tag current version before updating
roselite tag <SITE_ID> "stable"

# Update site
roselite update <SITE_ID> new-bundle.tar.gz

# Rollback if needed
roselite rollback <SITE_ID> "stable"
```

## Security Considerations

### Content Validation

Validate your content before publishing:

```bash
# Check for sensitive files
roselite validate ./public

# Scan for security issues
roselite security-scan site.tar.gz
```

### Access Control

Implement access controls for sensitive content:

```bash
# Publish with access restrictions
roselite publish site.tar.gz \
  --private \
  --allowed-domains "*.example.com" \
  --allowed-ips "192.168.1.0/24"
```

### Content Integrity

Verify content integrity:

```bash
# Generate checksums
roselite checksum site.tar.gz

# Verify published content
roselite verify <SITE_ID>
```

## Monitoring and Analytics

### Site Health Monitoring

Monitor your published sites:

```bash
# Check site availability
roselite health-check <SITE_ID>

# Monitor multiple sites
roselite monitor site1 site2 site3

# Set up alerts
roselite alert create \
  --site <SITE_ID> \
  --condition "availability < 90%" \
  --webhook "https://hooks.slack.com/..."
```

### Usage Analytics

Track site usage:

```bash
# Get site statistics
roselite stats <SITE_ID>

# Export analytics data
roselite analytics export <SITE_ID> --format csv
```

## Troubleshooting

### Common Publishing Issues

#### Bundle Too Large
```bash
# Check bundle size
roselite bundle . --dry-run

# Reduce size with better compression
roselite bundle . --compression bzip2

# Exclude unnecessary files
roselite bundle . --exclude "*.pdf" --exclude "videos/*"
```

#### Network Timeouts
```bash
# Increase timeout
roselite publish site.tar.gz --timeout 300

# Use more replicas for reliability
roselite publish site.tar.gz --replicas 7
```

#### DHT Connection Issues
```bash
# Check network status
roselite status

# Try different bootstrap nodes
roselite config set network.bootstrap_nodes "alt1.veilid.org,alt2.veilid.org"

# Debug connection
roselite -v publish site.tar.gz
```

## Best Practices

### 📁 File Organization
- Keep related files together
- Use descriptive filenames
- Organize assets in subdirectories
- Include a clear `index.html`

### 🗜️ Compression
- Use `zstd` for frequently updated sites
- Use `bzip2` for long-term archival
- Exclude source maps and logs
- Optimize images before bundling

### 🌐 Distribution
- Use descriptive site names and tags
- Set appropriate TTL values
- Consider geographic distribution
- Monitor site health regularly

### 🔒 Security
- Never include sensitive data
- Validate content before publishing
- Use private publishing for internal sites
- Implement proper access controls

### 📊 Performance
- Keep bundles under 50MB when possible
- Use compression efficiently
- Optimize assets before bundling
- Monitor load times and availability

!!! tip "Pro Tip"
    Create a deployment pipeline that automatically builds, bundles, and publishes your site whenever you push changes to your repository. This ensures consistent deployments and reduces manual errors.

## Next Steps

- Learn about [Gateway Usage](gateway.md) for hosting your own gateway
- Explore [Content Management](content.md) for advanced content strategies  
- Check the [CLI Reference](cli.md) for all available commands
- Review [Configuration](configuration.md) for customization options 