# Quick Start

Deploy your first static site to the Veilid DHT in under 5 minutes.

## What you'll build

In this quick start, you'll:

1. Bundle a simple HTML site
2. Publish it to the Veilid DHT
3. Start a local gateway
4. Access your site in a web browser

!!! tip "Before you start"

    Make sure you have [installed Roselite](installation.md) and have a basic static site ready (even a single HTML file works).

## Step 1: Prepare your site

Create a simple site or use an existing one:

=== "Create a new site"

    ```bash
    # Create a simple test site
    mkdir my-test-site
    cd my-test-site
    
    # Create index.html
    cat > index.html << 'EOF'
    <!DOCTYPE html>
    <html>
    <head>
        <title>My Roselite Site</title>
        <style>
            body { font-family: Arial, sans-serif; margin: 40px; }
            .container { max-width: 600px; margin: 0 auto; }
            .hero { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); 
                    color: white; padding: 2rem; border-radius: 8px; }
        </style>
    </head>
    <body>
        <div class="container">
            <div class="hero">
                <h1>🚀 Welcome to Roselite!</h1>
                <p>This site is hosted on the Veilid DHT with zero censorship.</p>
            </div>
            <h2>Censorship-Resistant Hosting</h2>
            <p>Your content is now distributed across the Veilid network.</p>
        </div>
    </body>
    </html>
    EOF
    ```

=== "Use existing site"

    ```bash
    # Navigate to your existing static site directory
    cd path/to/your/static/site
    
    # Make sure it has an index.html file
    ls -la
    ```

## Step 2: Bundle your site

Package your site into a Veilid package:

```bash
# Bundle the current directory
roselite bundle . --name "My Test Site" --version "1.0.0"

# This creates: my-test-site.veilidpkg
```

!!! info "Package created"

    The `.veilidpkg` file contains your entire site optimized for DHT storage.

## Step 3: Publish to DHT

Deploy your package to the Veilid network:

```bash
# Publish the package
roselite publish my-test-site.veilidpkg

# Output will show:
# 📤 Publishing package: my-test-site.veilidpkg
# ✅ Published successfully!
# 🔗 Slug: my-test-site
```

!!! success "Published!"

    Your site is now distributed across the Veilid DHT. Note the **slug** - you'll need it for access.

## Step 4: Start the gateway

Launch a local gateway server:

```bash
# Start gateway on port 3000
roselite-gateway --port 3000 --domain localhost:3000

# You should see:
# 🌐 Gateway server starting on localhost:3000
# 🚀 Ready to serve Veilid content
```

Keep this terminal open - the gateway needs to run to serve your content.

## Step 5: Access your site

Open a new terminal and access your site:

=== "Using curl"

    ```bash
    # Access via HTTP header
    curl -H "Host: my-test-site.localhost:3000" http://localhost:3000/
    
    # You should see your HTML content
    ```

=== "Using browser"

    Open your web browser and visit:
    
    ```
    http://my-test-site.localhost:3000
    ```
    
    !!! note "Browser setup"
    
        Some browsers may need manual host configuration. If this doesn't work, use the curl method to verify the gateway is working.

## Step 6: Make changes

Update your site and redeploy:

```bash
# Edit your content
echo '<p>Updated content!</p>' >> index.html

# Re-bundle and publish
roselite bundle . --name "My Test Site" --version "1.0.1"
roselite publish my-test-site.veilidpkg

# Content is updated in the DHT
```

## What just happened?

1. **Bundled**: Your static files were packaged into a DHT-optimized format
2. **Published**: The package was distributed across Veilid nodes worldwide
3. **Served**: The gateway retrieved your content from the DHT and served it via HTTP
4. **Accessed**: Your site is now accessible through any Roselite gateway

## Next steps

🎉 **Congratulations!** You've successfully deployed a censorship-resistant static site.

### Learn more

- [Architecture Overview](../architecture/) - Understanding how it all works
- [CLI Reference](../reference/cli-commands.md) - Complete command documentation
- [Gateway Setup](../reference/gateway-api.md) - Advanced gateway configuration

### Production deployment

- [First Deployment Guide](first-deployment.md) - Complete production setup
- [Gateway Configuration](../reference/configuration.md) - Production gateway setup
- [Custom Domains](../architecture/gateway-system.md) - Using your own domain

### Advanced features

- **Multiple Gateways**: Deploy on multiple servers for redundancy
- **Custom Domains**: Use your own domain names
- **SSL/TLS**: Production HTTPS setup
- **CI/CD Integration**: Automated deployments

!!! tip "Share your site"

    Your site URL can be shared with anyone. As long as they have access to a Roselite gateway, they can view your content - even if the original gateway goes down.

## Troubleshooting

### Common issues

!!! warning "Gateway connection failed"

    **Issue**: Can't connect to gateway
    
    **Solution**:
    ```bash
    # Check if gateway is running
    ps aux | grep roselite-gateway
    
    # Check port availability
    netstat -an | grep 3000
    
    # Try different port
    roselite-gateway --port 8080 --domain localhost:8080
    ```

!!! warning "Publish failed"

    **Issue**: Package publishing fails
    
    **Solution**:
    ```bash
    # Check network connectivity
    roselite status
    
    # Try with debug logging
    RUST_LOG=debug roselite publish my-test-site.veilidpkg
    
    # Check package file exists
    ls -la *.veilidpkg
    ```

!!! warning "Site not loading"

    **Issue**: Browser shows error or empty page
    
    **Solution**:
    ```bash
    # Test with curl first
    curl -H "Host: my-test-site.localhost:3000" http://localhost:3000/
    
    # Check gateway logs
    # Look at terminal where gateway is running
    
    # Verify slug is correct
    roselite list
    ```

Need more help? Check the [troubleshooting guide](../reference/configuration.md#troubleshooting) or [open an issue](https://github.com/jdbohrman/roselite/issues). 