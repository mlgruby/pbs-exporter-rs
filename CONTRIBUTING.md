# Contributing to PBS Exporter

Thank you for your interest in contributing to PBS Exporter! This document outlines our contribution workflow and guidelines.

## Branching Strategy

We follow a **Gitflow-inspired workflow** with two main branches:

- **`main`**: Production-ready releases only
- **`develop`**: Integration branch for features and fixes

### Branch Protection

Both `main` and `develop` are **protected branches**:

- No direct commits allowed
- No force pushes allowed
- All changes must go through Pull Requests
- All CI checks must pass before merging

## Contribution Workflow

### 1. Fork the Repository

Click the "Fork" button on GitHub to create your own copy of the repository.

### 2. Clone Your Fork

```bash
git clone https://github.com/YOUR_USERNAME/pbs-exporter-rs.git
cd pbs-exporter-rs
```

### 3. Add Upstream Remote

```bash
git remote add upstream https://github.com/mlgruby/pbs-exporter-rs.git
```

### 4. Create a Feature Branch

**Always branch from `develop`:**

```bash
git checkout develop
git pull upstream develop
git checkout -b feature/your-feature-name
```

**Branch naming conventions:**

- `feature/` - New features (e.g., `feature/tape-metrics`)
- `fix/` - Bug fixes (e.g., `fix/memory-leak`)
- `docs/` - Documentation updates (e.g., `docs/api-guide`)
- `refactor/` - Code refactoring (e.g., `refactor/client-module`)

### 5. Make Your Changes

- Write clean, well-documented code
- Follow Rust best practices
- Add tests for new functionality
- Update documentation as needed

### 6. Commit Your Changes

Use [Conventional Commits](https://www.conventionalcommits.org/) format:

```bash
git commit -m "feat: add tape drive metrics support"
git commit -m "fix: resolve memory leak in snapshot collection"
git commit -m "docs: update API documentation"
```

**Commit message format:**

```text
<type>: <description>

[optional body]

[optional footer]
```

**Types:**

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

### 7. Push to Your Fork

```bash
git push origin feature/your-feature-name
```

### 8. Create a Pull Request

1. Go to your fork on GitHub
2. Click "New Pull Request"
3. **Set base branch to `develop`** (NOT `main`)
4. Fill out the PR template with:
   - Description of changes
   - Related issue numbers (if applicable)
   - Testing performed
   - Screenshots (if UI changes)

## Pull Request Guidelines

### PR Requirements

- All CI checks must pass
- Code must be formatted (`cargo fmt`)
- No clippy warnings (`cargo clippy`)
- Tests must pass (`cargo test`)
- Docker build must succeed
- Branch must be up-to-date with `develop`

### PR Review Process

1. **Automated Checks**: CI runs automatically
2. **Code Review**: Maintainers review your code
3. **Feedback**: Address any requested changes
4. **Approval**: Once approved, maintainers will merge

## Release Process

### From `develop` to `main`

Only maintainers create release PRs:

1. **Feature Collection**: Multiple features merged to `develop`
2. **Testing**: Thorough testing on `develop` branch
3. **Release PR**: Maintainer creates PR from `develop` → `main`
4. **Version Bump**: Update version in `Cargo.toml`
5. **Changelog**: Update `CHANGELOG.md`
6. **Merge**: After approval, merge to `main`
7. **Tag**: Create version tag (e.g., `v0.2.0`)

## Development Setup

### Prerequisites

- Rust 1.71 or later
- Docker (for container testing)
- Access to a PBS instance (for integration testing)

### Local Development

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/pbs-exporter-rs.git
cd pbs-exporter-rs

# Install dependencies
cargo build

# Run tests
cargo test

# Run locally
cp .env.example .env
# Edit .env with your PBS credentials
cargo run
```

### Running CI Checks Locally

```bash
# Format check
cargo fmt --check

# Linting
cargo clippy --all-targets --all-features -- -D warnings

# Tests
cargo test --all-features

# Build
cargo build --release

# Docker build
docker build -t pbs-exporter:test .
```

## Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Use meaningful variable and function names
- Add comments for complex logic
- Keep functions focused and small
- Write self-documenting code

## Testing

- Add unit tests for new functions
- Add integration tests for API interactions
- Ensure all tests pass before submitting PR
- Aim for high test coverage

## Documentation

- Update README.md if adding new features
- Add inline documentation for public APIs
- Update CHANGELOG.md (maintainers will finalize)
- Include examples for new functionality

## Getting Help

- **Issues**: Check existing issues or create a new one
- **Discussions**: Use GitHub Discussions for questions
- **Discord**: [Coming soon]

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Help others learn and grow
- Follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct)

## License

By contributing, you agree that your contributions will be licensed under the Apache-2.0 License.

---

Thank you for contributing to PBS Exporter!
