# Keyboard Shortcuts & Navigation

Toss uses a **Vim-inspired, Lazydocker-style** control scheme. It is designed for power users who prefer keyboard-first navigation without relying on a mouse.

The application is structured in **Logical Layers**. You navigate *up* and *down* within lists, and *drill down* or *pop up* through layers.

---

## 1. Global Shortcuts

These shortcuts work almost everywhere, unless you are actively typing text in an input field (Editing Mode).

| Key                        | Action                                                                 |
| :------------------------- | :--------------------------------------------------------------------- |
| `Ctrl + Enter`             | **Send Request** (Triggers the request and focuses the Response panel) |
| `q`                        | Quit Application                                                       |
| `?` or `leader + h`        | Toggle Help Menu                                                       |

---

## 2. Layer Navigation (The Core Flow)

The UI is divided into a Left Column (Layer 1) and Right Column (Layers 2-5).

| Key          | Action                                                                       | Why it makes sense                               |
| :----------- | :--------------------------------------------------------------------------- | :----------------------------------------------- |
| `j` / `k`    | Move cursor down/up within the currently focused list.                       | Standard Vim navigation.                         |
| `Enter` / `l`| **Drill Down / Select**. Moves focus from Layer 1 -> Layer 2, Layer 2 -> Layer 3. | `l` goes "Right" (deeper into the hierarchy).    |
| `Esc` / `h`  | **Go Back / Pop Up**. Moves focus from Layer 3 -> Layer 2, Layer 2 -> Layer 1.    | `h` goes "Left" (back out of the hierarchy).     |
| `Tab`        | Cycle focus within the **current layer**. (e.g., Collections ↔ APIs).        | Standard TUI convention for sibling panels.      |

### Example Flow:
1. Start in **Collections** (Layer 1). Press `j`/`k` to find a request.
2. Press `Enter` (or `l`). Focus jumps to **Properties** (Layer 2) on the right.
3. Press `j`/`k` to highlight "Body".
4. Press `Enter` (or `l`). Focus jumps into the **Property Details** (Layer 3) to edit the JSON body.
5. Press `Esc` (or `h`) when done editing to return to **Properties** (Layer 2).
6. Press `Ctrl+Enter` to send. Focus jumps to **Response** (Layer 4).

---

## 3. Context-Specific Shortcuts

### **Layer 1: Collections & APIs Panels**
- `a`: **Add** new Request or Folder. (If a Request is created, focus automatically shifts to the Request Bar).
- `d`: **Delete** selected item (requires confirmation).
- `r`: **Rename** selected item.
- `e`: **Express to Request Bar**. Instantly focus the URL input from the sidebar.
- `/`: **Search/Filter** collections.
- `Space`: Expand/Collapse a folder.

### **Layer 2 & 3: Properties & Details**
- `e`: **Focus Request Bar**. Quickly jump up to edit the URL or Method.
- `Esc` / `h`: Return to previous layer.
- `Ctrl + Enter`: Send Request.

### **Layer 4: Response Panel**
- `j` / `k`: Scroll response body down/up.
- `G`: Scroll to the absolute bottom.
- `gg`: Scroll to the absolute top.
- `y`: **Yank** (Copy) the entire response body to the clipboard.
- `f`: Toggle **Full Screen** for the response view.

---

## 4. Input Modes

Toss has different state modes to prevent conflicts between navigation and typing:

1. **Normal Mode**: The default state. Keys like `j`, `k`, `h`, `l` navigate the UI.
2. **Editing Mode**: Triggered automatically when entering Layer 3 (Property Details) text inputs, or manually via `i`. Keystrokes are captured as text. Press `Esc` to return to Normal Mode.
3. **Command Mode (`:`)**: Pressing `:` opens a bottom command line for quick power actions (e.g., `:set env dev`, `:import ./swagger.json`, `:export openapi api_spec.json`). Press `Enter` to execute or `Esc` to cancel.
