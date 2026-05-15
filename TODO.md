# Toss Project TODO

## Phase 1: Foundation & The "Request Engine"
- [x] **Project Scaffolding**: Initialize Rust project with `tokio` async runtime.
- [x] **CLI Argument Parsing**: Implement `clap` for `toss send` and global flags.
- [x] **HTTP Core**: Integrate `reqwest` for `GET`, `POST`, `PUT`, `PATCH`, and `DELETE`.
- [x] **Environment System**: Basic JSON/YAML parser for `{{variable}}` substitution. (Implemented in CLI)
    - [ ] **Integration with TUI**: Use the environment system within the TUI.

## Phase 2: The TUI Skeleton & Layered Navigation
- [x] **Terminal Loop**: Set up `crossterm` and `ratatui` for raw terminal mode and rendering.
- [x] **Layered Layout**: Implement the split screen with panels (Collections, Apis, Properties, Details, Response, Stats).
- [x] **Vim State Machine**: Implement `InputMode` (Normal, Editing, Command, Rename, Search, etc.).
- [x] **Drill-Down Navigation**: Implement logic to shift focus deeper (`Enter`/`l`) or move between panels (`h`/`l`).
- [x] **Command Mode (`:`)**: Build bottom-bar command line for actions like `:q`, `:import`.

## Phase 3: Advanced Source Code Imports (PRIORITY)
- [x] **CLI Subcommand**: Add `toss parse <path>` subcommand.
- [x] **Framework Detection**: Add logic to detect frameworks (Spring, Express, Django, etc.).
- [x] **Endpoint Extraction**: Extract HTTP methods, paths, and body metadata from source files.
    - [x] **Express.js / Next.js** support (with TS model discovery).
    - [x] **FastAPI / Flask** support (with Python model discovery).
    - [x] **Spring Boot / Quarkus** support (with Java/Kotlin model discovery).
    - [x] **ASP.NET** support (with C# model discovery).
    - [x] **Golang** support (with struct discovery).
    - [ ] **Django/Laravel/Rails** basic support.
- [x] **Collection Generation**: Group endpoints into Toss Collections mirroring the project structure.

## Phase 4: Data Management, CRUD & Multi-format Imports
- [x] **Tree Implementation**: Build Collections and APIs panels with nested folder support.
- [x] **CRUD Operations**: Implement `a` (Add), `r` (Rename), and `d` (Delete) functionality.
- [x] **Search & Filter (`/`)**: Add real-time tree filtering for collections and APIs.
- [x] **Multi-format Support**: Implement `toss import <path>`.
    - [x] **Postman**: Import `postman_collection` (JSON).
    - [ ] **Insomnia**: Build parser for Insomnia exports.
    - [ ] **Swagger/OpenAPI**: Build parser for Swagger/OpenAPI specs.
- [x] **Persistence Layer**: Implement local storage (JSON) for history and collections.

## Phase 5: Advanced REST, Highlighting & Editor Integration
- [x] **Properties Implementation**: Add panels for Params, Headers, Auth, Body, and Scripts.
- [ ] **Authentication Suites**:
    - [x] **Bearer Token**
    - [x] **Basic Auth**
    - [x] **API Keys**
    - [ ] **OAuth 1.0/2.0**
- [ ] **Rich Body & Beautification**:
    - [x] **JSON (Raw)**
    - [x] **Form-data**
    - [x] **X-www-form-urlencoded**
    - [ ] **GraphQL**
- [ ] **Syntax Highlighting**: Integrate `syntect` for Response and Body panels.
- [x] **External Editor (`v`)**: Open request body in system `$EDITOR`.
- [x] **Response Stats**: Real-time calculation of response time, size, and protocol.

## Phase 6: CLI Mode, Automation & Scripting
- [ ] **`toss run` Subcommand**: High-performance collection runner.
- [ ] **Scripting Engine**: Integrate a JS runtime (e.g., `deno_core` or `boa`) for pre-request/post-response logic.
- [ ] **Result Reporting**: Summary outputs and exit codes for automated testing pipelines.

## Phase 7: Configuration, Optimization & Polish
- [ ] **Full Configuration System**: Move themes, layouts, and global settings to a YAML/TOML config file.
- [ ] **Dynamic Themes**: Support for terminal colors and custom TUI styling.
- [ ] **Performance Tuning**: Optimize rendering and tree traversal for massive collections.
- [ ] **Cross-Platform Validation**: Ensure consistent behavior across Windows, Linux, and macOS.
