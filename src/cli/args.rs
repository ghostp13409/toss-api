use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(name = "toss")]
#[command(about = "A Vim-inspired TUI API client", long_about = None)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Send an HTTP request
    Send {
        /// HTTP method (GET, POST, PUT, PATCH, DELETE)
        #[arg(short, long, default_value = "GET")]
        method: Method,

        /// Request URL
        url: String,

        /// Request body (JSON)
        #[arg(short, long)]
        body: Option<String>,

        /// Request headers (Key:Value)
        #[arg(short = 'H', long)]
        header: Vec<String>,

        /// Path to environment file (JSON or YAML)
        #[arg(short, long)]
        env: Option<String>,

        /// Suppress all output except the actual response body
        #[arg(long)]
        silent: bool,

        /// Force the output to be raw JSON, disabling fancy formatting
        #[arg(long)]
        json: bool,

        /// Print only the response headers
        #[arg(long)]
        headers_only: bool,

        /// Validate parameters and variables without sending the request
        #[arg(long)]
        offline: bool,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug, Serialize, Deserialize)]
#[value(rename_all = "UPPERCASE")]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl From<Method> for reqwest::Method {
    fn from(method: Method) -> Self {
        match method {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
            Method::Patch => reqwest::Method::PATCH,
            Method::Delete => reqwest::Method::DELETE,
        }
    }
}
