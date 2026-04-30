use std::cell::Cell;
use std::io::Read as _;
use std::path::PathBuf;

use clap::Parser;
use comrak::plugins::syntect::SyntectAdapterBuilder;
use comrak::{markdown_to_html_with_plugins, Options, Plugins};
use gtk4::gdk::{Key, ModifierType};
use gtk4::gio;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, EventControllerKey};
use serde::{Deserialize, Serialize};
use webkit6::prelude::*;
use webkit6::{PrintOperation, WebView};

const APP_ID: &str = "com.github.mdview";
const BASE_CSS: &str = include_str!("theme.css");
const KATEX_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/katex.inlined.css"));
const KATEX_JS: &str = include_str!("../assets/katex/katex.min.js");
const KATEX_BOOTSTRAP_JS: &str = r#"
(function() {
  function render() {
    document.querySelectorAll("[data-math-style]").forEach(function(el) {
      if (el.dataset.mdviewRendered) return;
      var display = el.getAttribute("data-math-style") === "display";
      var tex = el.textContent;
      if (display && tex.startsWith("$$") && tex.endsWith("$$")) tex = tex.slice(2, -2);
      else if (tex.startsWith("$") && tex.endsWith("$")) tex = tex.slice(1, -1);
      try {
        katex.render(tex.trim(), el, { displayMode: display, throwOnError: false });
        el.dataset.mdviewRendered = "1";
      } catch (e) { console.error("KaTeX:", e); }
    });
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", render);
  } else {
    render();
  }
})();
"#;
const DEFAULT_ZOOM: f64 = 1.0;
const ZOOM_STEP: f64 = 0.1;
const THEME_COUNT: usize = 5;

#[derive(Parser)]
#[command(name = "mdview", about = "Lightweight markdown viewer")]
struct Cli {
    /// Markdown file to view (reads stdin if omitted)
    file: Option<PathBuf>,

    /// Window width in pixels
    #[arg(long)]
    width: Option<i32>,

    /// Window height in pixels
    #[arg(long)]
    height: Option<i32>,
}

// --- Themes ---

struct PageTheme {
    name: &'static str,
    syntect_theme: &'static str,
    bg: &'static str,
    text: &'static str,
    accent: &'static str,
    heading_border: &'static str,
    muted: &'static str,
    code_bg: &'static str,
    pre_bg: &'static str,
    pre_border: &'static str,
    blockquote_bg: &'static str,
    blockquote_border: &'static str,
    table_border: &'static str,
    table_header_bg: &'static str,
    table_stripe: &'static str,
    hr: &'static str,
}

const LIGHT_THEMES: [PageTheme; THEME_COUNT] = [
    PageTheme {
        name: "Default",
        syntect_theme: "InspiredGitHub",
        bg: "#fff",
        text: "#1a1a1a",
        accent: "#0366d6",
        heading_border: "#e0e0e0",
        muted: "#555",
        code_bg: "#f0f0f0",
        pre_bg: "#f6f6f6",
        pre_border: "#e0e0e0",
        blockquote_bg: "#fafafa",
        blockquote_border: "#ddd",
        table_border: "#ddd",
        table_header_bg: "#f6f6f6",
        table_stripe: "#fafafa",
        hr: "#e0e0e0",
    },
    PageTheme {
        name: "Warm",
        syntect_theme: "Solarized (light)",
        bg: "#faf8f5",
        text: "#2c2416",
        accent: "#8b5a00",
        heading_border: "#e8e0d0",
        muted: "#6b5b47",
        code_bg: "#f0ebe0",
        pre_bg: "#f5f0e8",
        pre_border: "#e0d8c8",
        blockquote_bg: "#f5f0e8",
        blockquote_border: "#d0c4a8",
        table_border: "#ddd0bc",
        table_header_bg: "#f0ebe0",
        table_stripe: "#f5f0e8",
        hr: "#e0d8c8",
    },
    PageTheme {
        name: "Cool",
        syntect_theme: "base16-ocean.light",
        bg: "#f5f7fa",
        text: "#1a2332",
        accent: "#2563eb",
        heading_border: "#d8dfe8",
        muted: "#4a5568",
        code_bg: "#e8ecf2",
        pre_bg: "#edf0f5",
        pre_border: "#d0d8e4",
        blockquote_bg: "#edf0f5",
        blockquote_border: "#b0bcd0",
        table_border: "#c8d0dc",
        table_header_bg: "#e8ecf2",
        table_stripe: "#edf0f5",
        hr: "#d0d8e4",
    },
    PageTheme {
        name: "Solarized",
        syntect_theme: "Solarized (light)",
        bg: "#fdf6e3",
        text: "#586e75",
        accent: "#268bd2",
        heading_border: "#eee8d5",
        muted: "#93a1a1",
        code_bg: "#eee8d5",
        pre_bg: "#eee8d5",
        pre_border: "#ddd6c1",
        blockquote_bg: "#eee8d5",
        blockquote_border: "#93a1a1",
        table_border: "#ddd6c1",
        table_header_bg: "#eee8d5",
        table_stripe: "#f5efdc",
        hr: "#ddd6c1",
    },
    PageTheme {
        name: "Paper",
        syntect_theme: "InspiredGitHub",
        bg: "#f9f5ed",
        text: "#333",
        accent: "#7c4d28",
        heading_border: "#e0d8c8",
        muted: "#666",
        code_bg: "#efe9de",
        pre_bg: "#f3ede2",
        pre_border: "#ddd5c4",
        blockquote_bg: "#f3ede2",
        blockquote_border: "#c8b898",
        table_border: "#d8d0c0",
        table_header_bg: "#efe9de",
        table_stripe: "#f3ede2",
        hr: "#ddd5c4",
    },
];

