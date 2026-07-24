# Script View & TUI Navigation / Stats Improvements Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix navigation keybindings (`S` for Scripts, `L` for Console Logs), update the Scripts tab in the TUI to support formatted read view, inline editor (`i`), and external editor (`v`) matching Body Raw, and correct the Stats section initials for Payload (`P`) and Tests (`T`).

**Architecture:** Update TUI keyhandlers in `normal.rs` and `popups.rs`, update `TuiAction` in `enums.rs`, implement `TuiAction::EditScript` handling in `mod.rs`, update `details.rs` for rendering Scripts view/edit modes and Stats tab indicators.

**Tech Stack:** Rust, Ratatui, Crossterm, Edtui.

## Global Constraints

- Navigation keys for panel/tab direct access must be uppercase (`Shift` + letter).
- Scripts section must support both inline (`'i'`) and external (`'v'`) editing.
- Payload tab initial must be `P`, Tests tab initial must be `T`.

---

### Task 1: Correct Stats Section Tab Initials

**Files:**
- Modify: `src/tui/ui/widgets/details.rs:109-116`

- [ ] **Step 1: Update tab initials in `details.rs`**

In `src/tui/ui/widgets/details.rs`, change:
```rust
    let tabs = [
        ("O", StatsTab::Overview),
        ("N", StatsTab::Network),
        ("L", StatsTab::Payload),
        ("S", StatsTab::Security),
        ("E", StatsTab::Tests),
    ];
```
to:
```rust
    let tabs = [
        ("O", StatsTab::Overview),
        ("N", StatsTab::Network),
        ("P", StatsTab::Payload),
        ("S", StatsTab::Security),
        ("T", StatsTab::Tests),
    ];
```

- [ ] **Step 2: Run `cargo check` to verify compilation**

Run: `cargo check`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/tui/ui/widgets/details.rs
git commit -m "fix(tui): correct stats section tab initials for payload and tests"
```

---

### Task 2: Align Keybindings for Navigation (`S` for Scripts, `L` for Console Logs)

**Files:**
- Modify: `src/tui/input/normal.rs:35-43`
- Modify: `src/tui/input/popups.rs:26-28`, `src/tui/input/popups.rs:179-185`
- Modify: `src/tui/ui/widgets/popups.rs:192`, `218`, `272-274`, `434`

- [ ] **Step 1: Update `normal.rs` key handlers**

In `src/tui/input/normal.rs`:
Replace lines 35-43:
```rust
        KeyCode::Char('S') => {
            if app.current_request_id.is_some() {
                app.selected_property_tab = PropertyTab::Scripts;
                app.focused_panel = FocusedPanel::Details;
            }
        }
        KeyCode::Char('L') => {
            app.show_console = !app.show_console;
        }
```

- [ ] **Step 2: Update help popup text & console overlay messages in `popups.rs` and `widgets/popups.rs`**

In `src/tui/ui/widgets/popups.rs`:
Change line 272-273:
```rust
            Span::styled("  B / S / T ", Style::default().fg(Color::Cyan)),
            Span::raw(": Body / Scripts / Stats"),
```
And line 192, 219, 434:
Update console hints from `'S'` to `'L'`.

- [ ] **Step 3: Run `cargo check` & `cargo test`**

Run: `cargo check && cargo test`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/tui/input/normal.rs src/tui/input/popups.rs src/tui/ui/widgets/popups.rs
git commit -m "fix(tui): align navigation shortcut S for Scripts and L for Console Logs"
```

---

### Task 3: Support Formatted View, Inline Edit (`i`), and External Edit (`v`) for Scripts Section

**Files:**
- Modify: `src/tui/app/enums.rs:66-76`
- Modify: `src/tui/input/normal.rs:676-682`, `775-793`
- Modify: `src/tui/ui/widgets/details.rs:745-778`
- Modify: `src/tui/mod.rs:327-415`

- [ ] **Step 1: Add `TuiAction::EditScript` to `enums.rs`**

In `src/tui/app/enums.rs`, add `EditScript` to `TuiAction`:
```rust
pub enum TuiAction {
    SendRequest,
    EditBody,
    EditScript,
    CopyBody,
    // ...
}
```

- [ ] **Step 2: Update `'v'` and `'i'` input handling in `normal.rs`**

In `src/tui/input/normal.rs`:
Update `'v'` handler:
```rust
        KeyCode::Char('v') => {
            if app.focused_panel == FocusedPanel::Details {
                if app.selected_property_tab == PropertyTab::Body {
                    app.pending_actions.push(TuiAction::EditBody);
                } else if app.selected_property_tab == PropertyTab::Scripts {
                    app.pending_actions.push(TuiAction::EditScript);
                }
            }
        }
```

- [ ] **Step 3: Update `TuiAction::EditScript` handler in `src/tui/mod.rs`**

Handle `TuiAction::EditScript` similarly to `TuiAction::EditBody`, saving back to `req.pre_request_script` or `req.post_response_script` depending on `app.scripts_subtab`.

- [ ] **Step 4: Update `details.rs` to render formatted script view when not editing, and inline editor when editing**

In `src/tui/ui/widgets/details.rs`:
Check `app.input_mode == InputMode::BodyEditor` when on `PropertyTab::Scripts`.
If not editing, display highlighted script content with `(Press 'i' for inline edit, 'v' for external)`.
If editing, render `edtui::EditorView`.

- [ ] **Step 5: Run `cargo test` to verify everything compiles and passes**

Run: `cargo test`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/tui/app/enums.rs src/tui/input/normal.rs src/tui/ui/widgets/details.rs src/tui/mod.rs
git commit -m "feat(tui): implement formatted view, inline editor, and external editor for scripts"
```
