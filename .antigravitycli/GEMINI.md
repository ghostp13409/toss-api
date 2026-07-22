# Toss-API Project Guidelines

## UI & Navigation Design
- **Lazydocker-inspired**: The TUI must visually resemble lazydocker (Left sidebar for lists, Right main area for context/details).
- **Drill-Down Layers**: Navigation must follow the logical layers (Collections -> Properties -> Details).
- **Shortcuts**: Use `Enter`/`l` to drill down, and `Esc`/`h` to pop up. Avoid treating all panels as equal peers cycleable by `Tab` only.

## Architecture & Structure
- Rigorously adhere to the modular architecture defined in `docs/architecture.md`.
- Ensure a strict separation of concerns between `engine`, `core`, `cli`, and `tui`.
- Do not add business logic to `main.rs`; use it only as a thin entry point for dispatching.

## Testing Protocol
- After each testable implementation, do not perform the verification tests yourself.
- Instead, provide the user with clear, step-by-step instructions (commands and expected outcomes) to test the implementation manually.
- Wait for user confirmation or feedback before proceeding to the next task.

## Maintain Code Quality
- Make sure to create appropriate files and structure the codebase. 
- Don't always add code into one single file; break it down into smaller, manageable pieces when applicable.
- Make sure to write clear, self-explanatory code.
- code structure should follow the best practices of the Rust ecosystem.
- If a file contains 400-500+ lines of code, it's probably better to refactor it into smaller, more manageable pieces.
