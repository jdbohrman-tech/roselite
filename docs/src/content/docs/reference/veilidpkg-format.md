---
title: VeilidPkg Format Reference
description: Complete specification for the .veilidpkg package format
---

The `.veilidpkg` format is Roselite's standardized package format for bundling static sites for deployment to the Veilid DHT network.

## Package Structure

A `.veilidpkg` file is a TAR archive containing:

```
my-site.veilidpkg
├── veilid.json       # Package metadata and configuration  
├── index.html        # Static site files stored directly
├── about.html
├── css/
│   └── style.css
├── js/
│   └── main.js
└── images/
    └── logo.png
```

## Manifest Schema

The `veilid.json` file contains all package metadata:

```json
{
  "name": "My Website",
  "slug": "my-website", 
  "version": "1.0.0",
  "description": "My personal portfolio website",
  "developer": "Developer Name",
  "author": "Author Name",
  "category": "website",
  "entry": "index.html",
  "tags": ["portfolio", "react", "typescript"],
  "identity": "veilid_identity_key_here",
  "signature": "cryptographic_signature_here",
  "format_version": "1.0.0",
  "dependencies": [],
  "permissions": [],
  "created_at": "2024-06-02T08:30:00Z",
  "updated_at": "2024-06-02T08:30:00Z",
  "public_key": "public_key_for_verification"
}
```

## Field Reference

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | `string` | Human-readable package name |
| `slug` | `string` | URL-safe identifier (a-z, 0-9, hyphens only) |
| `version` | `string` | Semantic version (e.g., "1.0.0") |
| `description` | `string` | Package description |
| `developer` | `string` | Developer/organization name |
| `author` | `string` | Author name (can be same as developer) |
| `category` | `string` | Content category |
| `entry` | `string` | Main entry file (usually "index.html") |
| `tags` | `string[]` | Search/classification tags |
| `identity` | `string` | Veilid identity key for DHT storage |
| `signature` | `string` | Cryptographic signature for verification |
| `format_version` | `string` | VeilidPkg format version |
| `dependencies` | `string[]` | Package dependencies (currently unused) |
| `permissions` | `Permission[]` | App permissions (currently unused) |
| `created_at` | `string` | ISO 8601 timestamp |
| `updated_at` | `string` | ISO 8601 timestamp |
| `public_key` | `string` | Public key for signature verification |

### Permission Types

Currently defined but not actively used:

- `Network` - Network access permission
- `FileSystem` - File system access permission  
- `Camera` - Camera access permission
- `Microphone` - Microphone access permission
- `Clipboard` - Clipboard access permission

## Slug Requirements

The `slug` field must conform to these rules:

- **Characters**: Only lowercase letters (a-z), numbers (0-9), and hyphens (-)
- **Length**: 3-63 characters
- **Start/End**: Must start and end with alphanumeric character
- **Uniqueness**: Must be unique across the Veilid network
- **Reserved**: Cannot use reserved words: `api`, `www`, `admin`, `root`

### Valid Slug Examples
```
my-site
portfolio-2024
docs-v2
blog
tech-startup
```

### Invalid Slug Examples
```
My-Site          # uppercase letters
my_site          # underscores not allowed
-my-site         # cannot start with hyphen
my-site-         # cannot end with hyphen
my.site          # dots not allowed
api              # reserved word
```

## File Organization

### Direct Storage

Static site files are stored directly in the TAR archive at their relative paths:

```
my-site.veilidpkg
├── veilid.json          # Manifest
├── index.html           # Entry point
├── about.html
├── contact.html
├── css/                 # Subdirectories preserved
│   ├── style.css
│   └── theme.css
├── js/
│   └── main.js
└── images/
    ├── logo.png
    └── banner.jpg
```

### Path Conventions

- **Entry File**: Must exist at the path specified in `entry` field
- **Index Files**: `index.html` files enable directory browsing  
- **Case Sensitivity**: File paths are case-sensitive
- **URL Mapping**: File paths map directly to gateway URLs
- **No Root Directory**: Files stored at archive root (no wrapping directory)

## Cryptographic Security

### Identity and Signatures

Each package includes cryptographic verification:

- **Identity**: Veilid DHT identity key for storage/retrieval
- **Signature**: Cryptographic signature of package content
- **Public Key**: Public key for verifying signatures
- **Verification**: CLI validates signatures before deployment

### Key Generation

The CLI automatically generates:
- Ed25519 keypair for signing
- Veilid-compatible identity for DHT storage
- Cryptographic signatures for package integrity

## Version History

### Format Version 1.0.0 (Current)

- TAR archive with gzip compression
- `veilid.json` manifest with complete metadata schema
- Cryptographic signatures for integrity
- Slug-based addressing for gateway routing
- Direct file storage (no wrapper directories)

## CLI Integration

### Bundle Command

```bash
roselite bundle ./website/ \
  --name "My Site" \
  --version "1.0.0" \
  --description "My personal website" \
  --developer "Developer Name"
```

### Auto-generated Fields

When bundling, the CLI automatically generates:

- `slug`: Generated from name if not provided
- `identity`: Veilid DHT identity key
- `signature`: Cryptographic signature of content
- `public_key`: Public key for verification
- `created_at`: Current timestamp
- `updated_at`: Current timestamp  
- `format_version`: Current VeilidPkg format version

### Validation

The CLI validates packages before deployment:

- ✅ Required fields present
- ✅ Slug format compliance
- ✅ Entry file exists
- ✅ Cryptographic signature valid
- ✅ No invalid characters in paths
- ✅ Manifest schema compliance

## Gateway Integration

### URL Mapping

Gateway servers map VeilidPkg contents to URLs:

```
Package slug: my-site
Gateway domain: localhost:8080

URLs:
https://my-site.localhost:8080/           → index.html
https://my-site.localhost:8080/about.html → about.html
https://my-site.localhost:8080/css/       → css/index.html (if exists)
```

### Content Retrieval

1. Extract slug from subdomain
2. Look up package in Veilid DHT using identity key
3. Extract and serve requested file from TAR archive
4. Set appropriate Content-Type headers

## Best Practices

### Package Organization

1. **Use semantic versioning**: `1.0.0`, `1.1.0`, `2.0.0`
2. **Choose descriptive slugs**: `portfolio-2024` vs `site1`
3. **Organize assets**: Use `css/`, `js/`, `images/` subdirectories
4. **Optimize files**: Minify CSS/JS, optimize images
5. **Test locally**: Verify all links work before packaging

### Performance Optimization

1. **Minimize file count**: Combine CSS/JS files when possible
2. **Compress images**: Use WebP, optimize PNG/JPEG
3. **Remove unused files**: Don't include source files or build artifacts
4. **Use relative paths**: Ensure portability across different domains

### Security Considerations

1. **No sensitive data**: Never include API keys, passwords, or secrets
2. **Client-side only**: VeilidPkg is for static content only
3. **Validate inputs**: Sanitize any user-generated content
4. **HTTPS only**: Always use HTTPS for production deployments

## Examples

### Minimal Package Manifest

```json
{
  "name": "Hello World",
  "slug": "hello-world",
  "version": "1.0.0",
  "description": "Simple hello world site",
  "developer": "Developer",
  "author": "Developer", 
  "category": "general",
  "entry": "index.html",
  "tags": [],
  "identity": "generated_veilid_identity",
  "signature": "generated_signature",
  "format_version": "1.0.0",
  "dependencies": [],
  "permissions": [],
  "created_at": "2024-06-02T08:30:00Z",
  "updated_at": "2024-06-02T08:30:00Z",
  "public_key": "generated_public_key"
}
```