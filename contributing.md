Here's a comprehensive `CONTRIBUTING.md` guide based on your requirements:

```markdown
# Contribution Guide

Please take a moment to review these guidelines before submitting a pull request.

## Code Contributions

### Commit Message Format

All commit messages must follow
the [Conventional Commits](https://gist.github.com/qoomon/5dfcdf8eec66a051ecd85625518cfd13) specification. This helps
with automated changelog generation and version management.

Example format:
```
<type>[optional scope]: <description>

[optional body]

[optional footer]
```

### Code Formatting
- **Rust code**: Must be formatted with:
  ```sh
  cargo +nightly fmt --all
  ```
- **JavaScript/TypeScript code**: Must be formatted with:
  ```sh
  npm run format
  ```

### Code Quality
- **Rust code**: Must pass clippy checks:
  ```sh
  cargo clippy --all
  ```
- **JavaScript/TypeScript code**: Must pass linting:
  ```sh
  npm run lint
  ```

### Testing
All contributions must pass existing tests. New features should include appropriate tests.

## Workflow

1. **Feature Approval**: Before beginning work on a new feature, please:
   - Open an issue to discuss the proposed change
   - Wait for approval from maintainers
   - Unapproved features risk being rejected or significantly modified

2. **Pull Requests**:
   - Reference any related issues
   - Ensure all checks pass (formatting, linting, tests)
   - Keep changes focused - prefer multiple small PRs over one large one

## Community Standards

All project interactions (issues, discussions, code reviews, etc.) must:
- Be conducted in good faith, without discrimination or hateful conduct.

The maintainers reserve the right to moderate content and contributions that violate this principle.

Thank you for your contribution!
