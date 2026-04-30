# Agent-render with MDViewer (plan)

Mirror the agent's last semantic response into a Markdown file; render it in
**[MDViewer](README.md)** rather than a browser. MDViewer is a single native
GTK4 + WebKitGTK process — lighter than a full browser, and after the
[`feat/katex-math`](src/main.rs) work it renders LaTeX via bundled KaTeX. With
live-reload built in, a single long-lived mdview window becomes the rendered
view of the agent's most recent answer.

```text
CLI agent in terminal
        │
        ├── normal terminal text remains source-of-truth
        │
        └── last assistant message → ~/.cache/agent-render/last.md
                                       │
                                       └── mdview live-reloads on save
```

For Claude Code, use a **Stop hook**. Stop hooks run on shell exit of an agent
turn and receive the JSON event `last_assistant_message`, so we capture the
actual assistant reply instead of scraping ANSI-laden terminal scrollback.
([Claude][1])

## What changes vs. the original browser-sidecar plan

The original plan had three layers: (1) capture, (2) Markdown→HTML via Pandoc
with MathJax, (3) open `last.html` in a browser. With mdview as the renderer,
two of those layers collapse:

- **No Pandoc step.** mdview parses markdown itself (comrak with GFM +
  `math_dollars`/`math_code`) and renders math via embedded KaTeX. The Stop
  hook just writes the raw markdown.
- **No `xdg-open` per turn.** mdview watches the file; one open window
  re-renders on every save. The hook never spawns a process.
- **No HTML/CSS scaffold to maintain.** Themes, syntax highlighting, math
  rendering, and dark-mode detection are all in mdview already.

What stays from the original plan:
- The Stop-hook capture pattern.
- The `~/.cache/agent-render/last.md` rendezvous file.
- The optional tmux key for raising the renderer.
- The clipboard-paste fallback for non-Claude agents.

## Recommendation

Three small pieces:

1. **Agent output convention** — ask the agent to emit GFM Markdown with LaTeX
   math (`$...$` inline, `$$...$$` display). MDViewer's comrak math extensions
   already match this convention, so no special instructions are needed beyond
   "use real LaTeX, not Unicode approximations for complex equations."
2. **Stop hook (silent capture)** — write the last assistant message to
   `~/.cache/agent-render/last.md`. No rendering step needed.
3. **One mdview window** — left open on `last.md`. Auto-refreshes on every
   write. No `xdg-open`, no tabs accumulating.

---

## 1. Capture script

`~/.local/bin/agent-render`:

```bash
#!/usr/bin/env bash
set -euo pipefail

cache="${XDG_CACHE_HOME:-$HOME/.cache}/agent-render"
mkdir -p "$cache"
md="$cache/last.md"

case "${1:-write}" in
  write)
    cat > "$md"
    ;;
  open)
    if ! pgrep -f "mdview.*$md" >/dev/null; then
      mdview "$md" >/dev/null 2>&1 &
      disown
    fi
    ;;
  path)
    echo "$md"
    ;;
  *)
    echo "usage: agent-render [write|open|path]" >&2
    exit 2
    ;;
esac
```

Compare with the original ~80-line version that ran Pandoc and managed CSS —
this is ~15 lines because mdview owns rendering.

---

## 2. Claude Code Stop hook

`~/.claude/hooks/render-last-assistant.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail
input="$(cat)"
msg="$(jq -r '.last_assistant_message // ""' <<<"$input")"
[[ -z "$msg" ]] && exit 0
printf '%s\n' "$msg" | "$HOME/.local/bin/agent-render" write
```

`~/.claude/settings.json`:

```json
{
  "hooks": {
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "$HOME/.claude/hooks/render-last-assistant.sh",
            "async": true
          }
        ]
      }
    ]
  }
}
```

Hooks run with user permissions — keep the script minimal and quote variables.
([Claude][1])

---

## 3. Two viable workflows

**A. Persistent sidebar.** Open `mdview ~/.cache/agent-render/last.md` once
(maybe on login, or when starting a Claude session) and leave it on a second
monitor or workspace. Each Stop fires the hook, the file is rewritten, mdview
live-reloads. No further action.

**B. On-demand.** Bind a key (tmux or window manager) to `agent-render open`,
which spawns mdview only when there's something interesting to look at. Useful
if you don't want a viewer window most of the time.

A tmux binding for either:

```tmux
bind-key r run-shell '$HOME/.local/bin/agent-render open'
```

---

## 4. Fallbacks for non-Claude agents

For agents without a Stop-hook equivalent (Codex, Aider, etc.), pipe selected
text directly:

```bash
# Wayland
wl-paste | agent-render write && agent-render open

# X11
xclip -o -selection clipboard | agent-render write && agent-render open
```

Or a tmux scrollback grab:

```tmux
bind-key R run-shell 'tmux capture-pane -pS -3000 | $HOME/.local/bin/agent-render write && $HOME/.local/bin/agent-render open'
```

This captures terminal UI noise (spinners, ANSI), so it's lower-fidelity than
the Stop-hook path — but it's universal.

---

## 5. Optional terminal-native preview

For pure-text answers without math, `glow` and `mdcat` render Markdown in the
terminal directly. ([GitHub][2]) The mdview path is preferred when math,
tables, or long derivations are involved — that's exactly what mdview is good
at and what terminals are bad at.

A tmux popup version, for the moments you want both:

```tmux
bind-key g display-popup -w 90% -h 90% -E 'glow -p ~/.cache/agent-render/last.md'
```

---

## Open questions / next steps before implementing

- **Snap confinement and `~/.cache/`**: mdview-as-snap with `home` plug should
  read `~/.cache/agent-render/last.md` fine, but verify before building this
  into a habit. The non-snap cargo build has no such concern.
- **Window focus on reload**: live-reload re-renders silently. For Workflow B
  (on-demand), `agent-render open` should also raise the existing window if
  one is already open — `wmctrl -a "mdview"` on X11, or a GTK side-channel
  (could be added to mdview itself: e.g. honor a single-instance flag and
  raise on second invocation).
- **Hook filtering**: do we want every Stop event mirrored, or only "long"
  responses? A length threshold in the hook (`[[ ${#msg} -gt 400 ]]`) would
  skip trivial replies.
- **Multi-conversation handling**: `last.md` is a single file. If two Claude
  sessions run in parallel they'll race on it. For solo use this is fine; if
  it matters later, key the cache path off `$CLAUDE_SESSION_ID` (if exposed by
  the hook) or `$TTY`.

[1]: https://code.claude.com/docs/en/hooks "Hooks reference - Claude Code Docs"
[2]: https://github.com/charmbracelet/glow "GitHub - charmbracelet/glow: Render markdown on the CLI"
