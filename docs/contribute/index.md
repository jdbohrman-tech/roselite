# Contribute to Roselite

Join the community building censorship-resistant hosting infrastructure. Whether you're a developer, designer, writer, or user - there are many ways to contribute!

## Ways to Contribute

### 🔧 **Code Contributions**
- **Bug fixes** and performance improvements
- **New features** and enhancements
- **Platform support** for additional operating systems
- **Integration plugins** for popular static site generators

### 📝 **Documentation**
- **Tutorial writing** and examples
- **API documentation** improvements
- **Translation** to other languages
- **Video tutorials** and guides

### 🧪 **Testing & QA**
- **Bug reporting** with detailed reproduction steps
- **Testing** new releases and features
- **Performance benchmarking**
- **Security vulnerability** research

### 🎨 **Design & UX**
- **CLI user experience** improvements
- **Documentation design** enhancements
- **Logo and branding** assets
- **Gateway interface** design

### 🌍 **Community**
- **Community support** in discussions
- **Content creation** (blogs, talks, demos)
- **Event organization** and participation
- **Advocacy** for decentralized technologies

## Getting Started

### 1. Set Up Development Environment

```bash
# Clone the repository
git clone https://github.com/jdbohrman/roselite.git
cd roselite

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the project
cargo build

# Run tests
cargo test
```

### 2. Explore the Codebase

Roselite is organized into three main crates:

- **`roselite-core/`** - Core DHT integration and package handling
- **`roselite-cli/`** - Command-line interface
- **`roselite-gateway/`** - HTTP gateway server

### 3. Find an Issue

Browse our [GitHub Issues](https://github.com/jdbohrman/roselite/issues) to find something to work on:

- 🟢 **Good First Issue** - Perfect for newcomers
- 🔴 **Bug** - Something that needs fixing
- 💡 **Enhancement** - New features and improvements
- 📚 **Documentation** - Docs that need updating

## Development Guidelines

### Code Standards

- **Rust Style**: Follow `rustfmt` and `clippy` suggestions
- **Comments**: Document public APIs and complex logic
- **Tests**: Add tests for new functionality
- **Error Handling**: Use proper error types and handling

### Testing

```bash
# Run all tests
cargo test

# Run tests with logging
RUST_LOG=debug cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run clippy for linting
cargo clippy

# Format code
cargo fmt
```

### Performance Considerations

- **Memory Usage**: Be mindful of memory allocation in DHT operations
- **Network Efficiency**: Minimize unnecessary network requests
- **Caching**: Implement appropriate caching strategies
- **Async/Await**: Use async patterns for I/O operations

## Contribution Process

### 1. Before You Start

- **Check existing issues** to avoid duplicate work
- **Create an issue** for significant changes
- **Discuss the approach** with maintainers
- **Fork the repository** to your account

### 2. Making Changes

```bash
# Create a feature branch
git checkout -b feature/your-feature-name

# Make your changes
# ... edit files ...

# Test your changes
cargo test
cargo clippy

# Commit your changes
git add .
git commit -m "feat: add new feature description"
```

### 3. Submitting a Pull Request

1. **Push your branch** to your fork
2. **Create a pull request** on GitHub
3. **Fill out the PR template** completely
4. **Wait for review** and address feedback
5. **Celebrate** when it's merged! 🎉

### Pull Request Guidelines

- **Clear title** describing the change
- **Detailed description** of what and why
- **Link to related issues**
- **Include tests** for new functionality
- **Update documentation** if needed

## Areas Needing Help

### High Priority

- **Gateway performance optimization**
- **Better error messages and debugging**
- **Windows compatibility improvements**
- **Mobile device gateway support**

### Medium Priority

- **CI/CD improvements**
- **Integration with popular static site generators**
- **Monitoring and observability features**
- **Alternative DHT implementations**

### Documentation Needs

- **Video tutorials** for getting started
- **Integration guides** for popular frameworks
- **Advanced configuration examples**
- **Troubleshooting guides**

## Community Guidelines

### Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please read our [Code of Conduct](code-of-conduct.md) before participating.

### Communication

- **Be respectful** and constructive in all interactions
- **Help others** learn and contribute
- **Ask questions** - no question is too basic
- **Share knowledge** and experiences

### Getting Help

Need help contributing? Here's where to ask:

- **GitHub Discussions** - General questions and discussions
- **GitHub Issues** - Bug reports and feature requests
- **Discord** - Real-time chat with the community
- **Email** - Direct contact for sensitive issues

## Recognition

### Contributors

All contributors are recognized in:

- **GitHub contributor graph**
- **Project README**
- **Release notes** for significant contributions
- **Annual contributor report**

### Maintainer Path

Regular contributors may be invited to become maintainers with:

- **Commit access** to the repository
- **Influence** on project direction
- **Recognition** as a core team member
- **Responsibility** for guiding the project

## Technical Roadmap

Help us build the future of decentralized hosting:

### Near Term (3-6 months)
- **Performance improvements** for large sites
- **Better caching** strategies
- **Improved error handling** and debugging
- **Windows and macOS** compatibility

### Medium Term (6-12 months)
- **Mobile SDKs** for iOS and Android
- **Alternative gateway** implementations
- **Advanced monitoring** and analytics
- **Plugin system** for extensibility

### Long Term (12+ months)
- **IPFS integration** as alternative DHT
- **Blockchain-based** discovery mechanisms
- **P2P gateway discovery**
- **Decentralized domain** resolution

## Resources for Contributors

### Learning Resources
- **[Veilid Documentation](https://veilid.com)** - Understanding the DHT
- **[Rust Book](https://doc.rust-lang.org/book/)** - Learning Rust
- **[Tokio Guide](https://tokio.rs/tokio/tutorial)** - Async Rust
- **[HTTP/1.1 Specification](https://tools.ietf.org/html/rfc2616)** - Web protocols

### Development Tools
- **[VS Code](https://code.visualstudio.com/)** with Rust Analyzer
- **[Zed](https://zed.dev/)** for Rust development
- **[IntelliJ IDEA](https://www.jetbrains.com/idea/)** with Rust plugin
- **[Wireshark](https://www.wireshark.org/)** for network debugging

## Get Started Today

Ready to contribute? Choose your path:

<div class="grid cards" markdown>

-   :material-code-tags:{ .lg .middle } **Start Coding**

    ---

    Jump into development with our contributor guide

    [:octicons-arrow-right-24: Development Setup](development.md)

-   :material-file-document:{ .lg .middle } **Improve Docs**

    ---

    Help make our documentation even better

    [:octicons-arrow-right-24: Documentation Guide](contributing.md#documentation)

-   :material-bug:{ .lg .middle } **Report Issues**

    ---

    Found a bug? Let us know how to fix it

    [:octicons-arrow-right-24: Bug Reports](https://github.com/jdbohrman/roselite/issues/new)

-   :material-heart:{ .lg .middle } **Join Community**

    ---

    Connect with other contributors and users

    [:octicons-arrow-right-24: GitHub Discussions](https://github.com/jdbohrman/roselite/discussions)

</div>

## Thank You

Every contribution makes Roselite better and helps build a more censorship-resistant internet. Whether you fix a typo, add a feature, or help another user - you're making a difference.

**Together, we're building the infrastructure for a free and open web.** 