const DARK_THEMES: [PageTheme; THEME_COUNT] = [
    PageTheme {
        name: "Default",
        syntect_theme: "base16-ocean.dark",
        bg: "#1e1e1e",
        text: "#d4d4d4",
        accent: "#58a6ff",
        heading_border: "#333",
        muted: "#999",
        code_bg: "#2d2d2d",
        pre_bg: "#252526",
        pre_border: "#333",
        blockquote_bg: "#252526",
        blockquote_border: "#444",
        table_border: "#333",
        table_header_bg: "#2d2d2d",
        table_stripe: "#252526",
        hr: "#333",
    },
    PageTheme {
        name: "Mocha",
        syntect_theme: "base16-mocha.dark",
        bg: "#1e1d2f",
        text: "#cdd6f4",
        accent: "#f5c2e7",
        heading_border: "#313244",
        muted: "#a6adc8",
        code_bg: "#28273d",
        pre_bg: "#252438",
        pre_border: "#313244",
        blockquote_bg: "#252438",
        blockquote_border: "#585b70",
        table_border: "#313244",
        table_header_bg: "#28273d",
        table_stripe: "#252438",
        hr: "#313244",
    },
    PageTheme {
        name: "Ocean",
        syntect_theme: "base16-ocean.dark",
        bg: "#2b303b",
        text: "#c0c5ce",
        accent: "#8fa1b3",
        heading_border: "#3b4252",
        muted: "#8b929e",
        code_bg: "#343d4a",
        pre_bg: "#313845",
        pre_border: "#3b4252",
        blockquote_bg: "#313845",
        blockquote_border: "#4c566a",
        table_border: "#3b4252",
        table_header_bg: "#343d4a",
        table_stripe: "#313845",
        hr: "#3b4252",
    },
    PageTheme {
        name: "Eighties",
        syntect_theme: "base16-eighties.dark",
        bg: "#2d2d2d",
        text: "#d3d0c8",
        accent: "#f99157",
        heading_border: "#3b3b3b",
        muted: "#a09f93",
        code_bg: "#383838",
        pre_bg: "#353535",
        pre_border: "#3b3b3b",
        blockquote_bg: "#353535",
        blockquote_border: "#555",
        table_border: "#3b3b3b",
        table_header_bg: "#383838",
        table_stripe: "#353535",
        hr: "#3b3b3b",
    },
    PageTheme {
        name: "Solarized",
        syntect_theme: "Solarized (dark)",
        bg: "#002b36",
        text: "#839496",
        accent: "#268bd2",
        heading_border: "#073642",
        muted: "#657b83",
        code_bg: "#073642",
        pre_bg: "#073642",
        pre_border: "#0a4050",
        blockquote_bg: "#073642",
        blockquote_border: "#586e75",
        table_border: "#073642",
        table_header_bg: "#073642",
        table_stripe: "#06313b",
        hr: "#073642",
    },
];

