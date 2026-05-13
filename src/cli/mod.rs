pub mod args;

use crate::core::env::Environment;
use crate::engine::http::RequestEngine;
use args::Commands;
use std::collections::HashMap;

pub async fn run_cli(command: Commands) -> Result<(), Box<dyn std::error::Error>> {
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

            let final_url = environment.replace_vars(&url);
            let final_body = body.map(|b| environment.replace_vars(&b));

            let mut final_headers = HashMap::new();
            for h in header {
                if let Some((key, value)) = h.split_once(':') {
                    final_headers.insert(
                        environment.replace_vars(key.trim()),
                        environment.replace_vars(value.trim()),
                    );
                }
            }

            if offline {
                println!("--- OFFLINE MODE ---");
                println!("Method: {:?}", method);
                println!("URL: {}", final_url);
                println!("Headers: {:#?}", final_headers);
                if let Some(b) = final_body {
                    println!("Body:\n{}", b);
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

            if !silent && !headers_only {
                println!("Status: {}", response.status());
            }

            if headers_only {
                println!("{:#?}", response.headers());
                return Ok(());
            }

            if !silent {
                println!("Headers: {:#?}", response.headers());
            }

            let body_text = response.text().await?;

            if json_flag {
                println!("{}", body_text);
            } else if !silent {
                println!("Body:\n{}", body_text);
            } else {
                print!("{}", body_text);
            }
        }
    }
    Ok(())
}
