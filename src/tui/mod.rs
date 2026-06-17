pub mod app;
pub mod input;
pub mod ui;

use crate::engine::http::RequestEngine;
use app::{App, FocusedPanel, InputMode, ResponseStats, TuiAction};
use arboard::Clipboard;
use crossterm::{
    event::{self, Event, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use input::handle_input;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::collections::HashMap;
use std::io;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

enum AppEvent {
    Input(KeyEvent),
    Tick,
    HttpResponse(
        String,
        Option<String>,
        Option<ResponseStats>,
        Option<String>,
    ),
}

pub async fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new();

    let persistence = crate::core::persistence::PersistenceManager::new();
    if let Ok(cols) = persistence.load_collections() {
        app.collections = cols;
    }

    let res = run_app(&mut terminal, &mut app).await;

    // restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    terminal.show_cursor()?;

    // Save data
    let _ = persistence.save_collections(&app.collections);

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>>
where
    <B as ratatui::backend::Backend>::Error: 'static,
{
    let (tx, mut rx) = mpsc::channel(100);
    let tick_rate = Duration::from_millis(250);
    let is_paused = Arc::new(AtomicBool::new(false));
    let is_running = Arc::new(AtomicBool::new(true));

    // Event thread
    let tx_event = tx.clone();
    let is_paused_clone = is_paused.clone();
    let is_running_clone = is_running.clone();
    tokio::spawn(async move {
        let mut last_tick = Instant::now();
        while is_running_clone.load(Ordering::SeqCst) {
            if is_paused_clone.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(50)).await;
                continue;
            }

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).unwrap_or(false) {
                if !is_running_clone.load(Ordering::SeqCst) {
                    break;
                }
                if is_paused_clone.load(Ordering::SeqCst) {
                    continue;
                }
                if let Ok(Event::Key(key)) = event::read() {
                    if key.kind == event::KeyEventKind::Press {
                        let _ = tx_event.send(AppEvent::Input(key)).await;
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if is_running_clone.load(Ordering::SeqCst) {
                    let _ = tx_event.send(AppEvent::Tick).await;
                }
                last_tick = Instant::now();
            }
        }
    });

    let mut clipboard = Clipboard::new().ok();

    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if let Some(event) = rx.recv().await {
            match event {
                AppEvent::Input(key) => {
                    handle_input(app, key);
                }
                AppEvent::Tick => {
                    if let Some((_, instant)) = &app.notification {
                        if instant.elapsed() >= Duration::from_secs(3) {
                            app.notification = None;
                        }
                    }
                }
                AppEvent::HttpResponse(body, status, stats, content_type) => {
                    app.response_body = body;
                    app.response_content_type = content_type;
                    app.response_status = status;
                    app.response_stats_data = stats;
                    app.response_scroll = 0;
                    app.response_cursor_row = 0;
                    app.response_cursor_col = 0;
                }
            }
        }

        // Process all currently pending actions
        let actions: Vec<TuiAction> = app.pending_actions.drain(..).collect();
        for action in actions {
            match action {
                TuiAction::SendRequest => {
                    app.response_body = "Sending...".to_string();
                    app.response_status = None;

                    if let Some(req) = app.get_current_request() {
                        let env = app.get_active_env();
                        let method = req.method.into();
                        let url = env.replace_vars(&app.url);

                        let mut headers = HashMap::new();
                        for h in &req.headers {
                            if h.enabled && !h.key.is_empty() {
                                let key = env.replace_vars(&h.key);
                                let value = env.replace_vars(&h.value);
                                headers.insert(key, value);
                            }
                        }

                        let mut body_type = req.body.clone();
                        match body_type.selected {
                            crate::core::collection::BodyType::Raw => {
                                body_type.raw.content = env.replace_vars(&body_type.raw.content);
                            }
                            crate::core::collection::BodyType::FormData => {
                                for item in &mut body_type.form_data.items {
                                    item.key = env.replace_vars(&item.key);
                                    item.value = env.replace_vars(&item.value);
                                }
                            }
                            crate::core::collection::BodyType::XWwwFormUrlEncoded => {
                                for item in &mut body_type.x_www_form_urlencoded.items {
                                    item.key = env.replace_vars(&item.key);
                                    item.value = env.replace_vars(&item.value);
                                }
                            }
                            _ => {}
                        }

                        let mut auth = req.auth.clone();
                        match auth.selected {
                            crate::core::collection::AuthType::Bearer => {
                                auth.bearer.token = env.replace_vars(&auth.bearer.token);
                            }
                            crate::core::collection::AuthType::Basic => {
                                auth.basic.username = env.replace_vars(&auth.basic.username);
                                auth.basic.password = env.replace_vars(&auth.basic.password);
                            }
                            crate::core::collection::AuthType::ApiKey => {
                                auth.api_key.key = env.replace_vars(&auth.api_key.key);
                                auth.api_key.value = env.replace_vars(&auth.api_key.value);
                            }
                            _ => {}
                        }

                        let tx_res = tx.clone();
                        let final_url = url.clone();
                        let final_method = format!("{:?}", method);

                        tokio::spawn(async move {
                            let client = reqwest::Client::builder()
                                .connection_verbose(true)
                                .build()
                                .unwrap_or_default();

                            let start = Instant::now();
                            let engine = RequestEngine::with_client(client);

                            match engine
                                .send(method, &url, headers, Vec::new(), body_type, auth)
                                .await
                            {
                                Ok(res) => {
                                    let ttfb = start.elapsed();
                                    let status = Some(res.status().to_string());
                                    let version = format!("{:?}", res.version());
                                    let remote_addr = res.remote_addr().map(|a| a.to_string());

                                    let mut header_size = 0;
                                    let mut headers_map = HashMap::new();
                                    for (k, v) in res.headers() {
                                        let key_str = k.as_str().to_string();
                                        let val_str = v.to_str().unwrap_or("").to_string();
                                        header_size += key_str.len() + val_str.len() + 4;
                                        headers_map.insert(key_str, val_str);
                                    }

                                    let content_type = headers_map.get("content-type").cloned();

                                    let body_start = Instant::now();
                                    let body_bytes = res.bytes().await.unwrap_or_default();
                                    let download_time = body_start.elapsed();
                                    let total_time = start.elapsed();

                                    let body_size = body_bytes.len();
                                    let body_text =
                                        String::from_utf8_lossy(&body_bytes).into_owned();

                                    let stats = ResponseStats {
                                        total_time,
                                        dns_time: Duration::from_millis(0),
                                        connect_time: Duration::from_millis(0),
                                        tls_time: Duration::from_millis(0),
                                        ttfb,
                                        download_time,
                                        header_size,
                                        body_size,
                                        version,
                                        headers: headers_map,
                                        url: final_url,
                                        method: final_method,
                                        remote_addr,
                                    };

                                    let _ = tx_res
                                        .send(AppEvent::HttpResponse(
                                            body_text,
                                            status,
                                            Some(stats),
                                            content_type,
                                        ))
                                        .await;
                                }
                                Err(e) => {
                                    let _ = tx_res
                                        .send(AppEvent::HttpResponse(
                                            format!("Error: {}", e),
                                            Some("ERROR".to_string()),
                                            None,
                                            None,
                                        ))
                                        .await;
                                }
                            }
                        });
                    }
                }
                TuiAction::EditBody => {
                    let (req_id, current_body, extension) =
                        if let Some(req) = app.get_current_request() {
                            let (body, ct) = match req.body.selected {
                                crate::core::collection::BodyType::Raw => (
                                    req.body.raw.content.clone(),
                                    req.body.raw.content_type.clone(),
                                ),
                                _ => (String::new(), "text/plain".to_string()),
                            };
                            let ext = if ct.contains("json") {
                                "json"
                            } else if ct.contains("xml") {
                                "xml"
                            } else if ct.contains("html") {
                                "html"
                            } else {
                                "txt"
                            };
                            (req.id.clone(), body, ext)
                        } else {
                            continue;
                        };

                    is_paused.store(true, Ordering::SeqCst);

                    let mut temp_file = std::env::temp_dir();
                    temp_file.push(format!("toss_body_{}.{}", req_id, extension));

                    let _ = std::fs::write(&temp_file, current_body);

                    let _ = disable_raw_mode();
                    let _ = execute!(std::io::stdout(), LeaveAlternateScreen);

                    let editor = app.external_editor.clone().or_else(|| std::env::var("EDITOR").ok());

                    let editor_to_use = editor.unwrap_or_else(|| {
                        if cfg!(windows) {
                            "nvim.exe".to_string()
                        } else {
                            "vi".to_string()
                        }
                    });

                    // Try to launch the editor
                    let mut status = std::process::Command::new(&editor_to_use)
                        .arg(&temp_file)
                        .status();

                    // If the specified editor fails and we are on Windows, show the "Open With" dialog
                    if status.is_err() && cfg!(windows) {
                        println!("\nEditor '{}' not found or failed to start.", editor_to_use);
                        println!("Opening Windows 'Open With' dialog...");
                        println!("--------------------------------------------------");
                        println!("1. Select your editor in the popup.");
                        println!("2. Edit, SAVE, and CLOSE the editor.");
                        println!("3. Press ENTER here to finish.");
                        println!("--------------------------------------------------");

                        let _ = std::process::Command::new("rundll32.exe")
                            .args(["shell32.dll,OpenAs_RunDLL", &temp_file.to_string_lossy()])
                            .status();

                        // Wait for manual confirmation since rundll32 returns immediately
                        let mut input = String::new();
                        let _ = std::io::stdin().read_line(&mut input);
                        status = Ok(std::process::ExitStatus::default()); // Mock success
                    }

                    if status.is_ok() {
                        if let Ok(new_body) = std::fs::read_to_string(&temp_file) {
                            if let Some(col) = app.collections.get_mut(app.active_collection_index) {
                                if let Some(req_mut) = col.find_request_mut(&req_id) {
                                    req_mut.body.selected = crate::core::collection::BodyType::Raw;
                                    req_mut.body.raw.content = new_body;
                                    if req_mut.body.raw.content_type.is_empty() {
                                        req_mut.body.raw.content_type = "application/json".to_string();
                                    }
                                }
                            }
                        }
                    }
                    let _ = std::fs::remove_file(temp_file);

                    let _ = execute!(std::io::stdout(), EnterAlternateScreen);
                    let _ = enable_raw_mode();
                    let _ = terminal.clear();

                    app.input_mode = InputMode::Normal;
                    is_paused.store(false, Ordering::SeqCst);
                }
                TuiAction::CopyBody => {
                    if let Some(req) = app.get_current_request() {
                        let body = match req.body.selected {
                            crate::core::collection::BodyType::Raw => req.body.raw.content.clone(),
                            _ => String::new(),
                        };
                        if let Some(cb) = clipboard.as_mut() {
                            match cb.set_text(body) {
                                Ok(_) => app.notify("Body copied to clipboard"),
                                Err(e) => app.notify(format!("Failed to copy: {}", e)),
                            }
                        } else {
                            app.notify("Clipboard not available");
                        }
                    }
                }
                TuiAction::PasteBody => {
                    if let Some(cb) = clipboard.as_mut() {
                        match cb.get_text() {
                            Ok(text) => {
                                if let Some(col) =
                                    app.collections.get_mut(app.active_collection_index)
                                {
                                    if let Some(req_id) = &app.current_request_id {
                                        if let Some(req_mut) = col.find_request_mut(req_id) {
                                            req_mut.body.selected =
                                                crate::core::collection::BodyType::Raw;
                                            req_mut.body.raw.content = text;
                                            if req_mut.body.raw.content_type.is_empty() {
                                                req_mut.body.raw.content_type =
                                                    "application/json".to_string();
                                            }
                                            app.notify("Body pasted from clipboard");
                                        }
                                    }
                                }
                            }
                            Err(e) => app.notify(format!("Failed to paste: {}", e)),
                        }
                    } else {
                        app.notify("Clipboard not available");
                    }
                }
                TuiAction::Paste => {
                    if let Some(cb) = clipboard.as_mut() {
                        match cb.get_text() {
                            Ok(text) => match app.input_mode {
                                InputMode::Editing => {
                                    if app.focused_panel == FocusedPanel::RequestBar
                                        && app.active_request_part
                                            == crate::tui::app::RequestBarPart::Url
                                    {
                                        app.insert_string_url(&text);
                                        app.sync_params_from_url();
                                    } else if app.focused_panel == FocusedPanel::Details {
                                        let mut val = app.get_kv_editor_value();
                                        app.insert_string(&mut val, &text);
                                        app.update_kv_param(val);
                                    } else if app.focused_panel == FocusedPanel::Environments {
                                        let mut val = app.get_env_editor_value();
                                        app.insert_string(&mut val, &text);
                                        app.update_env_editor_value(val);
                                    }
                                }
                                InputMode::Normal => {
                                    if app.focused_panel == FocusedPanel::Details {
                                        app.update_kv_param(text.clone());
                                        app.notify("Pasted into field");
                                    } else if app.focused_panel == FocusedPanel::Environments {
                                        app.update_env_editor_value(text.clone());
                                        app.notify("Pasted into variable");
                                    }
                                }
                                InputMode::Rename | InputMode::CreateItem => {
                                    app.insert_string_rename(&text);
                                }
                                InputMode::Search => {
                                    let pos = app.cursor_position.min(app.search_query.len());
                                    app.search_query.insert_str(pos, &text);
                                    app.cursor_position = pos + text.len();
                                }
                                InputMode::Command => {
                                    let pos = app.cursor_position.min(app.command_input.len());
                                    app.command_input.insert_str(pos, &text);
                                    app.cursor_position = pos + text.len();
                                }
                                _ => {}
                            },
                            Err(e) => app.notify(format!("Failed to paste: {}", e)),
                        }
                    } else {
                        app.notify("Clipboard not available");
                    }
                }
                TuiAction::Copy => {
                    if let Some(cb) = clipboard.as_mut() {
                        let text = match app.input_mode {
                            InputMode::Editing => {
                                if app.focused_panel == FocusedPanel::RequestBar
                                    && app.active_request_part
                                        == crate::tui::app::RequestBarPart::Url
                                {
                                    Some(app.url.clone())
                                } else if app.focused_panel == FocusedPanel::Details {
                                    Some(app.get_kv_editor_value())
                                } else if app.focused_panel == FocusedPanel::Environments {
                                    Some(app.get_env_editor_value())
                                } else {
                                    None
                                }
                            }
                            InputMode::Rename | InputMode::CreateItem => {
                                Some(app.rename_input.clone())
                            }
                            InputMode::Search => Some(app.search_query.clone()),
                            InputMode::Command => Some(app.command_input.clone()),
                            InputMode::Normal => {
                                if app.focused_panel == FocusedPanel::Response {
                                    app.pending_actions.push(TuiAction::CopyResponseValue);
                                    None
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        };

                        if let Some(t) = text {
                            match cb.set_text(t) {
                                Ok(_) => app.notify("Copied to clipboard"),
                                Err(e) => app.notify(format!("Failed to copy: {}", e)),
                            }
                        }
                    } else {
                        app.notify("Clipboard not available");
                    }
                }
                TuiAction::CopyResponseBody => {
                    if let Some(cb) = clipboard.as_mut() {
                        match cb.set_text(app.response_body.clone()) {
                            Ok(_) => app.notify("Response body copied"),
                            Err(e) => app.notify(format!("Failed to copy: {}", e)),
                        }
                    } else {
                        app.notify("Clipboard not available");
                    }
                }
                TuiAction::CopyResponseValue => {
                    if let Some(cb) = clipboard.as_mut() {
                        let formatted = crate::tui::ui::syntax::format_content(
                            &app.response_body,
                            app.response_content_type.as_deref(),
                        );
                        let lines: Vec<&str> = formatted.lines().collect();
                        if let Some(line) = lines.get(app.response_cursor_row as usize) {
                            let (key, value) = crate::tui::ui::utils::extract_json_value(line);
                            match cb.set_text(value) {
                                Ok(_) => {
                                    if let Some(k) = key {
                                        app.notify(format!("{} value copied", k));
                                    } else {
                                        app.notify("Value copied to clipboard");
                                    }
                                }
                                Err(e) => app.notify(format!("Failed to copy: {}", e)),
                            }
                        }
                    } else {
                        app.notify("Clipboard not available");
                    }
                }
                TuiAction::CopyResponseAll => {
                    let all = format!(
                        "Status: {}\n\n{}",
                        app.response_status.as_deref().unwrap_or("Unknown"),
                        app.response_body
                    );
                    if let Some(cb) = clipboard.as_mut() {
                        match cb.set_text(all) {
                            Ok(_) => app.notify("Response copied"),
                            Err(e) => app.notify(format!("Failed to copy: {}", e)),
                        }
                    } else {
                        app.notify("Clipboard not available");
                    }
                }
            }
        }

        if app.should_quit {
            is_running.store(false, Ordering::SeqCst);
            return Ok(());
        }
    }
}
