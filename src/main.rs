use std::cell::Cell;
use std::io::Read as _;
use std::path::PathBuf;

use clap::Parser;
use comrak::{markdown_to_html, Options};
use gtk4::gdk::{Key, ModifierType};
use gtk4::gio;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, EventControllerKey};
use webkit6::prelude::*;
use webkit6::WebView;

const APP_ID: &str = "com.github.mdview";
const CSS: &str = include_str!("theme.css");
const DEFAULT_ZOOM: f64 = 1.0;
const ZOOM_STEP: f64 = 0.1;

#[derive(Parser)]
#[command(name = "mdview", about = "Lightweight markdown viewer")]
struct Cli {
    /// Markdown file to view (reads stdin if omitted)
    file: Option<PathBuf>,

    /// Window width in pixels
    #[arg(long, default_value_t = 900)]
    width: i32,

    /// Window height in pixels
    #[arg(long, default_value_t = 700)]
    height: i32,
}

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

fn md_to_html_doc(markdown: &str) -> String {
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.render.unsafe_ = false;

    let body = markdown_to_html(markdown, &options);

    format!(
        "<!DOCTYPE html>\n\
         <html>\n\
         <head>\n\
         <meta charset=\"utf-8\">\n\
         <style>\n{CSS}\n</style>\n\
         </head>\n\
         <body>\n{body}\n</body>\n\
         </html>"
    )
}

fn base_uri(base_dir: Option<&PathBuf>) -> Option<String> {
    base_dir.map(|d| format!("file://{}/", d.display()))
}

fn reload_content(webview: &WebView, file: &PathBuf, base_dir: Option<&PathBuf>) {
    let content = match std::fs::read_to_string(file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("mdview: reload failed: {}", e);
            return;
        }
    };
    let html = md_to_html_doc(&content);
    webview.load_html(&html, base_uri(base_dir).as_deref());
}

fn build_ui(app: &Application, cli: &Cli) {
    let (markdown, title, base_dir) = read_markdown(cli.file.as_ref());
    let html = md_to_html_doc(&markdown);

    let webview = WebView::new();
    let settings = webkit6::prelude::WebViewExt::settings(&webview).unwrap();
    settings.set_enable_javascript(false);

    webview.load_html(&html, base_uri(base_dir.as_ref()).as_deref());

    // Open external links in the default browser
    webview.connect_decide_policy(|_, decision, decision_type| {
        if decision_type == webkit6::PolicyDecisionType::NavigationAction {
            if let Some(nav_decision) = decision.downcast_ref::<webkit6::NavigationPolicyDecision>()
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

    let window = ApplicationWindow::builder()
        .application(app)
        .title(&format!("mdview — {}", title))
        .default_width(cli.width)
        .default_height(cli.height)
        .child(&webview)
        .build();

    // Keyboard shortcuts
    let key_controller = EventControllerKey::new();
    let win = window.clone();
    let wv = webview.clone();
    let file_path = cli.file.clone();
    let base_dir_clone = base_dir.clone();
    let zoom_level = Cell::new(DEFAULT_ZOOM);

    key_controller.connect_key_pressed(move |_, key, _, modifiers| {
        let ctrl = modifiers.contains(ModifierType::CONTROL_MASK);

        match (key, ctrl) {
            // Quit
            (Key::q, false) | (Key::Escape, _) => {
                win.close();
                gtk4::glib::Propagation::Stop
            }
            // Manual reload
            (Key::r, false) => {
                if let Some(ref path) = file_path {
                    reload_content(&wv, path, base_dir_clone.as_ref());
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
                        reload_content(&wv, &file, base.as_ref());
                    }
                });
                // Keep the monitor alive by leaking a ref into the window's data
                // (dropped when the window is destroyed, which is at app exit)
                // SAFETY: we store the monitor to keep it alive; the key is unique
                // and the data lives as long as the window.
                unsafe { window.set_data("_file_monitor", monitor); }
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

    let app = Application::builder().application_id(APP_ID).build();

    let cli = std::sync::Arc::new(cli);
    let cli_clone = cli.clone();
    app.connect_activate(move |app| {
        build_ui(app, &cli_clone);
    });

    app.run_with_args::<&str>(&[]);
}