fn theme_css(t: &PageTheme) -> String {
    format!(
        r#"
body {{
    color: {text};
    background: {bg};
}}
h1, h2 {{ border-bottom-color: {heading_border}; }}
h6 {{ color: {muted}; }}
a {{ color: {accent}; }}
code {{ background: {code_bg}; }}
pre {{ background: {pre_bg}; border-color: {pre_border}; }}
blockquote {{ border-left-color: {blockquote_border}; color: {muted}; background: {blockquote_bg}; }}
th, td {{ border-color: {table_border}; }}
th {{ background: {table_header_bg}; }}
tr:nth-child(even) {{ background: {table_stripe}; }}
hr {{ border-top-color: {hr}; }}
del {{ color: {muted}; }}
"#,
        bg = t.bg,
        text = t.text,
        accent = t.accent,
        heading_border = t.heading_border,
        muted = t.muted,
        code_bg = t.code_bg,
        pre_bg = t.pre_bg,
        pre_border = t.pre_border,
        blockquote_bg = t.blockquote_bg,
        blockquote_border = t.blockquote_border,
        table_border = t.table_border,
        table_header_bg = t.table_header_bg,
        table_stripe = t.table_stripe,
        hr = t.hr,
    )
}

fn get_theme(dark: bool, index: usize) -> &'static PageTheme {
    if dark {
        &DARK_THEMES[index % THEME_COUNT]
    } else {
        &LIGHT_THEMES[index % THEME_COUNT]
    }
}

// --- Window size persistence ---

#[derive(Serialize, Deserialize)]
struct WindowState {
    width: i32,
    height: i32,
    #[serde(default)]
    theme_index: usize,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: 900,
            height: 700,
            theme_index: 0,
        }
    }
}

fn state_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("mdview").join("state.json"))
}

fn load_window_state() -> WindowState {
    state_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_window_state(state: &WindowState) {
    if let Some(path) = state_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, serde_json::to_string(state).unwrap_or_default());
    }
}

// --- Markdown rendering ---

fn read_markdown(file: Option<&PathBuf>) -> (String, String, Option<PathBuf>) {
    match file {
        Some(path) => {
            let content = std::fs::read_to_string(path).unwrap_or_else(|e| {
                eprintln!("mdview: {}: {}", path.display(), e);
                std::process::exit(1);
            });
            let abs = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
            let title = abs
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "mdview".into());
            let base_dir = abs.parent().map(|p| p.to_path_buf());
            (content, title, base_dir)
        }
        None => {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .unwrap_or_else(|e| {
                    eprintln!("mdview: failed to read stdin: {}", e);
                    std::process::exit(1);
                });
            (buf, "stdin".into(), None)
        }
    }
}

fn is_dark_mode() -> bool {
    let settings = match gtk4::Settings::default() {
        Some(s) => s,
        None => return false,
    };
    if settings.is_gtk_application_prefer_dark_theme() {
        return true;
    }
    if let Some(theme) = settings.gtk_theme_name() {
        if theme.as_str().to_lowercase().contains("dark") {
            return true;
        }
    }
    false
}

fn md_to_html_doc(markdown: &str, dark: bool, theme_index: usize) -> String {
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.math_dollars = true;
    options.extension.math_code = true;
    options.render.unsafe_ = false;

    let theme = get_theme(dark, theme_index);
    let adapter = SyntectAdapterBuilder::new()
        .theme(theme.syntect_theme)
        .build();
    let mut plugins = Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    let body = markdown_to_html_with_plugins(markdown, &options, &plugins);
    let colors = theme_css(theme);

    format!(
        "<!DOCTYPE html>\n\
         <html>\n\
         <head>\n\
         <meta charset=\"utf-8\">\n\
         <style>\n{BASE_CSS}\n{colors}\n{KATEX_CSS}\n</style>\n\
         </head>\n\
         <body>\n{body}\n\
         <script>{KATEX_JS}</script>\n\
         <script>{KATEX_BOOTSTRAP_JS}</script>\n\
         </body>\n\
         </html>"
    )
}

fn base_uri(base_dir: Option<&PathBuf>) -> Option<String> {
    base_dir.map(|d| format!("file://{}/", d.display()))
}

fn render_to_webview(
    webview: &WebView,
    markdown: &str,
    dark: bool,
    theme_index: usize,
    base_dir: Option<&PathBuf>,
) {
    let html = md_to_html_doc(markdown, dark, theme_index);
    webview.load_html(&html, base_uri(base_dir).as_deref());
}

// --- UI ---

