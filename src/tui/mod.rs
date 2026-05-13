pub mod app;
pub mod input;
pub mod ui;

use crate::engine::http::RequestEngine;
use app::{App, TuiAction};
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
    HttpResponse(String, Option<String>, Option<String>),
}

pub fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
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

    let res = run_app(&mut terminal, &mut app);

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

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>>
where
    <B as ratatui::backend::Backend>::Error: 'static,
{
    let (tx, mut rx) = mpsc::channel(100);
    let tick_rate = Duration::from_millis(250);
    let is_paused = Arc::new(AtomicBool::new(false));

    // Event thread
    let tx_event = tx.clone();
    let is_paused_clone = is_paused.clone();
    tokio::spawn(async move {
        let mut last_tick = Instant::now();
        loop {
            if is_paused_clone.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(50)).await;
                continue;
            }

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).unwrap_or(false) {
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
                let _ = tx_event.send(AppEvent::Tick).await;
                last_tick = Instant::now();
            }
        }
    });

    let engine = RequestEngine::new();
    let mut clipboard = Clipboard::new().ok();

    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if let Ok(event) = rx.try_recv() {
            match event {
                AppEvent::Input(key) => {
                    handle_input(app, key);
                }
                AppEvent::Tick => {}
                AppEvent::HttpResponse(body, status, stats) => {
                    app.response_body = body;
                    app.response_status = status;
                    if let Some(s) = stats {
                        app.response_stats = s;
                    }
                }
            }
        }

        // Process pending actions
        let actions: Vec<TuiAction> = app.pending_actions.drain(..).collect();
        for action in actions {
            match action {
                TuiAction::SendRequest => {
                    app.response_body = "Sending...".to_string();
                    app.response_status = None;

                    if let Some(req) = app.get_current_request() {
                        let method = req.method.into();
                        let url = app.url.clone();
                        let mut headers = HashMap::new();
                        for h in &req.headers {
                            if h.enabled && !h.key.is_empty() {
                                headers.insert(h.key.clone(), h.value.clone());
                            }
                        }

                        let mut params = Vec::new();
                        for p in &req.params {
                            if p.enabled && !p.key.is_empty() {
                                params.push((p.key.clone(), p.value.clone()));
                            }
                        }

                        let body_type = req.body.clone();
                        let auth = req.auth.clone();

                        let tx_res = tx.clone();
                        let engine_clone = engine.clone();

                        tokio::spawn(async move {
                            let start = Instant::now();
                            match engine_clone
                                .send(method, &url, headers, params, body_type, auth)
                                .await
                            {
                                Ok(res) => {
                                    let duration = start.elapsed();
                                    let status = Some(res.status().to_string());
                                    let version = format!("{:?}", res.version());
                                    let body = res
                                        .text()
                                        .await
                                        .unwrap_or_else(|e| format!("Error reading body: {}", e));
                                    let size = body.len();
                                    let stats = format!(
                                        "Time: {:?}\nSize: {} bytes\nProto: {}",
                                        duration, size, version
                                    );
                                    let _ = tx_res
                                        .send(AppEvent::HttpResponse(body, status, Some(stats)))
                                        .await;
                                }
                                Err(e) => {
                                    let _ = tx_res
                                        .send(AppEvent::HttpResponse(
                                            format!("Error: {}", e),
                                            Some("ERROR".to_string()),
                                            None,
                                        ))
                                        .await;
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
                            let _ = cb.set_text(body);
                        }
                    }
                }
                TuiAction::PasteBody => {
                    if let Some(cb) = clipboard.as_mut() {
                        if let Ok(text) = cb.get_text() {
                            if let Some(col) = app.collections.get_mut(app.active_collection_index)
                            {
                                if let Some(req_id) = &app.current_request_id {
                                    if let Some(req_mut) = col.find_request_mut(req_id) {
                                        req_mut.body = crate::core::collection::RequestBody::Raw {
                                            content: text,
                                            content_type: "application/json".to_string(),
                                        };
                                    }
                                }
                            }
                        }
                    }
                }
                TuiAction::CopyResponseBody => {
                    if let Some(cb) = clipboard.as_mut() {
                        let _ = cb.set_text(app.response_body.clone());
                    }
                }
                TuiAction::CopyResponseAll => {
                    let all = format!(
                        "Status: {}\n\n{}",
                        app.response_status.as_deref().unwrap_or("Unknown"),
                        app.response_body
                    );
                    if let Some(cb) = clipboard.as_mut() {
                        let _ = cb.set_text(all);
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
