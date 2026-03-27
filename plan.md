# MDViewer — Lightweight Markdown Viewer for Linux

A minimal, read-only, CLI-first markdown viewer. Opens a local `.md` file (or stdin) in a native Linux window and renders it attractively. No editing, no project browser, no network fetches. Keyboard-driven, starts fast, works offline.

**Success criterion:** `mdview README.md` feels as immediate as `eog image.jpg`.

## Stack

| Component | Role |
|-----------|------|
| **Rust** | Language |
| **GTK4** | Windowing |
| **WebKitGTK** | HTML rendering (scroll, zoom, selection for free) |
| **Comrak** | GFM-compatible Markdown → HTML (tables, task lists, etc.) |
| **syntect** | Server-side syntax highlighting for fenced code blocks |
| **clap** | CLI argument parsing |
| **serde + serde_json** | Window state persistence |
| **dirs** | XDG config directory resolution |

## Architecture

```
CLI args / stdin
      │
      ▼
  File ingestion (read UTF-8, resolve base dir)
      │
      ▼
  Comrak + syntect (Markdown → highlighted HTML)
      │
      ▼
  HTML template (embedded CSS + rendered body)
      │
      ▼
  WebKitWebView (hardened: no JS, no remote assets)
      │
      ▼
  GIO file monitor → re-render on change
```

### Module overview

```rust
// src/main.rs — single-file for now, split when it grows

fn main()               // CLI parsing, GTK app setup
fn build_ui()           // window, webview, keybindings, file monitor
fn md_to_html_doc()     // comrak + syntect → full HTML document
fn reload_content()     // re-read file, re-render, reload webview
fn read_markdown()      // file or stdin ingestion
fn is_dark_mode()       // GTK theme detection
fn load_window_state()  // ~/.config/mdview/state.json
fn save_window_state()  // persist on window close
```

## UX

- `mdview file.md` — open file
- `cat file.md | mdview` — read from stdin
- `--width` / `--height` — control window size
- Window size saved/restored between sessions (`~/.config/mdview/state.json`)
- `q` or `Esc` — quit
- `r` — manual reload
- `Ctrl++` / `Ctrl+-` / `Ctrl+0` — zoom
- External links open in default browser
- Window title = filename (or "stdin")
- No sidebars, no file trees, no workspace, no file-open dialog
- No mouse required

## Security

- JavaScript disabled via `WebKitSettings`
- No remote asset fetches
- Local file paths only, resolved relative to document base directory
- Strict base-URI handling for WebKit `load_html()`

## Rendering features

### Supported

- [x] Headings (h1–h6, with bottom borders on h1/h2)
- [x] Paragraphs, emphasis, strong, strikethrough
- [x] Inline code and fenced code blocks
- [x] **Syntax highlighting** via syntect (language-aware, inline styles)
- [x] Links (external → default browser)
- [x] Bullet and ordered lists
- [x] Task lists (checkboxes)
- [x] Blockquotes
- [x] Tables (GFM-style)
- [x] Horizontal rules
- [x] Images from local relative paths
- [x] Dark/light theme (auto-detected from GTK theme)

### Deferred

- [ ] Math (KaTeX/MathJax)
- [ ] Mermaid diagrams
- [ ] Footnotes
- [ ] Raw HTML passthrough
- [ ] Search within document
- [ ] Print/export to PDF
- [ ] Selection-preserving reload
- [ ] Tabs/multi-document UI

## Milestones

### ~~Milestone 1 — One-evening prototype~~ ✓

- Read file or stdin
- Convert Markdown → HTML with Comrak
- Show in WebKitGTK window
- `q` / `Esc` to quit

### ~~Milestone 2 — Proper viewer~~ ✓

- CSS theme (typography, code blocks, max-width, spacing)
- Local image support
- External links open in browser
- Keyboard zoom (`Ctrl++` / `Ctrl+-` / `Ctrl+0`)
- Live reload on file save (GIO file monitor)
- `r` for manual reload
- `--width` / `--height` CLI flags

### ~~Milestone 3 — Daily-driver quality~~ ✓

- Save/restore window size between sessions
- Dark/light theme (respect system GTK theme)
- Syntax highlighting for code blocks (syntect)
- Better error handling (missing file, bad UTF-8, reload failures)

### Milestone 4 — Snap packaging

#### Strategy

1. **Local snap** — build and test with `--devmode`, then `--strict` confinement
2. **Unlisted snap** — publish to the Snap Store as unlisted for private testing
3. **Public snap** — switch to public after confidence in quality

#### Confinement

Target: **`strict`** confinement (no `--classic`).

Required interfaces/plugs:
- `desktop` / `desktop-legacy` — GTK4 windowing
- `wayland` / `x11` — display access
- `opengl` — WebKitGTK rendering
- `home` — read markdown files from `$HOME`
- `browser-support` — open external links via `xdg-open`

#### Build plan

- Use `snapcraft` with the **`rust`** plugin
- Base: `core24`
- Build dependencies: `libgtk-4-dev`, `libwebkitgtk-6.0-dev`
- Stage packages: `libgtk-4-1`, `libwebkitgtk-6.0-4` (runtime shared libs)
- GNOME extension (`gnome`) for GTK4/WebKit runtime snaps (shared content snap, smaller package)

#### `snapcraft.yaml` outline

```yaml
name: mdview
base: core24
version: '0.1.0'
summary: Lightweight markdown viewer for Linux
description: |
  A minimal, keyboard-driven markdown viewer.
  Renders GFM-compatible markdown with syntax highlighting.
  No editing, no bloat — just clean reading.
grade: devel          # → stable when ready for public
confinement: strict

apps:
  mdview:
    command: bin/mdview
    extensions: [gnome]
    plugs:
      - home
      - browser-support

parts:
  mdview:
    plugin: rust
    source: .
    build-packages:
      - libgtk-4-dev
      - libwebkitgtk-6.0-dev
    stage-packages:
      - libwebkitgtk-6.0-4
```

## What to avoid

- Custom text layout engine
- Native rendering of every Markdown node
- WYSIWYG / editor architecture
- Plugin system
- Notebook / workspace model
- File-open dialog or any GUI chrome
- Browser-like multiprocess features beyond what WebKit provides
- `--classic` confinement (strict is achievable for this app)
