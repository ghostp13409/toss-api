pub mod detector;
pub mod frameworks;
pub mod models;

use crate::core::collection::{Collection, CollectionItem, Folder};
use std::path::Path;

pub trait SourceParser {
    fn parse(&self, project_path: &Path) -> anyhow::Result<Collection>;
}

pub fn parse_project<P: AsRef<Path>>(path: P) -> anyhow::Result<Collection> {
    let path = path.as_ref();
    // Resolve absolute path to get correct folder name if path is "."
    let abs_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };

    // First try direct detection at root
    let framework = detector::detect_framework(&abs_path);
    if framework != detector::Framework::Unknown {
        let col = parse_single_project(&abs_path, framework)?;
        if col.items.is_empty() {
            anyhow::bail!(
                "Found project at {:?} but no endpoints were extracted",
                abs_path
            );
        }
        return Ok(col);
    }

    // If not found at root, discover recursively
    let discovered = detector::discover_projects(&abs_path);
    if discovered.is_empty() {
        anyhow::bail!("Unsupported or unknown framework at {:?}", abs_path);
    }

    let mut master_collection = Collection::new(format!(
        "{} (Workspace)",
        abs_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Root".to_string())
    ));

    for (p, f) in discovered {
        match parse_single_project(&p, f) {
            Ok(col) => {
                if !col.items.is_empty() {
                    let mut folder = Folder::new(col.name.clone());
                    folder.items = col.items;
                    master_collection.items.push(CollectionItem::Folder(folder));
                }
            }
            Err(e) => {
                eprintln!("Warning: failed to parse project at {:?}: {}", p, e);
            }
        }
    }

    if master_collection.items.is_empty() {
        anyhow::bail!(
            "Found projects but no endpoints were extracted from any of them at {:?}",
            abs_path
        );
    }

    Ok(master_collection)
}

fn parse_single_project(path: &Path, framework: detector::Framework) -> anyhow::Result<Collection> {
    match framework {
        detector::Framework::Express => {
            let parser = frameworks::express::ExpressParser;
            parser.parse(path)
        }
        detector::Framework::FastAPI => {
            let parser = frameworks::fastapi::FastApiParser;
            parser.parse(path)
        }
        detector::Framework::Spring => {
            let parser = frameworks::spring::SpringParser;
            parser.parse(path)
        }
        detector::Framework::AspNet => {
            let parser = frameworks::aspnet::AspNetParser;
            parser.parse(path)
        }
        detector::Framework::Flask => {
            let parser = frameworks::flask::FlaskParser;
            parser.parse(path)
        }
        detector::Framework::Django => {
            let parser = frameworks::django::DjangoParser;
            parser.parse(path)
        }
        detector::Framework::Laravel => {
            let parser = frameworks::laravel::LaravelParser;
            parser.parse(path)
        }
        detector::Framework::RubyOnRails => {
            let parser = frameworks::ruby_on_rails::RubyOnRailsParser;
            parser.parse(path)
        }
        detector::Framework::Golang => {
            let parser = frameworks::golang::GolangParser;
            parser.parse(path)
        }
        detector::Framework::Quarkus => {
            let parser = frameworks::quarkus::QuarkusParser;
            parser.parse(path)
        }
        detector::Framework::NextJs => {
            let parser = frameworks::nextjs::NextJsParser;
            parser.parse(path)
        }
        _ => anyhow::bail!("Unsupported framework {:?} at {:?}", framework, path),
    }
}
