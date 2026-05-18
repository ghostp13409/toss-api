pub mod app;
pub mod input;
pub mod ui;

use crate::engine::http::RequestEngine;
use app::{App, ResponseStats, TuiAction};
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
    match persistence.load_collections() {
        Ok(cols) if !cols.is_empty() => app.collections = cols,
        _ => app.load_sample_data(),
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
                        match &mut body_type {
                            crate::core::collection::RequestBody::Raw { content, .. } => {
                                *content = env.replace_vars(content);
                            }
                            crate::core::collection::RequestBody::FormData { items }
                            | crate::core::collection::RequestBody::XWwwFormUrlEncoded { items } => {
                                for item in items {
                                    item.key = env.replace_vars(&item.key);
                                    item.value = env.replace_vars(&item.value);
                                }
                            }
                            crate::core::collection::RequestBody::None => {}
                        }

                        let auth = match req.auth.clone() {
                            crate::core::collection::Auth::Bearer { token } => {
                                crate::core::collection::Auth::Bearer {
                                    token: env.replace_vars(&token),
                                }
                            }
                            crate::core::collection::Auth::Basic { username, password } => {
                                crate::core::collection::Auth::Basic {
                                    username: env.replace_vars(&username),
                                    password: env.replace_vars(&password),
                                }
                            }
                            crate::core::collection::Auth::ApiKey {
                                key,
                                value,
                                in_header,
                            } => crate::core::collection::Auth::ApiKey {
                                key: env.replace_vars(&key),
                                value: env.replace_vars(&value),
                                in_header,
                            },
                            crate::core::collection::Auth::None => {
                                crate::core::collection::Auth::None
                            }
                        };

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

                            match engine.send(method, &url, headers, Vec::new(), body_type, auth).await {
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
                                    let body_text = String::from_utf8_lossy(&body_bytes).into_owned();

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

                                    let _ = tx_res.send(AppEvent::HttpResponse(
                                        body_text,
                                        status,
                                        Some(stats),
                                        content_type,
                                    )).await;
                                }
                                Err(e) => {
                                    let _ = tx_res.send(AppEvent::HttpResponse(
                                        format!("Error: {}", e),
                                        Some("ERROR".to_string()),
                                        None,
                                        None,
                                    )).await;
                                }
                            }
                        });
                    }
                }
                TuiAction::EditBody => {
                    let (req_id, current_body) = if let Some(req) = app.get_current_request() {
                        let body = match &req.body {
                            crate::core::collection::RequestBody::Raw { content, .. } => {
                                content.clone()
                            }
                            _ => String::new(),
                        };
                        (req.id.clone(), body)
                    } else {
                        continue;
                    };

                    is_paused.store(true, Ordering::SeqCst);

                    let temp_file = format!("/tmp/toss_body_{}.txt", req_id);
                    let _ = std::fs::write(&temp_file, current_body);

                    let _ = disable_raw_mode();
                    let _ = execute!(std::io::stdout(), LeaveAlternateScreen);

                    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
                    let _ = std::process::Command::new(editor).arg(&temp_file).status();

                    if let Ok(new_body) = std::fs::read_to_string(&temp_file) {
                        if let Some(col) = app.collections.get_mut(app.active_collection_index) {
                            if let Some(req_mut) = col.find_request_mut(&req_id) {
                                req_mut.body = crate::core::collection::RequestBody::Raw {
                                    content: new_body,
                                    content_type: "application/json".to_string(), // default
                                };
                            }
                        }
                    }
                    let _ = std::fs::remove_file(temp_file);

                    let _ = execute!(std::io::stdout(), EnterAlternateScreen);
                    let _ = enable_raw_mode();
                    let _ = terminal.clear();

                    is_paused.store(false, Ordering::SeqCst);
                }
                TuiAction::CopyBody => {
                    if let Some(req) = app.get_current_request() {
                        let body = match &req.body {
                            crate::core::collection::RequestBody::Raw { content, .. } => {
                                content.clone()
                            }
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
                                if let Some(col) = app.collections.get_mut(app.active_collection_index)
                                {
                                    if let Some(req_id) = &app.current_request_id {
                                        if let Some(req_mut) = col.find_request_mut(req_id) {
                                            req_mut.body = crate::core::collection::RequestBody::Raw {
                                                content: text,
                                                content_type: "application/json".to_string(),
                                            };
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
