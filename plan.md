# MDViewer — Lightweight Markdown Viewer for Linux

A minimal, read-only, CLI-first markdown viewer. Opens a local `.md` file (or stdin) in a native Linux window and renders it attractively. No editing, no project browser, no network fetches. Keyboard-driven, starts fast, works offline.

**Success criterion:** `mdview README.md` feels as immediate as `eog image.jpg`.

## Stack

- **Rust**
- **GTK4** — windowing
- **WebKitGTK** — HTML rendering (scroll, zoom, selection for free)
- **Comrak** — GFM-compatible Markdown → HTML (tables, task lists, etc.)
- **clap** — CLI argument parsing
- Single bundled CSS theme (embedded at build time)

## Architecture

```
CLI args / stdin
      │
      ▼
  File ingestion (read UTF-8, resolve base dir)
      │
      ▼
  Comrak (Markdown → HTML)
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

## UX

- `mdview file.md` — open file
- `cat file.md | mdview` — read from stdin
- `--width` / `--height` — control window size
- Window size saved/restored between sessions
- `q` or `Esc` — quit
- `r` — manual reload
- `Ctrl++` / `Ctrl+-` / `Ctrl+0` — zoom
- External links open in default browser
- Window title = filename (or "stdin")
- No sidebars, no file trees, no workspace, no file-open dialog
- No mouse required

## Security

- JavaScript disabled via WebKitSettings
- No remote asset fetches
- Local file paths only, resolved relative to document base directory
- Strict base-URI handling for WebKit `load_html()`

## v0.1 Feature Cut

**Ship:**
- Headings, paragraphs, emphasis/strong
- Inline code, fenced code blocks
- Links, bullet/ordered lists, blockquotes
- Horizontal rules
- Images from local relative paths

**Defer:**
- Tables, task lists, footnotes
- Math, Mermaid
- Raw HTML passthrough
- Search, print/export
- Selection-preserving reload
- Tabs/multi-document UI

## Milestones

### Milestone 1 — One-evening prototype
- Read file or stdin
- Convert Markdown → HTML with Comrak
- Show in WebKitGTK window
- `q` / `Esc` to quit
- No theme, no images, no reload

### Milestone 2 — Proper viewer
- CSS theme (typography, code blocks, max-width, spacing)
- Local image support
- External links open in browser
- Keyboard zoom (`Ctrl++` / `Ctrl+-` / `Ctrl+0`)
- Live reload on file save (GIO file monitor)
- `r` for manual reload
- `--width` / `--height` CLI flags

### Milestone 3 — Daily-driver quality
- Save/restore window size between sessions
- Dark/light theme (respect system preference)
- Syntax highlighting for code blocks
- Better error handling (missing file, bad UTF-8)

### Milestone 4 — Polish & packaging
- Native distro package / AppImage / Flatpak
- Final theme tuning

## What to avoid

- Custom text layout engine
- Native rendering of every Markdown node
- WYSIWYG / editor architecture
- Plugin system
- Notebook / workspace model
- File-open dialog or any GUI chrome
- Browser-like multiprocess features beyond what WebKit provides
