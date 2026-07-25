# Complete Multi-Platform Release Setup Guide for Toss-API

This guide provides step-by-step instructions for registering accounts, generating tokens/SSH keys, and setting up repository secrets to enable the fully autonomous release pipeline for **`toss-api`**.

Once set up, pushing a version tag (e.g., `v0.1.6`) automatically builds and publishes to:
- **GitHub Releases** (Pre-compiled binaries & installers)
- **crates.io** (Cargo registry)
- **AUR** (Arch User Repository)
- **WinGet** (Windows Package Manager)
- **Snap Store** (Linux Snap)
- **Homebrew** (macOS/Linux package installer)

---

## 📋 Overview of GitHub Repository Secrets Needed

Go to your repository on GitHub:
**Settings -> Secrets and variables -> Actions -> Repository secrets -> New repository secret**

Add the following 4 secrets:

| Secret Name | Platform | Description |
| :--- | :--- | :--- |
| `CARGO_REGISTRY_TOKEN` | crates.io | Token for publishing Rust crates |
| `AUR_SSH_KEY` | Arch Linux AUR | Private SSH Key with access to AUR git repo |
| `WINGET_TOKEN` | Windows WinGet | GitHub PAT (Personal Access Token) with `repo` scope |
| `SNAPCRAFT_STORE_CREDENTIALS` | Linux Snap Store | Exported login credentials for Snapcraft |

---

## 1. crates.io Setup (Rust Package Registry)

### Steps:
1. Open [https://crates.io](https://crates.io) and log in using your GitHub account.
2. Click your profile icon at the top right -> **Account Settings**.
3. Navigate to **API Tokens**.
4. Click **New Token**.
   - **Name**: `github-actions-toss`
   - **Scopes**: Select `publish-update` and `publish-new` (or check all publish options).
5. Click **Generate** and copy the secret token immediately.
6. Open your GitHub Repository -> **Settings** -> **Secrets and variables** -> **Actions**.
7. Click **New repository secret**:
   - **Name**: `CARGO_REGISTRY_TOKEN`
   - **Secret**: *(paste token from crates.io)*

---

## 2. AUR Setup (Arch User Repository)

The workflow automatically clones your package repo `aur@aur.archlinux.org:toss-bin.git` (or `toss.git`), updates `PKGBUILD` and `.SRCINFO`, and pushes updates.

### Steps:
1. Open a terminal on your computer and generate a dedicated SSH keypair (leave passphrase empty for CI):
   ```bash
   ssh-keygen -t ed25519 -C "aur-ci@toss-api" -f ~/.ssh/aur_ci_key -N ""
   ```
2. Display and copy your **Public Key**:
   ```bash
   cat ~/.ssh/aur_ci_key.pub
   ```
3. Log in to your account on [https://aur.archlinux.org](https://aur.archlinux.org).
4. Click **My Account** (top right) -> Edit account details.
5. Paste the public key into the **SSH Public Key** field and save.
6. Display and copy your **Private Key**:
   ```bash
   cat ~/.ssh/aur_ci_key
   ```
7. Open your GitHub Repository -> **Settings** -> **Secrets and variables** -> **Actions**.
8. Click **New repository secret**:
   - **Name**: `AUR_SSH_KEY`
   - **Secret**: *(paste entire content of `~/.ssh/aur_ci_key` including BEGIN/END header lines)*

---

## 3. WinGet Setup (Windows Package Manager)

WinGet packages live in Microsoft's repository (`microsoft/winget-pkgs`). The pipeline creates automated Pull Requests to update `toss-api` whenever a new release is published.

### Steps:
1. Open GitHub and go to **Settings** -> **Developer Settings** (bottom of left sidebar).
2. Go to **Personal Access Tokens** -> **Tokens (classic)**.
3. Click **Generate new token (classic)**.
   - **Note**: `winget-releaser-toss-api`
   - **Expiration**: Select `No expiration` (or 1 year)
   - **Scopes**: Check **`repo`** (Full control of private repositories & pull requests).
4. Click **Generate token** and copy it.
5. Open your GitHub Repository -> **Settings** -> **Secrets and variables** -> **Actions**.
6. Click **New repository secret**:
   - **Name**: `WINGET_TOKEN`
   - **Secret**: *(paste your GitHub PAT token)*

---

## 4. Snap Store Setup (Linux Universal Snap)

Allows Linux users on Ubuntu, Debian, Fedora, Arch, etc., to install via `snap install toss-api`.

### Steps:
1. Register/log in to [https://snapcraft.io](https://snapcraft.io).
2. Install `snapcraft` on your computer if not already installed:
   ```bash
   sudo snap install snapcraft --classic
   ```
3. Log in to your Snapcraft account in terminal:
   ```bash
   snapcraft login
   ```
4. Register the package name `toss-api`:
   ```bash
   snapcraft register toss-api
   ```
5. Export your login credentials to a file for GitHub Actions:
   ```bash
   snapcraft export-login --snaps toss-api --acl package_access,package_push,package_update,package_release snapcraft.login
   ```
6. Display and copy the credentials file content:
   ```bash
   cat snapcraft.login
   ```
7. Open your GitHub Repository -> **Settings** -> **Secrets and variables** -> **Actions**.
8. Click **New repository secret**:
   - **Name**: `SNAPCRAFT_STORE_CREDENTIALS`
   - **Secret**: *(paste entire content of `snapcraft.login`)*
9. Securely delete the local credentials file:
   ```bash
   rm snapcraft.login
   ```

---

## 🚀 How to Execute a Release

Once all secrets are set up in GitHub:

```bash
# 1. Update version number in Cargo.toml (e.g. 0.1.6)
# 2. Commit and push to main branch
git add Cargo.toml
git commit -m "chore: bump version to 0.1.9"
git push origin main

# 3. Create and push tag
git tag v0.1.9
git push origin v0.1.9
```

### What Happens Automatically:
1. **`cargo-dist`** runs:
   - Builds 5 platform binaries (Linux x86_64/aarch64, macOS x86_64/aarch64, Windows x86_64).
   - Generates shell & powershell installer scripts.
   - Publishes GitHub Release.
2. **`release-ecosystems.yml`** runs:
   - Runs `cargo publish` to **crates.io**.
   - Updates `PKGBUILD` and `.SRCINFO` on **AUR**.
   - Submits manifest update PR to **WinGet**.
   - Builds and uploads Snap package to **Snap Store**.
