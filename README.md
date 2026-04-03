# mdview

A lightweight, read-only, keyboard-driven markdown viewer for Linux. Opens a `.md` file (or stdin) in a native window and renders it with syntax highlighting. No editing, no bloat -- just clean reading.

## Features

- GFM-compatible rendering (tables, task lists, strikethrough, autolinks)
- Syntax highlighting for fenced code blocks
- 5 light + 5 dark color themes, cycled with `t`
- Auto-detects system dark/light mode
- Live reload on file save
- Keyboard zoom
- Print / export to PDF via menu (`m` then `p`)
- Window size and theme persisted across sessions
- Reads from file or stdin

## Installation

### Build dependencies

On Ubuntu/Debian:

```
sudo apt install libgtk-4-dev libwebkitgtk-6.0-dev
```

### Install

```
cargo install --path .
```

This places the `mdview` binary in `~/.cargo/bin/`.

For a system-wide install:

```
cargo build --release
sudo cp target/release/mdview /usr/local/bin/
```

### Snap

```
sudo snapcraft --destructive-mode
sudo snap install mdview_0.1.0_amd64.snap --dangerous
```

## Usage

```
mdview file.md
mdview --width 1200 --height 900 file.md
cat file.md | mdview
```

## Keybindings

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `t` | Cycle color theme |
| `r` | Reload file |
| `m` | Open menu |
| `Ctrl +` / `Ctrl -` | Zoom in / out |
| `Ctrl 0` | Reset zoom |

### Menu

Press `m` to open the overlay menu, then:

| Key | Action |
|-----|--------|
| `p` | Print / export PDF |
| `Esc` | Close menu |

## Stack

| Component | Role |
|-----------|------|
| Rust | Language |
| GTK4 | Windowing |
| WebKitGTK | HTML rendering |
| Comrak | GFM Markdown to HTML |
| syntect | Syntax highlighting |
| clap | CLI argument parsing |

## License

MIT
