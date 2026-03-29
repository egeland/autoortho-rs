# Development Workflow

## Overview

This project uses a branch-based development workflow with automated CI/CD.

## Workflow

### 1. Create Branch
```bash
git checkout main
git pull
git checkout -b feature/my-feature
```

### 2. Develop & Push
Make changes, commit frequently. Push branch:
```bash
git push -u origin feature/my-feature
```

### 3. Open PR
- Target: `main`
- CI automatically runs on PR:
  - **ci.yml**: Format → Clippy → Test (Linux)

### 4. Review & Approve
- Check CI results
- Review code changes
- Approve PR

### 5. Merge
- **Squash merge** to main
- After merge, CI runs:
  - **cross-platform.yml**: Test on Linux/macOS/Windows
  - **security.yml**: cargo-audit + cargo-deny

### 6. Version Bump
- If tests pass, **version.yml** creates a version bump PR
- Review & merge the version PR
- Tag pushed → **release.yml** runs

---

## CI/CD Pipeline

| Workflow | Trigger | Jobs |
|----------|---------|------|
| `ci.yml` | PR | check → test |
| `security.yml` | PR + Main | audit, deny |
| `cross-platform.yml` | Main push | test (matrix) |
| `version.yml` | Main push (after tests) | release-please |
| `release.yml` | Tag (v1.0.0) | build & release |

---

## Testing

Before committing:
```bash
cargo fmt
cargo clippy -- -D warnings
cargo test --lib
```

---

## Code Standards

- Always write unit tests first, run them, see them fail, then write the code
- Always use a linting tool (clippy) before committing
- Always run unit tests before committing
- Use [conventional commits](https://www.conventionalcommits.org/) for commit messages
- When adding dependencies, verify licenses are allowed in `deny.toml`
