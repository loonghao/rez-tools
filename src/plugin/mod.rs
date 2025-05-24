pub mod parser;
pub mod scanner;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a rez tool plugin loaded from a .rt file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    /// The command to execute
    pub command: String,
    /// Optional name override (defaults to filename without extension)
    pub name: Option<String>,
    /// Short help description
    pub short_help: Option<String>,
    /// List of rez packages required
    pub requires: Vec<String>,
    /// Whether to run the command detached
    #[serde(default)]
    pub run_detached: bool,
    /// Tools this plugin inherits from
    pub inherits_from: Option<String>,
    /// File path where this plugin was loaded from
    #[serde(skip)]
    pub file_path: PathBuf,
}

impl Plugin {
    /// Get the effective name of the plugin
    pub fn get_name(&self) -> String {
        if let Some(ref name) = self.name {
            name.clone()
        } else {
            // Extract name from file path, removing .rt extension
            self.file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string()
        }
    }

    /// Get the short help text
    pub fn get_short_help(&self) -> String {
        self.short_help
            .clone()
            .unwrap_or_else(|| format!("A rez plugin - {}.", self.get_name()))
    }

    /// Validate the plugin configuration
    pub fn validate(&self) -> crate::error::Result<()> {
        use crate::error::RezToolsError;
        use regex::Regex;

        // Validate plugin name format
        let name = self.get_name();
        let name_pattern = Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]+$")?;
        if !name_pattern.is_match(&name) {
            return Err(RezToolsError::PluginValidationError(format!(
                "Plugin name '{}' does not match required pattern '^[a-zA-Z][a-zA-Z0-9_]+$'",
                name
            )));
        }

        // Validate that command is not empty
        if self.command.trim().is_empty() {
            return Err(RezToolsError::PluginValidationError(
                "Plugin command cannot be empty".to_string(),
            ));
        }

        // Validate that requires is not empty
        if self.requires.is_empty() {
            return Err(RezToolsError::PluginValidationError(
                "Plugin must specify at least one required package".to_string(),
            ));
        }

        Ok(())
    }
}
