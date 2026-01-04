# Contributing to Haptic Harmony Simulation

We love your input! We want to make contributing to Haptic Harmony Simulation as easy and transparent as possible, whether it's:

- Reporting a bug
- Discussing the current state of the code
- Submitting a fix
- Proposing new features
- Becoming a maintainer

## 🚀 Development Process

We use GitHub to host code, to track issues and feature requests, as well as accept pull requests.

### Pull Requests

1. Fork the repo and create your branch from `main`.
2. If you've added code that should be tested, add tests.
3. If you've changed APIs, update the documentation.
4. Ensure the test suite passes.
5. Make sure your code lints.
6. Issue that pull request!

## 🔧 Development Setup

### Prerequisites

- **Rust 1.70+** - [Install Rust](https://rustup.rs/)
- **Node.js 18+** - For Tauri frontend development
- **Git** - Version control

### Local Development

```bash
# Clone your fork
git clone https://github.com/your-username/haptic-harmony-simulation.git
cd haptic-harmony-simulation

# Install development tools
cargo install cargo-watch cargo-tarpaulin cargo-audit

# Run tests
cargo test

# Run with hot reload
cargo watch -x test
```

### Code Style

We use the standard Rust formatting and linting tools:

```bash
# Format code
cargo fmt

# Run clippy lints
cargo clippy -- -D warnings

# Check for security vulnerabilities
cargo audit
```

### Testing

```bash
# Run all tests
cargo test

# Run tests with coverage
cargo tarpaulin --out Html

# Run specific test suites
cargo test --test integration
cargo test connectivity
```

## 📝 Coding Standards

### Rust Code

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Address all `cargo clippy` warnings
- Write comprehensive documentation for public APIs
- Include unit tests for new functionality

### Documentation

- Use clear, concise language
- Include code examples where appropriate
- Update README.md for significant changes
- Document breaking changes in CHANGELOG.md

### Commit Messages

Use conventional commit format:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Adding tests
- `chore`: Maintenance tasks

Examples:
```
feat(ble): add battery level simulation
fix(gui): resolve window sizing issue
docs(readme): update installation instructions
```

## 🐛 Bug Reports

We use GitHub issues to track public bugs. Report a bug by [opening a new issue](https://github.com/haptic-harmony/haptic-harmony-simulation/issues).

**Great Bug Reports** tend to have:

- A quick summary and/or background
- Steps to reproduce
  - Be specific!
  - Give sample code if you can
- What you expected would happen
- What actually happens
- Notes (possibly including why you think this might be happening, or stuff you tried that didn't work)

## 💡 Feature Requests

We welcome feature requests! Please:

1. Check if the feature already exists or is planned
2. Open an issue with the `enhancement` label
3. Describe the feature and its use case
4. Provide examples if possible

## 📋 Issue Labels

- `bug` - Something isn't working
- `enhancement` - New feature or request
- `documentation` - Improvements or additions to documentation
- `good first issue` - Good for newcomers
- `help wanted` - Extra attention is needed
- `question` - Further information is requested

## 🏗️ Architecture Guidelines

### Module Organization

- Keep modules focused and cohesive
- Use clear, descriptive names
- Separate concerns appropriately
- Follow the existing project structure

### Error Handling

- Use `Result<T, E>` for fallible operations
- Create custom error types when appropriate
- Provide helpful error messages
- Use `anyhow` for application errors

### Async Code

- Use `async`/`await` for I/O operations
- Prefer `tokio` for async runtime
- Handle cancellation gracefully
- Avoid blocking operations in async contexts

## 🔒 Security

If you discover a security vulnerability, please send an email to security@haptic-harmony.com instead of opening a public issue.

## 📄 License

By contributing, you agree that your contributions will be licensed under the MIT License.

## 🤝 Code of Conduct

This project adheres to a code of conduct. By participating, you are expected to uphold this code.

### Our Pledge

We pledge to make participation in our project a harassment-free experience for everyone, regardless of age, body size, disability, ethnicity, gender identity and expression, level of experience, nationality, personal appearance, race, religion, or sexual identity and orientation.

### Our Standards

Examples of behavior that contributes to creating a positive environment include:

- Using welcoming and inclusive language
- Being respectful of differing viewpoints and experiences
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

## 🙏 Recognition

Contributors will be recognized in:

- The project README
- Release notes for significant contributions
- The project's contributors page

Thank you for contributing to Haptic Harmony Simulation! 🎉
