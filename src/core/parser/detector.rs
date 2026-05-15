use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Framework {
    Express,
    FastAPI,
    Flask,
    Django,
    Spring,
    NextJs,
    Laravel,
    AspNet,
    RubyOnRails,
    Golang,
    Quarkus,
    Unknown,
}

pub fn detect_framework(path: &Path) -> Framework {
    if path.join("package.json").exists() {
        if let Ok(content) = std::fs::read_to_string(path.join("package.json")) {
            if content.contains("\"express\"") {
                return Framework::Express;
            }
            if content.contains("\"next\"") {
                return Framework::NextJs;
            }
        }
    }

    let python_indicators = ["requirements.txt", "pyproject.toml", "Pipfile"];
    for indicator in python_indicators {
        let p = path.join(indicator);
        if p.exists() {
            if let Ok(content) = std::fs::read_to_string(p) {
                let content_lower = content.to_lowercase();
                if content_lower.contains("fastapi") {
                    return Framework::FastAPI;
                }
                if content_lower.contains("flask") {
                    return Framework::Flask;
                }
                if content_lower.contains("django") {
                    return Framework::Django;
                }
            }
        }
    }

    if path.join("pom.xml").exists()
        || path.join("build.gradle").exists()
        || path.join("build.gradle.kts").exists()
    {
        if let Ok(content) = std::fs::read_to_string(path.join(if path.join("pom.xml").exists() { "pom.xml" } else { "build.gradle" })) {
            if content.contains("quarkus") {
                return Framework::Quarkus;
            }
        }
        return Framework::Spring;
    }

    if path.join("artisan").exists() {
        return Framework::Laravel;
    }

    if path.join("go.mod").exists() {
        return Framework::Golang;
    }

    if path.join("Gemfile").exists() {
        if let Ok(content) = std::fs::read_to_string(path.join("Gemfile")) {
            if content.contains("'rails'") || content.contains("\"rails\"") {
                return Framework::RubyOnRails;
            }
        }
    }

    // ASP.NET detection
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            if let Some(ext) = entry.path().extension() {
                if ext == "csproj" || ext == "sln" {
                    return Framework::AspNet;
                }
            }
        }
    }

    Framework::Unknown
}

pub fn discover_projects(root: &Path) -> Vec<(PathBuf, Framework)> {
    let mut projects = Vec::new();
    let walker = WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != "node_modules" && name != "target" && name != ".git" && name != "venv" && name != ".venv"
        })
        .filter_map(|e| e.ok());

    for entry in walker {
        if entry.file_type().is_dir() {
            let framework = detect_framework(entry.path());
            if framework != Framework::Unknown {
                projects.push((entry.path().to_path_buf(), framework));
            }
        }
    }
    projects
}
