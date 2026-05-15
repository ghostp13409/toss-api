# Installation & Release Guide

Toss can be installed easily using `cargo`, the Rust package manager.

## 🚀 One-Command Installation

### From Source (Local)
If you have the source code locally, run this from the project root:
```bash
cargo install --path .
```

### From GitHub
Users can install Toss directly from your repository without cloning it manually:
```bash
cargo install --git https://github.com/ghostp13409/toss
```

### From crates.io (Future)
Once you publish Toss to the official Rust registry, anyone can install it with:
```bash
cargo install toss
```

---

## 🛠️ How to Prepare a Release

As the maintainer, follow these steps to release a new version:

### 1. Update Version
Increment the `version` field in `Cargo.toml` (e.g., `0.1.0` -> `0.1.1`).

### 2. Verify Build & Lints
Ensure everything is perfect before shipping:
```bash
cargo check
cargo test
cargo fmt
```

### 3. Tag the Release (Git)
```bash
git tag -a v0.1.1 -m "Release version 0.1.1"
git push origin v0.1.1
```

### 4. Publish to crates.io
If you want to make Toss available to the entire Rust ecosystem:
1. Create an account on [crates.io](https://crates.io).
2. Get an API token from your settings.
3. Run `cargo login <your-token>`.
4. Run `cargo publish`.

---

## 📋 Prerequisites
- **Rust Toolchain**: Users must have Rust installed. They can get it via [rustup.rs](https://rustup.rs/).
- **OpenSSL (Linux)**: Some systems might need OpenSSL headers for `reqwest`.
  - Ubuntu/Debian: `sudo apt install libssl-dev pkg-config`
  - Fedora: `sudo dnf install openssl-devel`
