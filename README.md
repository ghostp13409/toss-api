# toss-api

A Vim-inspired, high-performance TUI and CLI API client built with Rust and Ratatui.

`toss-api` is designed for developers who want a fast, keyboard-driven workflow for exploring and testing APIs without leaving the terminal. It combines the visual power of an API client with the efficiency of a Vim-like interface.

---

## 🚀 Key Features

- **⚡ Fast & Lightweight**: Built with Rust for near-instant startup and minimal resource usage.
- **⌨️ Vim-Inspired Navigation**: Navigate through collections, requests, and environments using familiar `h/j/k/l` keys.
- **🔍 Smart Project Parsing**: Automatically extract API endpoints directly from your codebase (Express, FastAPI, Spring, etc.).
- **🧪 Advanced REST Client**: Support for Params, Headers, Auth (Bearer, Basic, API Key), and Body (JSON, Form Data, etc.).
- **📂 Collection Management**: Organize requests into folders and subfolders. Import directly from Postman.
- **🔐 Environment Variables**: Context-aware variables with masking support for sensitive data.
- **🛠️ CLI & TUI**: Switch seamlessly between a full TUI dashboard and quick CLI commands.
- **🎨 Visual Polish**: LazyVim-style mode indicators and color-coded HTTP methods.

## 📦 Installation

You can install `toss` using `cargo`, the Rust package manager.

### From Crates.io (Recommended)
```bash
cargo install toss-api
```

### From GitHub
```bash
cargo install --git https://github.com/ghostp13409/toss
```

### From Source
```bash
git clone https://github.com/ghostp13409/toss
cd toss
cargo install --path .
```

*Note: Ensure you have the Rust toolchain installed from [rustup.rs](https://rustup.rs/).*

## 📖 Quickstart

1. **Launch the TUI**: Simply run `toss-api` in your terminal.
2. **Import a Project**: Press `:` to enter command mode and type `:parse .` to extract APIs from the current directory.
3. **Navigate**: Use `j/k` to move, `Enter` to select/edit, and `Tab` to switch panels.
4. **Send a Request**: Press `Ctrl + s` or navigate to the [ Send ] button and press `Enter`.
5. **Help**: Press `?` at any time to see the full list of shortcuts and command mode actions.

## 🏗️ Supported Frameworks (for Smart Parsing)

Toss can intelligently detect and extract endpoints from the following frameworks:
- **Node.js**: Express.js, Next.js
- **Python**: FastAPI, Flask, Django
- **Java/Kotlin**: Spring Boot, Quarkus
- **PHP**: Laravel
- **Ruby**: Ruby on Rails
- **C#**: ASP.NET Core
- **Go**: Standard library / Gin-style patterns

## 🛠️ CLI Mode

For quick actions without entering the UI:
- `toss-api send GET https://api.example.com/users`
- `toss-api run "My Collection" "Get User"`
- `toss-api collections list`
- `toss-api env show "My Collection"`

Run `toss --help` for the full command list.

## 🗺️ Roadmap

- [ ] GraphQL Support (with Schema auto-fetching)
- [ ] Scripting (Pre-request & Post-response JavaScript)
- [ ] Mass API Testing & Results Visualization
- [ ] More Auth Methods (OAuth2, Digest)
- [ ] Export to Postman/Swagger

<!--## 📄 Documentation

- [Keyboard Shortcuts](./docs/shortcuts.md)
- [Feature Checklist](./docs/feature-checklist.md)
- [Installation & Release Guide](./docs/release.md)
- [Architecture Overview](./docs/architecture.md)-->

## ❤️ Tips...

`toss` is and will always be completely free and open source. If you find it useful, consider buying me a coffee!

[![Ko-fi](https://img.shields.io/badge/Ko--fi-F16061?style=for-the-badge&logo=ko-fi&logoColor=white)](https://ko-fi.com/parthgajjar)

---
*Created with ❤️ by [Parth Gajjar](https://github.com/ghostp13409)*
