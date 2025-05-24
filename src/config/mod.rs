pub mod loader;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for rez-tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Paths to search for .rt files
    pub tool_paths: Vec<PathBuf>,
    /// File extension for tool files (default: ".rt")
    pub extension: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tool_paths: vec![
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("packages"),
            ],
            extension: ".rt".to_string(),
        }
    }
}

impl Config {
    /// Create a new config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a tool path to the configuration
    pub fn add_tool_path<P: Into<PathBuf>>(&mut self, path: P) {
        self.tool_paths.push(path.into());
    }

    /// Set the file extension
    pub fn set_extension<S: Into<String>>(&mut self, extension: S) {
        self.extension = extension.into();
    }

    /// Expand and normalize all tool paths
    pub fn normalize_paths(&mut self) {
        self.tool_paths = self
            .tool_paths
            .iter()
            .map(|path| {
                // Expand user home directory
                if path.starts_with("~") {
                    if let Some(home) = dirs::home_dir() {
                        home.join(path.strip_prefix("~").unwrap_or(path))
                    } else {
                        path.clone()
                    }
                } else {
                    path.clone()
                }
            })
            .map(|path| {
                // Normalize the path
                path.canonicalize().unwrap_or(path)
            })
            .collect();
    }
}
