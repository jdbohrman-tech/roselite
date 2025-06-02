# Reference

Complete reference documentation for Roselite CLI, gateway, and APIs.

## CLI Reference

<div class="grid cards" markdown>

-   :material-console:{ .lg .middle } **CLI Commands**

    ---

    Complete command-line interface documentation

    [:octicons-arrow-right-24: CLI Commands](cli-commands.md)

-   :material-package-variant:{ .lg .middle } **Package Format**

    ---

    Veilid package format specification

    [:octicons-arrow-right-24: Package Format](package-format.md)

</div>

## Gateway Reference

<div class="grid cards" markdown>

-   :material-api:{ .lg .middle } **Gateway API**

    ---

    HTTP gateway API reference

    [:octicons-arrow-right-24: Gateway API](gateway-api.md)

-   :material-cog:{ .lg .middle } **Configuration**

    ---

    Configuration options and troubleshooting

    [:octicons-arrow-right-24: Configuration](configuration.md)

</div>

## Quick Reference

### Common Commands

```bash
# Bundle a site
roselite bundle ./site --name "My Site" --version "1.0.0"

# Publish to DHT
roselite publish my-site.veilidpkg

# Start gateway
roselite-gateway --port 8080 --domain localhost:8080

# Check status
roselite status
```

### Gateway URLs

```
http://[slug].[domain]/[path]
https://my-site.localhost:8080/index.html
https://docs.localhost:8080/guide/
```

### Package Structure

```json
{
  "name": "Site Name",
  "version": "1.0.0",
  "entry": "index.html",
  "files": ["index.html", "style.css", "app.js"]
}
``` 