fn build_ui(app: &Application, cli: &Cli) {
    let (markdown, title, base_dir) = read_markdown(cli.file.as_ref());

    let saved = load_window_state();
    let win_width = cli.width.unwrap_or(saved.width);
    let win_height = cli.height.unwrap_or(saved.height);
    let theme_index = Cell::new(saved.theme_index);
    let dark = is_dark_mode();

    let webview = WebView::new();
    let settings = webkit6::prelude::WebViewExt::settings(&webview).unwrap();
    // JS is needed for KaTeX. Markdown HTML is still escaped (unsafe_=false in
    // comrak), so only the bundled KaTeX scripts run.
    settings.set_enable_javascript(true);

    render_to_webview(&webview, &markdown, dark, theme_index.get(), base_dir.as_ref());

    // Open external links in the default browser
    webview.connect_decide_policy(|_, decision, decision_type| {
        if decision_type == webkit6::PolicyDecisionType::NavigationAction {
            if let Some(nav_decision) =
                decision.downcast_ref::<webkit6::NavigationPolicyDecision>()
            {
                let nav_action = nav_decision.navigation_action();
                if let Some(mut nav_action) = nav_action {
                    let request = nav_action.request();
                    if let Some(request) = request {
                        if let Some(uri) = request.uri() {
                            let uri_str = uri.as_str();
                            if uri_str.starts_with("http://") || uri_str.starts_with("https://") {
                                let _ = gio::AppInfo::launch_default_for_uri(
                                    uri_str,
                                    gio::AppLaunchContext::NONE,
                                );
                                decision.ignore();
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    });

    // --- Menu overlay (m to open, p to print, any key to close) ---
    let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    menu_box.set_halign(gtk4::Align::Center);
    menu_box.set_valign(gtk4::Align::Center);
    menu_box.set_visible(false);
    menu_box.add_css_class("mdview-menu");

    let lbl = gtk4::Label::new(Some("mdview"));
    lbl.add_css_class("mdview-menu-title");
    menu_box.append(&lbl);

    let sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    sep.add_css_class("mdview-menu-sep");
    menu_box.append(&sep);

    for (key, desc) in [("p", "Print / Export PDF"), ("Esc", "Close menu")] {
        let item = gtk4::Label::new(Some(&format!("{:<5} {}", key, desc)));
        item.set_halign(gtk4::Align::Start);
        item.add_css_class("mdview-menu-item");
        menu_box.append(&item);
    }

    let overlay = gtk4::Overlay::new();
    overlay.set_child(Some(&webview));
    overlay.add_overlay(&menu_box);

    let css_provider = gtk4::CssProvider::new();
    css_provider.load_from_data(
        ".mdview-menu {
            background: rgba(30, 30, 30, 0.92);
            border-radius: 10px;
            padding: 20px 28px;
            min-width: 220px;
        }
        .mdview-menu-title {
            color: white;
            font-weight: bold;
            font-size: 15px;
        }
        .mdview-menu-sep {
            margin: 4px 0;
            min-height: 1px;
            background: rgba(255, 255, 255, 0.15);
        }
        .mdview-menu-item {
            color: rgba(255, 255, 255, 0.75);
            font-family: monospace;
            font-size: 13px;
        }",
    );
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().unwrap(),
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let window = ApplicationWindow::builder()
        .application(app)
        .title(&format!("mdview — {}", title))
        .default_width(win_width)
        .default_height(win_height)
        .child(&overlay)
        .build();

    // Save window size and theme on close
    let theme_index_save = theme_index.clone();
    window.connect_close_request(move |win| {
        let state = WindowState {
            width: win.width(),
            height: win.height(),
            theme_index: theme_index_save.get(),
        };
        save_window_state(&state);
        gtk4::glib::Propagation::Proceed
    });

    // Keyboard shortcuts
    let key_controller = EventControllerKey::new();
    let win = window.clone();
    let wv = webview.clone();
    let file_path = cli.file.clone();
    let base_dir_clone = base_dir.clone();
    let zoom_level = Cell::new(DEFAULT_ZOOM);
    let theme_index_reload = theme_index.clone();
    // Keep a copy of the initial markdown for stdin (can't re-read)
    let cached_markdown = std::sync::Arc::new(markdown);
    let menu_visible = Cell::new(false);
    let menu_clone = menu_box.clone();

    key_controller.connect_key_pressed(move |_, key, _, modifiers| {
        let ctrl = modifiers.contains(ModifierType::CONTROL_MASK);

        // Menu mode: handle action, then close
        if menu_visible.get() {
            if key == Key::p {
                let print_op = PrintOperation::new(&wv);
                print_op.run_dialog(Some(&win));
            }
            menu_clone.set_visible(false);
            menu_visible.set(false);
            return gtk4::glib::Propagation::Stop;
        }

        match (key, ctrl) {
            // Open menu
            (Key::m, false) => {
                menu_clone.set_visible(true);
                menu_visible.set(true);
                gtk4::glib::Propagation::Stop
            }
            // Quit
            (Key::q, false) | (Key::Escape, _) => {
                win.close();
                gtk4::glib::Propagation::Stop
            }
            // Cycle theme
            (Key::t, false) => {
                let next = (theme_index.get() + 1) % THEME_COUNT;
                theme_index.set(next);
                let dark = is_dark_mode();
                let theme = get_theme(dark, next);
                let md = if let Some(ref path) = file_path {
                    std::fs::read_to_string(path).unwrap_or_else(|_| (*cached_markdown).clone())
                } else {
                    (*cached_markdown).clone()
                };
                render_to_webview(&wv, &md, dark, next, base_dir_clone.as_ref());
                eprintln!(
                    "mdview: theme → {} ({})",
                    theme.name,
                    if dark { "dark" } else { "light" }
                );
                gtk4::glib::Propagation::Stop
            }
            // Manual reload
            (Key::r, false) => {
                if let Some(ref path) = file_path {
                    let content = match std::fs::read_to_string(path) {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("mdview: reload failed: {}", e);
                            return gtk4::glib::Propagation::Stop;
                        }
                    };
                    render_to_webview(
                        &wv,
                        &content,
                        is_dark_mode(),
                        theme_index.get(),
                        base_dir_clone.as_ref(),
                    );
                }
                gtk4::glib::Propagation::Stop
            }
            // Zoom in: Ctrl++ or Ctrl+=
            (Key::plus | Key::equal, true) => {
                let z = (zoom_level.get() + ZOOM_STEP).min(5.0);
                zoom_level.set(z);
                wv.set_zoom_level(z);
                gtk4::glib::Propagation::Stop
            }
            // Zoom out: Ctrl+-
            (Key::minus, true) => {
                let z = (zoom_level.get() - ZOOM_STEP).max(0.2);
                zoom_level.set(z);
                wv.set_zoom_level(z);
                gtk4::glib::Propagation::Stop
            }
            // Zoom reset: Ctrl+0
            (Key::_0, true) => {
                zoom_level.set(DEFAULT_ZOOM);
                wv.set_zoom_level(DEFAULT_ZOOM);
                gtk4::glib::Propagation::Stop
            }
            _ => gtk4::glib::Propagation::Proceed,
        }
    });
    window.add_controller(key_controller);

    // Live reload: watch the file for changes
    if let Some(ref path) = cli.file {
        let abs_path = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());
        let gio_file = gio::File::for_path(&abs_path);
        match gio_file.monitor_file(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE) {
            Ok(monitor) => {
                let wv = webview.clone();
                let file = abs_path.clone();
                let base = base_dir.clone();
                monitor.connect_changed(move |_, _, _, event| {
                    if event == gio::FileMonitorEvent::ChangesDoneHint
                        || event == gio::FileMonitorEvent::Changed
                    {
                        let content = match std::fs::read_to_string(&file) {
                            Ok(c) => c,
                            Err(e) => {
                                eprintln!("mdview: reload failed: {}", e);
                                return;
                            }
                        };
                        render_to_webview(
                            &wv,
                            &content,
                            is_dark_mode(),
                            theme_index_reload.get(),
                            base.as_ref(),
                        );
                    }
                });
                // SAFETY: we store the monitor to keep it alive; the key is unique
                // and the data lives as long as the window.
                unsafe {
                    window.set_data("_file_monitor", monitor);
                }
            }
            Err(e) => {
                eprintln!("mdview: could not watch file: {}", e);
            }
        }
    }

    window.present();
}

fn main() {
    let cli = Cli::parse();

    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    // Prevent DBus registration — not needed for a viewer, and avoids
    // AppArmor denials under snap strict confinement.
    app.set_register_session(false);

    let cli = std::sync::Arc::new(cli);
    let cli_clone = cli.clone();
    app.connect_activate(move |app| {
        build_ui(app, &cli_clone);
    });

    app.run_with_args::<&str>(&[]);
}
