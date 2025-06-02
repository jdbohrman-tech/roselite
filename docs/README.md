# Roselite Documentation

This directory contains the documentation for Roselite, built with [Material for MkDocs](https://squidfunk.github.io/mkdocs-material/).

## 🚀 Quick Start

### Prerequisites

- Python 3.8 or higher
- pip package manager

### Local Development

1. **Install dependencies**:
   ```bash
   cd docs
   pip install -r requirements.txt
   ```

2. **Start development server**:
   ```bash
   mkdocs serve
   ```

3. **Open in browser**: http://localhost:8000

The documentation will automatically reload when you make changes to any `.md` files.

## 📁 Structure

```
docs/
├── mkdocs.yml              # MkDocs configuration
├── requirements.txt        # Python dependencies
├── README.md              # This file
├── stylesheets/
│   └── extra.css          # Custom CSS styles
├── assets/
│   └── logo.svg           # Roselite logo
└── *.md                   # Documentation pages
```

## 🎨 Customization

### Theme Configuration

The site uses Material for MkDocs with custom styling:

- **Colors**: Pink/rose theme matching Roselite branding
- **Logo**: SVG logo in header
- **Features**: Search, navigation, code highlighting, dark mode
- **Custom CSS**: Additional styling in `stylesheets/extra.css`

### Adding Pages

1. Create a new `.md` file in the `docs/` directory
2. Add it to the navigation in `mkdocs.yml`:
   ```yaml
   nav:
     - New Page: new-page.md
   ```

### Custom Components

The documentation includes custom CSS classes for enhanced styling:

- `.hero-section` - Homepage hero section
- `.feature-grid` - Feature cards layout
- `.feature-card` - Individual feature cards

## 🔧 Building

### Local Build

```bash
# Build static site
mkdocs build

# Build output will be in docs/site/
```

### Production Build

The documentation is automatically built and deployed via GitHub Actions when changes are pushed to the main branch.

## 🚀 Deployment

### Netlify Deployment

This documentation is configured to deploy automatically to Netlify:

1. **GitHub Actions**: Builds the site on every push to main
2. **Netlify Integration**: Deploys the built site to Netlify
3. **Preview Deployments**: Creates preview deployments for pull requests

### Required Secrets

For GitHub Actions deployment, set these repository secrets:

- `NETLIFY_AUTH_TOKEN`: Your Netlify personal access token
- `NETLIFY_SITE_ID`: Your Netlify site ID

### Manual Deployment

To deploy manually:

```bash
# Build the site
mkdocs build

# Deploy to Netlify (using Netlify CLI)
netlify deploy --dir=site --prod
```

## 📝 Writing Guidelines

### Markdown Features

The documentation supports these Markdown extensions:

- **Admonitions**: `!!! note`, `!!! tip`, `!!! warning`
- **Code highlighting**: Syntax highlighting for many languages
- **Tabs**: Tabbed content sections
- **Tables**: Standard Markdown tables
- **Emojis**: GitHub-style emoji shortcuts

### Style Guide

- Use clear, concise language
- Include code examples where applicable
- Use admonitions for important notes
- Follow consistent heading hierarchy
- Include links to related sections

### Example Admonitions

```markdown
!!! note "Information"
    This is an informational note.

!!! tip "Pro Tip"
    This is a helpful tip.

!!! warning "Important"
    This is a warning about something important.
```

## 🐛 Troubleshooting

### Common Issues

#### "Module not found" errors
```bash
pip install --upgrade -r requirements.txt
```

#### Port already in use
```bash
mkdocs serve --dev-addr 127.0.0.1:8001
```

#### Build fails
```bash
# Check for broken links
mkdocs build --strict
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes to the documentation
4. Test locally with `mkdocs serve`
5. Submit a pull request

### Content Guidelines

- Keep documentation up to date with code changes
- Test all code examples
- Use screenshots sparingly (prefer code examples)
- Maintain consistent tone and style
- Update navigation when adding new pages

## 📚 Resources

- [MkDocs Documentation](https://www.mkdocs.org/)
- [Material for MkDocs](https://squidfunk.github.io/mkdocs-material/)
- [Markdown Guide](https://www.markdownguide.org/)
- [Netlify Documentation](https://docs.netlify.com/)

---

*Built with ❤️ using Material for MkDocs* 