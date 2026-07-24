# Design Spec: Script View & TUI Navigation / Stats Improvements

## Problem Statement
1. **Navigation Inconsistency**: Currently, accessing the **Scripts** section requires pressing `s` (lowercase), while toggling console logs requires `Shift + S` (`'S'`). This breaks consistency with the rest of the application, where panel/tab shortcuts are uppercase letters (`C`, `R`, `P`, `H`, `U`, `B`, `E`, `T`, `V`, `A`).
2. **Script View Aesthetics & Editing**: The **Scripts** section currently renders an always-active `edtui` editor widget without support for external editor launching (`'v'`), syntax-highlighted read view, or status banners matching **Body Raw**.
3. **Stats Tab Labels**: In the **Stats** section tab bar, the initial label for **Payload** is rendered as `L` instead of `P`, and **Tests** is rendered as `E` instead of `T`.

## Key Changes

### 1. Keybindings & Navigation Alignment
- Re-bind `Shift + S` (`'S'`) in normal input mode to switch to `PropertyTab::Scripts` (`app.selected_property_tab = PropertyTab::Scripts`, `app.focused_panel = FocusedPanel::Details`).
- Re-bind `Shift + L` (`'L'`) in normal input mode to toggle `app.show_console`.
- Update popup help text in `popups.rs` and console overlay title to reference `'S'` for Scripts and `'L'` (or `:console`) for Console logs.

### 2. Script View & Editor Parity with Body Raw
- **View Mode**:
  - Render pre-request or post-response script content using standard syntax highlighting (`highlight_content(&script, Some("js"))`) and environment variable highlight (`apply_env_vars`).
  - Title banner: `" Scripts: Pre-Request [js] (Press 'i' for inline edit, 'v' for external) "` or `" Scripts: Post-Response [js] (Press 'i' for inline edit, 'v' for external) "`.
  - Display subtab selector at the top (`[Pre-Request]  Post-Response` or ` Pre-Request  [Post-Response]`), changeable via `'t'`.
- **Inline Editing (`'i'`)**:
  - When `'i'` is pressed while on `PropertyTab::Scripts`, set `app.input_mode = InputMode::BodyEditor` (or `ScriptEditor`) and populate `app.pre_request_editor_state` / `app.post_response_editor_state`.
  - Pressing `Esc` in normal mode within the editor saves the contents back to `req.pre_request_script` or `req.post_response_script` and returns `app.input_mode` to `Normal`.
- **External Editor (`'v'`)**:
  - When `'v'` is pressed while on `PropertyTab::Scripts`, dispatch `TuiAction::EditScript`.
  - In `src/tui/mod.rs`, handle `TuiAction::EditScript` by creating a temp file (`toss_script_pre_{req_id}.js` or `toss_script_post_{req_id}.js`), populating it with current script content, invoking `$EDITOR` (or `nvim`/`vi`), and updating the request script field upon exit.

### 3. Stats Section Initials Fix
- In `src/tui/ui/widgets/details.rs`:
  - Change `("L", StatsTab::Payload)` to `("P", StatsTab::Payload)`.
  - Change `("E", StatsTab::Tests)` to `("T", StatsTab::Tests)`.

## Verification Plan
1. `cargo check` & `cargo test` to verify zero compilation or unit test regressions.
2. Manually test keybindings (`S` for Scripts, `L` for Console Logs, `i` and `v` for script editing).
3. Verify Stats section displays `P` for Payload and `T` for Tests.
