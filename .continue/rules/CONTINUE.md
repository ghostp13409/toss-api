```markdown
# CONTINUE.md

This guide provides an overview of the project structure, development workflow, and key concepts for contributors.

## Project Overview

This is a command-line application built with Rust that provides a terminal user interface (TUI) for interacting with a core business logic engine. The application follows a modular architecture that separates concerns into distinct components: CLI, Core, Engine, and UI.

## Getting Started

### Prerequisites
- Rust 1.70+
- Cargo (included with Rust)
- Modern terminal (supports ANSI escape codes)

### Installation
```
cargo build --release
./target/release/app
```

### Basic Usage
```
./target/release/app <command> [arguments]
```

### Running Tests
```
cargo test
cargo test --release  # for production-quality testing
```

## Project Structure

| Directory | Purpose |
|------|---|
| `cli/` | Command-line interface implementation |
| `core/` | Business logic and shared functionality |
| `engine/` | Main processing engine or business rules |
| `tui/` | Terminal User Interface implementation |
| `main.rs` | Entry point of the application |

## Development Workflow

### Coding Standards
- Follow Rust style guide
- Use `mod` for module organization
- Prefer `pub` where appropriate
- Use `impl` blocks for trait implementations
- Use `struct` for data modeling

### Testing
- Write unit tests in `tests/`
- Write integration tests for the TUI and engine
- Use `cargo test` for local testing
- Use `cargo test --release` for production testing

### Build and Deployment
- Use `cargo build --release` for production builds
- Use `cargo install` for standalone CLI tools
- CI/CD pipeline typically uses GitHub Actions or GitLab CI
- Releases are built and published to crates.io

### Contribution Guidelines
1. Fork the repository
2. Create a new feature branch
3. Make your changes
4. Add tests
5. Commit and push
6. Open a Pull Request

## Key Concepts

### Domain-Specific Terminology
- **CLI**: Command-line interface for handling user commands
- **Core**: Business logic and shared functionality
- **Engine**: Main processing component that executes business rules
- **TUI**: Terminal User Interface for interactive user sessions

### Core Abstractions
- `CommandHandler`: Processes user commands
- `UIState`: Tracks UI components and their state
- `EngineConfig`: Configuration for the processing engine
- `BusinessRule`: Abstract representation of business logic

### Design Patterns
- **Strategy Pattern**: Different command implementations
- **Observer Pattern**: UI components notify when state changes
- **Command Pattern**: Encapsulate command execution

## Common Tasks

### Building the Project
```
cargo build --release
```

### Running Tests
```
cargo test
cargo test --release
```

### Adding a New Feature to the CLI
1. Create a new command in `cli/commands/`
2. Implement the logic in `core/`
3. Add a handler in `cli/handlers/`
4. Update `main.rs` with the new command
5. Write tests for the new functionality

### Implementing a New UI Component
1. Create a new module in `tui/components/`
2. Implement the UI logic
3. Register the component with the UI state
4. Update the main loop to render it
5. Add tests for UI rendering

## Troubleshooting

### Common Issues
- **TUI rendering issues**: Ensure terminal supports ANSI escape codes. Use `TERM=xterm-256color` environment variable.
- **Build errors**: Check `cargo check` output. Common issues: missing dependencies, version conflicts, or incorrect module organization.
- **Command not found**: Ensure CLI is properly registered in `main.rs` and the command handler is correctly implemented.
- **Engine processing errors**: Check `core/` for business logic errors. Use logging or tracing for debugging.

### Debugging Tips
- Use `cargo run --release -- --debug` to run with debug output
- Add `println!` statements for tracing
- Use `cargo audit` to check for security vulnerabilities
- Check `cargo vendor` output for dependency issues

## References

- [Rust Documentation](https://doc.rust-lang.org/)
- [Cargo Documentation](https://doc.rust-lang.org/cargo/)
- [TUI Frameworks](https://docs.rs/crossterm)
- [Rust Style Guide](https://rust-lang.github.io/rfcs/accepted/2022/01/05-style-guide.html)
- [Project-Specific Documentation](https://github.com/your-repo/docs)
```