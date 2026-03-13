# GitHub Actions Setup

This repository now has automated CI/CD via GitHub Actions!

## What's Automated

### CI (Continuous Integration)
Runs on every push and pull request to `master` or `main`:
- ✅ Checks code formatting with `cargo fmt`
- ✅ Lints code with `cargo clippy`
- ✅ Runs all tests with `cargo test`
- ✅ Builds the project

### CD (Continuous Deployment)
Runs when you push a version tag (e.g., `v1.1.4`):
- 📦 Builds release binary with optimizations
- 🚀 Creates GitHub Release automatically
- 📤 Uploads `ruin-injector.exe` to the release

Caching is enabled for faster builds.

## How to Create a New Release

### 1. Update Version
Edit `Cargo.toml` and update the version number:
```toml
[package]
name = "ruin-injector"
version = "1.1.4"  # Update this
```

### 2. Commit Changes
```bash
git add Cargo.toml
git commit -m "Bump version to 1.1.4"
git push origin master
```

### 3. Create and Push Tag
```bash
git tag v1.1.4
git push origin v1.1.4
```

### 4. Watch GitHub Actions
Go to the **Actions** tab in GitHub to watch the release workflow run. Once complete:
- New release will be created automatically
- Binary will be uploaded as an asset
- You can add custom release notes if needed

## Workflow Files

- `.github/workflows/ci.yml` - CI testing on every push/PR
- `.github/workflows/release.yml` - Automated release on version tags

## Example: Next Release

```bash
# Update Cargo.toml to version "1.1.4"
git commit -am "Bump version to 1.1.4" && git push

# Create and push tag - GitHub Actions will handle the rest!
git tag v1.1.4 && git push origin v1.1.4
```

That's it! No manual building, no manual uploads. GitHub does everything for you. 🎉
