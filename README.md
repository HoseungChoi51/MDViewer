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

### myget (recommended)

This repo is published through `myget`, a personal GitHub-backed installer.
Once the repo carries the `myget` topic, install with:

```
myget install MDViewer
```

The GitHub repo is `MDViewer`; the installed command is `mdview`. `myget` prefers the
prebuilt `mdview-x86_64-linux` release asset and falls back to a source build
(`cargo install --path .`) when no matching asset exists. Build behavior, the release-asset
match, and the system dependencies below are declared in [`.myget.toml`](.myget.toml).

**System dependencies** (apt / Debian / Ubuntu):

- Runtime (prebuilt binary *and* source build): `libgtk-4-1 libwebkitgtk-6.0-4`
- Additional for the source-build fallback: `libgtk-4-dev libwebkitgtk-6.0-dev` (plus a Rust toolchain)

```
sudo apt install libgtk-4-1 libwebkitgtk-6.0-4   # runtime
```

### Snap

```
sudo snap install mdview_*.snap --dangerous
```

`--dangerous` is needed because the snap is distributed as a local file rather than through the Snap Store. Drop it if you ever publish to a store.

### Build the snap from source

```
sudo snap install snapcraft --classic
sudo snap install lxd
sudo lxd init --auto
sudo usermod -aG lxd $USER     # log out / log in after this
cd path/to/MDViewer
snapcraft pack
```

Produces `mdview_<version>_amd64.snap` in the repo root.

### Cargo build (development only)

For iterating on the code without rebuilding the whole snap:

```
sudo apt install libgtk-4-dev libwebkitgtk-6.0-dev
cargo run -- README.md
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
