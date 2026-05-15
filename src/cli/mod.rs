pub mod args;

use crate::core::env::Environment;
use crate::core::import::import_collection;
use crate::core::parser::parse_project;
use crate::core::persistence::PersistenceManager;
use crate::engine::http::RequestEngine;
use args::{CollectionsCommands, Commands, EnvCommands, Method};
use crossterm::style::{Color, Stylize};
use std::collections::HashMap;

pub async fn run_cli(command: Commands) -> Result<(), Box<dyn std::error::Error>> {
    let persistence = PersistenceManager::new();

    match command {
        Commands::Send {
            method,
            url,
            body,
            header,
            env,
            silent,
            json: json_flag,
            headers_only,
            offline,
        } => {
            let environment = if let Some(path) = env {
                Environment::from_file(path)?
            } else {
                Environment::default()
            };

            send_request(
                method,
                &url,
                body,
                header,
                &environment,
                silent,
                json_flag,
                headers_only,
                offline,
            )
            .await?;
        }
        Commands::Import { path } => {
            let collection = import_collection(path)?;
            let mut existing = persistence.load_collections()?;
            existing.push(collection.clone());
            persistence.save_collections(&existing)?;
            println!(
                "{} imported collection: {}",
                "Successfully".with(Color::Green),
                collection.name.with(Color::Cyan)
            );
        }
        Commands::Parse { path } => {
            let collection = parse_project(path)?;
            let mut existing = persistence.load_collections()?;
            existing.push(collection.clone());
            persistence.save_collections(&existing)?;
            println!(
                "{} parsed project and created collection: {}",
                "Successfully".with(Color::Green),
                collection.name.with(Color::Cyan)
            );
        }
        Commands::Collections { command } => match command {
            CollectionsCommands::List => {
                let collections = persistence.load_collections()?;
                if collections.is_empty() {
                    println!("No collections found. Use 'import' or 'parse' to add some.");
                    return Ok(());
                }
                println!("{}", "Your Collections:".with(Color::Yellow).bold());
                for (i, col) in collections.iter().enumerate() {
                    println!(
                        "  {}. {}",
                        (i + 1).to_string().with(Color::DarkGrey),
                        col.name
                    );
                }
            }
            CollectionsCommands::Show { name } => {
                let collections = persistence.load_collections()?;
                let col = collections
                    .iter()
                    .find(|c| c.name == name)
                    .ok_or_else(|| format!("Collection '{}' not found", name))?;

                println!(
                    "{} {}",
                    "Collection:".with(Color::Yellow).bold(),
                    col.name.clone().with(Color::Cyan)
                );
                print_collection_items(&col.items, 0);
            }
        },
        Commands::Run {
            collection,
            request,
            env,
            silent,
            json: json_flag,
        } => {
            let collections = persistence.load_collections()?;
            let col = collections
                .iter()
                .find(|c| c.name == collection)
                .ok_or_else(|| format!("Collection '{}' not found", collection))?;

            let req = col.find_request_by_name(&request).ok_or_else(|| {
                format!(
                    "Request '{}' not found in collection '{}'",
                    request, collection
                )
            })?;

            let mut environment = Environment::default();
            // Load collection env vars
            for v in &col.env_vars {
                if v.enabled {
                    environment.variables.insert(v.key.clone(), v.value.clone());
                }
            }

            // If a specific env file is provided, it overrides
            if let Some(env_path) = env {
                let file_env = Environment::from_file(env_path)?;
                for (k, v) in file_env.variables {
                    environment.variables.insert(k, v);
                }
            }

            let headers = req
                .headers
                .iter()
                .filter(|h| h.enabled)
                .map(|h| format!("{}:{}", h.key, h.value))
                .collect();

            let body = match &req.body {
                crate::core::collection::RequestBody::Raw { content, .. } => Some(content.clone()),
                _ => None,
            };

            send_request(
                req.method,
                &req.url,
                body,
                headers,
                &environment,
                silent,
                json_flag,
                false,
                false,
            )
            .await?;
        }
        Commands::Env { command } => match command {
            EnvCommands::List => {
                let collections = persistence.load_collections()?;
                println!(
                    "{}",
                    "Collections with Environments:".with(Color::Yellow).bold()
                );
                for col in collections {
                    if !col.env_vars.is_empty() {
                        println!("  - {} ({} variables)", col.name, col.env_vars.len());
                    }
                }
            }
            EnvCommands::Show { collection } => {
                let collections = persistence.load_collections()?;
                let col = collections
                    .iter()
                    .find(|c| c.name == collection)
                    .ok_or_else(|| format!("Collection '{}' not found", collection))?;

                println!(
                    "{} {}",
                    "Environment for:".with(Color::Yellow).bold(),
                    col.name.clone().with(Color::Cyan)
                );
                for v in &col.env_vars {
                    let status = if v.enabled {
                        "[x]".with(Color::Green)
                    } else {
                        "[ ]".with(Color::DarkGrey)
                    };
                    println!(
                        "  {} {}: {}",
                        status,
                        v.key.clone().with(Color::Yellow),
                        v.value
                    );
                }
            }
        },
    }
    Ok(())
}

