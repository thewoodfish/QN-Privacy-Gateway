# Contributing to QN Privacy Gateway

Thank you for your interest in contributing to QN Privacy Gateway! We welcome contributions from the community.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/your-username/QN-Privacy-Gateway.git
   cd QN-Privacy-Gateway
   ```
3. **Create a branch** for your feature or fix:
   ```bash
   git checkout -b feature/my-new-feature
   ```

## Development Setup

### Prerequisites

- Rust 1.76 or later
- A QuickNode Solana endpoint (for testing)

### Building

```bash
# Check code
cargo check

# Run tests
cargo test

# Build in release mode
cargo build --release

# Run locally
cp .env.example .env
# Edit .env with your QuickNode URL
cargo run
```

## Code Standards

### Formatting

We use `rustfmt` for code formatting:

```bash
cargo fmt
```

### Linting

We use `clippy` for linting:

```bash
cargo clippy -- -D warnings
```

### Testing

Please add tests for new functionality:

```bash
cargo test
```

## Pull Request Process

1. **Update documentation** if you're changing functionality
2. **Add tests** for new features
3. **Run the full test suite** and ensure it passes
4. **Run `cargo fmt`** and `cargo clippy`
5. **Update CHANGELOG.md** with your changes (if applicable)
6. **Create a pull request** with a clear description of your changes

### PR Guidelines

- Keep PRs focused on a single feature or fix
- Write clear commit messages
- Reference any related issues
- Ensure CI checks pass

## Code Review

All submissions require review. We use GitHub pull requests for this purpose.

## Commit Message Guidelines

Use clear and descriptive commit messages:

```
Add support for custom cache TTL per method

- Allows configuring different TTLs for different RPC methods
- Updates configuration documentation
- Adds tests for new functionality
```

## Feature Requests

Have an idea for a new feature? Open an issue with the `enhancement` label!

## Bug Reports

Found a bug? Please open an issue with:

- Clear description of the bug
- Steps to reproduce
- Expected behavior
- Actual behavior
- Environment details (OS, Rust version, etc.)

## Questions?

Feel free to open an issue with the `question` label.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing!
