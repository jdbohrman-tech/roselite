# Quick Start Guide

Get your first site published to the decentralized web in just a few minutes!

## Overview

In this guide, you'll:

1. Create a simple static website
2. Bundle it with Roselite
3. Publish it to the Veilid DHT
4. Access it through the gateway

!!! tip "Prerequisites"
    Make sure you have [Roselite installed](installation.md) before proceeding.

## Step 1: Create Your First Site

Let's create a simple website to get started:

```bash
# Create a new directory for your site
mkdir my-first-roselite-site
cd my-first-roselite-site

# Create a basic HTML page
cat > index.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>My First Roselite Site</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 2rem;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            color: white;
        }
        .container {
            background: rgba(255, 255, 255, 0.1);
            padding: 2rem;
            border-radius: 10px;
            backdrop-filter: blur(10px);
        }
        h1 { color: #fff; text-align: center; }
        .emoji { font-size: 3rem; text-align: center; margin: 2rem 0; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🌹 Welcome to My Roselite Site!</h1>
        <div class="emoji">🚀</div>
        <p>This website is hosted on the <strong>decentralized web</strong> using the Veilid DHT network. No traditional servers required!</p>
        <p>Built with ❤️ using <a href="https://github.com/jdbohrman/roselite" style="color: #fff;">Roselite</a></p>
    </div>
</body>
</html>
EOF

# Create an about page
cat > about.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>About - My Roselite Site</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 2rem;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            color: white;
        }
        .container {
            background: rgba(255, 255, 255, 0.1);
            padding: 2rem;
            border-radius: 10px;
            backdrop-filter: blur(10px);
        }
        nav { margin-bottom: 2rem; }
        nav a { color: #fff; margin-right: 1rem; }
    </style>
</head>
<body>
    <div class="container">
        <nav>
            <a href="index.html">Home</a>
            <a href="about.html">About</a>
        </nav>
        <h1>About This Site</h1>
        <p>This is a demonstration of decentralized web hosting using Roselite and the Veilid DHT network.</p>
        <p>Key features:</p>
        <ul>
            <li>No centralized servers</li>
            <li>Censorship resistant</li>
            <li>Privacy-focused</li>
            <li>Globally accessible</li>
        </ul>
    </div>
</body>
</html>
EOF
```

Your directory structure should now look like this:
```
my-first-roselite-site/
├── index.html
└── about.html
```

## Step 2: Start the Roselite Gateway

Before publishing, start the Roselite gateway to handle requests:

```bash
roselite gateway start
```

Expected output:
```
🌹 Roselite Gateway starting...
✅ Veilid DHT connected
🌐 Gateway running on http://localhost:3000
🚀 Ready to serve content!
```

!!! info "Gateway Purpose"
    The gateway acts as a bridge between the traditional web and the Veilid DHT, allowing regular browsers to access your decentralized sites.

## Step 3: Bundle Your Site

Create a bundle of your site files:

```bash
roselite bundle .
```

This command will:
- Scan all files in the current directory
- Compress them into an optimized bundle
- Generate metadata for DHT storage

Expected output:
```
📦 Bundling site from current directory...
✅ Found 2 files (index.html, about.html)
📊 Total size: 2.1 KB
🗜️  Compressed to: 1.2 KB
💾 Bundle saved as: site.tar.gz
```

## Step 4: Publish to the DHT

Now publish your bundle to the Veilid DHT:

```bash
roselite publish site.tar.gz
```

This will:
- Upload your bundle to the DHT network
- Generate a unique content address
- Register the site with the gateway

Expected output:
```
🚀 Publishing site to Veilid DHT...
📡 Uploading to network... ⠋
✅ Site published successfully!

📍 DHT Address: VLD0:abc123def456...
🌐 Gateway URL: http://localhost:3000/site/abc123def456
🔗 Share URL: https://gateway.roselite.org/site/abc123def456

💡 Your site is now live on the decentralized web!
```

## Step 5: Access Your Site

You can now access your site in several ways:

### Local Gateway
```bash
# Open in your default browser
roselite open abc123def456

# Or manually visit:
# http://localhost:3000/site/abc123def456
```

### Public Gateway
Share this URL with others:
```
https://gateway.roselite.org/site/abc123def456
```

### Direct DHT Access (Advanced)
For applications with Veilid integration:
```
VLD0:abc123def456...
```

## Step 6: Verify Your Site

Test that everything is working correctly:

```bash
# Check site status
roselite status abc123def456

# List your published sites
roselite list

# Get detailed info about your site
roselite info abc123def456
```

## Next Steps

Congratulations! 🎉 You've successfully published your first site to the decentralized web. Here's what you can do next:

### Publish a Real Site

Try publishing an existing static site:

```bash
# For a Hugo site
hugo build
roselite bundle public/
roselite publish site.tar.gz

# For a Jekyll site
jekyll build
roselite bundle _site/
roselite publish site.tar.gz

# For any static site generator
# 1. Build your site
# 2. Bundle the output directory
# 3. Publish the bundle
```

### Learn More

- [**Publishing Guide**](publishing.md) - Advanced publishing options
- [**Gateway Usage**](gateway.md) - Running your own gateway
- [**CLI Reference**](cli.md) - Complete command reference
- [**Configuration**](configuration.md) - Customize Roselite behavior

### Update Your Site

To update your site:

```bash
# Make changes to your files
echo "<p>Updated content!</p>" >> index.html

# Create a new bundle
roselite bundle .

# Publish the update (this will create a new version)
roselite publish site.tar.gz

# Or update the existing site reference
roselite update abc123def456 site.tar.gz
```

## Troubleshooting

### Site Not Loading

If your site isn't accessible:

1. **Check gateway status**:
   ```bash
   roselite gateway status
   ```

2. **Verify DHT connection**:
   ```bash
   roselite status
   ```

3. **Check site status**:
   ```bash
   roselite status abc123def456
   ```

### Slow Loading

DHT propagation can take a few moments. If your site loads slowly:

- Wait 30-60 seconds for DHT propagation
- Try refreshing the page
- Check your internet connection

### Need Help?

- Check the [troubleshooting guide](troubleshooting.md)
- Visit our [GitHub Issues](https://github.com/jdbohrman/roselite/issues)
- Join the community discussions

!!! success "Welcome to the Decentralized Web!"
    You're now part of a new internet architecture that's more resilient, private, and censorship-resistant. Happy publishing! 🌹 