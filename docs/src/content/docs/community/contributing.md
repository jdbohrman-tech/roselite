---
title: Contributing
description: How to contribute to the Roselite project
---

We welcome contributions to Roselite! Whether you're fixing bugs, adding features, improving documentation, or helping with community support, your contributions make Roselite better for everyone.

## Ways to Contribute

### 🐛 Report Bugs
Found a bug? Help us fix it by [opening an issue](https://github.com/jdbohrman-tech/roselite/issues/new?template=bug_report.md) with:
- A clear description of the problem
- Steps to reproduce the issue
- Your system information (OS, Rust version, etc.)
- Expected vs. actual behavior

### 💡 Suggest Features
Have an idea for a new feature? [Open a feature request](https://github.com/jdbohrman-tech/roselite/issues/new?template=feature_request.md) including:
- A detailed description of the feature
- Use cases and benefits
- Possible implementation approaches
- Any related work or research

### 📖 Improve Documentation
Documentation improvements are always welcome:
- Fix typos or unclear explanations
- Add examples and tutorials
- Translate documentation
- Create video guides or blog posts

### 💻 Code Contributions
Ready to write some code? Here's how to get started:

## Development Setup

### Prerequisites
- **Rust 1.70+** - [Install Rust](https://rustup.rs/)
- **Git** - [Install Git](https://git-scm.com/)
- **Veilid** (for testing) - [Install Veilid](https://veilid.com/install/)

### Setting Up Your Development Environment

1. **Fork and Clone**
   ```bash
   # Fork the repository on GitHub, then clone your fork
   git clone https://github.com/YOUR_USERNAME/roselite.git
   cd roselite
   ```

2. **Install Dependencies**
   ```bash
   # Install Rust dependencies
   cargo build
   
   # Run tests to verify setup
   cargo test
   ```

3. **Set Up Pre-commit Hooks**
   ```bash
   # Install pre-commit (requires Python)
   pip install pre-commit
   pre-commit install
   ```

### Project Structure

```
roselite/
├── roselite-cli/          # CLI application
│   ├── src/
│   │   ├── main.rs        # CLI entry point
│   │   ├── commands/      # CLI commands
│   │   └── config.rs      # Configuration handling
│   └── Cargo.toml
├── roselite-core/         # Core library
│   ├── src/
│   │   ├── lib.rs         # Library entry point
│   │   ├── dht/           # DHT interface
│   │   ├── compression/   # Content compression
│   │   └── crypto/        # Cryptographic utilities
│   └── Cargo.toml
├── roselite-gateway/      # Gateway server
│   ├── src/
│   │   ├── main.rs        # Gateway entry point
│   │   ├── routes/        # HTTP routes
│   │   └── dht_client.rs  # DHT client
│   └── Cargo.toml
└── docs/                  # Documentation site
```

## Development Workflow

### 1. Create a Branch
```bash
# Create a new branch for your work
git checkout -b feature/your-feature-name
# or
git checkout -b fix/bug-description
```

### 2. Make Your Changes
- Write clean, well-documented code
- Add tests for new functionality
- Update documentation as needed
- Follow our coding standards (see below)

### 3. Test Your Changes
```bash
# Run all tests
cargo test

# Run clippy for linting
cargo clippy -- -D warnings

# Format code
cargo fmt

# Test the CLI locally
cargo run --bin roselite-cli -- --help
```

### 4. Submit a Pull Request
1. Push your branch to your fork
2. Open a pull request on GitHub
3. Fill out the PR template
4. Wait for review and address feedback

## Coding Standards

### Rust Style
- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for consistent formatting
- Fix all `cargo clippy` warnings
- Add documentation comments for public APIs

### Testing
- Write unit tests for all new functions
- Add integration tests for CLI commands
- Include error cases in tests
- Aim for high test coverage

### Documentation
- Document all public APIs with rustdoc
- Include examples in documentation
- Update user guides for new features
- Keep changelog up to date

## Code Review Process

All contributions go through code review:

1. **Automated Checks**: CI runs tests, linting, and security scans
2. **Maintainer Review**: A project maintainer reviews the code
3. **Community Input**: Other contributors may provide feedback
4. **Approval**: Changes are approved and merged

### What We Look For
- **Correctness**: Does the code work as intended?
- **Performance**: Are there any performance implications?
- **Security**: Are there security considerations?
- **Maintainability**: Is the code easy to understand and modify?
- **Testing**: Are there adequate tests?

## Release Process

Roselite follows [Semantic Versioning](https://semver.org/):
- **Major**: Breaking changes
- **Minor**: New features (backward compatible)
- **Patch**: Bug fixes

### Release Schedule
- Patch releases: As needed for critical bugs
- Minor releases: Monthly
- Major releases: Quarterly or as needed

## Community Guidelines

### Code of Conduct
We follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). Please be respectful and inclusive in all interactions.

### Communication
- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and community discussion
- **Discord**: Real-time chat (link in README)

### Getting Help
New to contributing? Here are some good first issues:
- Documentation improvements
- Adding tests
- Fixing typos
- Small bug fixes labeled "good first issue"

## Recognition

Contributors are recognized in:
- The project README
- Release notes
- Annual contributor report

## Legal

By contributing to Roselite, you agree that your contributions will be licensed under the same license as the project (MIT License).

## Questions?

Have questions about contributing? Feel free to:
- Open a discussion on GitHub
- Ask in our Discord community
- Email the maintainers

Thank you for contributing to Roselite! 🚀 