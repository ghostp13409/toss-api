# Release Guide for Toss-API

This project uses [cargo-dist](https://opensource.axo.dev/cargo-dist/) to automate building binaries and creating GitHub Releases.

## 🚀 How to Create a Release

Releases are triggered by **Git Tags**. Follow these steps to release a new version:

### 1. Update the Version
In `Cargo.toml`, update the `version` field to your new version (e.g., `0.1.3`):
```toml
[package]
name = "toss-api"
version = "0.1.3"
```

### 2. Commit and Merge
Commit your version change and merge it into the `main` branch.
```bash
git add Cargo.toml
git commit -m "chore: bump version to 0.1.3"
git push origin main
```

### 3. Tag and Push
Create a git tag and push it. This is what triggers the GitHub Action to build and upload binaries.
```bash
git tag v0.1.3
git push origin v0.1.3
```

---

## 🛠 Target Platforms
The platforms supported for distribution are defined in `dist-workspace.toml`. Current targets include:

- **macOS (Apple Silicon)**: `aarch64-apple-darwin`
- **macOS (Intel)**: `x86_64-apple-darwin`
- **Linux (64-bit)**: `x86_64-unknown-linux-gnu`
- **Linux (ARM64)**: `aarch64-unknown-linux-gnu`
- **Windows (64-bit)**: `x86_64-pc-windows-msvc`

To add more targets, update the `targets` array in `dist-workspace.toml`.

---

## 🏗 CI/CD Workflow
- **Pull Requests**: Every PR triggers a "Release Plan" check. This ensures that the code still compiles and is ready for distribution, but it **does not** create a release.
- **Tag Pushes**: Pushing a tag matching `v*.*.*` (e.g., `v1.2.3`) triggers the full release pipeline:
    1. **cargo-dist**: Builds binaries for all targets, generates shell/powershell installers, creates a GitHub Release, and uploads artifacts.
    2. **Ecosystem Releases** (`release-ecosystems.yml`): Automatically publishes to **crates.io**, **AUR**, **WinGet**, and **Snap Store** upon release publication.

---

## 🔑 Required Repository Secrets

To activate automated deployment across all platforms, configure the following secrets in GitHub Repository Settings (**Settings -> Secrets and variables -> Actions**):

| Platform | Secret Name | Description |
| :--- | :--- | :--- |
| **crates.io** | `CARGO_REGISTRY_TOKEN` | API Token generated from [crates.io/me](https://crates.io/me) |
| **AUR** | `AUR_SSH_KEY` | Private SSH key paired with public key on [aur.archlinux.org](https://aur.archlinux.org) |
| **WinGet** | `WINGET_TOKEN` | GitHub Personal Access Token (PAT) with `repo` permissions to submit PR to `microsoft/winget-pkgs` |
| **Snap Store** | `SNAPCRAFT_STORE_CREDENTIALS` | Token generated via `snapcraft export-login` |

---

## 📦 Distribution Channels & Installation

Users can install `toss-api` across platforms:

- **crates.io**: `cargo install toss-api`
- **Arch Linux (AUR)**: `yay -S toss-bin`
- **Windows (WinGet)**: `winget install toss-api`
- **Linux (Snap)**: `snap install toss-api`
- **Linux/macOS Shell Script**: `curl --proto '=https' --tlsv1.2 -LsSf https://github.com/ghostp13409/toss-api/releases/latest/download/toss-api-installer.sh | sh`
- **Windows PowerShell Script**: `powershell -c "irm https://github.com/ghostp13409/toss-api/releases/latest/download/toss-api-installer.ps1 | iex"`