async fn send_request(
    method: Method,
    url: &str,
    body: Option<String>,
    headers_list: Vec<String>,
    environment: &Environment,
    silent: bool,
    json_flag: bool,
    headers_only: bool,
    offline: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let final_url = environment.replace_vars(url);
    let final_body = body.map(|b| environment.replace_vars(&b));

    let mut final_headers = HashMap::new();
    for h in headers_list {
        if let Some((key, value)) = h.split_once(':') {
            final_headers.insert(
                environment.replace_vars(key.trim()),
                environment.replace_vars(value.trim()),
            );
        }
    }

    if offline {
        println!("{}", "--- OFFLINE MODE ---".with(Color::Yellow).bold());
        println!("{}: {:?}", "Method".bold(), method);
        println!("{}: {}", "URL".bold(), final_url);
        println!("{}: {:#?}", "Headers".bold(), final_headers);
        if let Some(b) = final_body {
            println!("{}:\n{}", "Body".bold(), b);
        }
        return Ok(());
    }

    let body_type = if let Some(b) = final_body {
        crate::core::collection::RequestBody::Raw {
            content: b,
            content_type: "application/json".to_string(),
        }
    } else {
        crate::core::collection::RequestBody::None
    };

    let engine = RequestEngine::new();
    let response = engine
        .send(
            method.into(),
            &final_url,
            final_headers,
            Vec::new(),
            body_type,
            crate::core::collection::Auth::None,
        )
        .await?;

    let status = response.status();
    let status_color = if status.is_success() {
        Color::Green
    } else if status.is_client_error() || status.is_server_error() {
        Color::Red
    } else {
        Color::Yellow
    };

    if !silent && !headers_only {
        println!(
            "{}: {}",
            "Status".bold(),
            status.to_string().with(status_color)
        );
    }

    if headers_only {
        println!("{:#?}", response.headers());
        return Ok(());
    }

    if !silent {
        println!("{}: {:#?}", "Headers".bold(), response.headers());
    }

    let body_text = response.text().await?;

    if json_flag {
        println!("{}", body_text);
    } else if !silent {
        println!("{}:\n{}", "Body".bold(), body_text);
    } else {
        print!("{}", body_text);
    }

    Ok(())
}

fn print_collection_items(items: &[crate::core::collection::CollectionItem], depth: usize) {
    use crate::core::collection::CollectionItem;
    let indent = "  ".repeat(depth);
    for item in items {
        match item {
            CollectionItem::Folder(f) => {
                println!("{}📁 {}", indent, f.name.clone().bold());
                print_collection_items(&f.items, depth + 1);
            }
            CollectionItem::Request(r) => {
                let method_color = match r.method {
                    Method::Get => Color::Green,
                    Method::Post => Color::Yellow,
                    Method::Put => Color::Blue,
                    Method::Patch => Color::Magenta,
                    Method::Delete => Color::Red,
                };
                println!(
                    "{}  {} {}",
                    indent,
                    format!("{:?}", r.method).with(method_color).bold(),
                    r.name
                );
            }
        }
    }
}
