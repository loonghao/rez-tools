use crate::config::Config;
use crate::error::{Result, RezToolsError};
use log::{debug, warn};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Load configuration from environment variable or default locations
pub fn load_config() -> Result<Config> {
    // Check for REZ_TOOL_CONFIG environment variable
    if let Ok(config_path) = env::var("REZ_TOOL_CONFIG") {
        debug!("Loading config from REZ_TOOL_CONFIG: {}", config_path);
        return load_config_from_file(&config_path);
    }

    // Check for default config file in home directory
    if let Some(home_dir) = dirs::home_dir() {
        let default_config = home_dir.join("reztoolsconfig.py");
        if default_config.exists() {
            debug!("Loading config from default location: {}", default_config.display());
            return load_config_from_file(&default_config);
        }
    }

    // Return default configuration
    debug!("No config file found, using default configuration");
    Ok(Config::default())
}

/// Load configuration from a config file (Python or TOML)
fn load_config_from_file<P: AsRef<Path>>(config_path: P) -> Result<Config> {
    let config_path = config_path.as_ref();

    if !config_path.exists() {
        return Err(RezToolsError::ConfigError(format!(
            "Config file not found: {}",
            config_path.display()
        )));
    }

    let content = fs::read_to_string(config_path)?;

    // Determine file type by extension
    if let Some(extension) = config_path.extension().and_then(|s| s.to_str()) {
        match extension.to_lowercase().as_str() {
            "toml" => parse_toml_config(&content),
            "py" => parse_python_config(&content, Some(config_path)),
            _ => parse_python_config(&content, Some(config_path)), // Default to Python
        }
    } else {
        parse_python_config(&content, Some(config_path))
    }
}

/// Parse a Python config file by executing it with Python interpreter
fn parse_python_config(content: &str, config_file_path: Option<&Path>) -> Result<Config> {
    // Try to execute with Python interpreter first
    if let Some(config_path) = config_file_path {
        if let Ok(config) = execute_python_config(config_path) {
            return Ok(config);
        }
        warn!("Failed to execute Python config, falling back to simple parser");
    }

    // Fallback to simple parser
    parse_python_config_simple(content)
}

/// Execute Python config file and extract configuration
fn execute_python_config(config_path: &Path) -> Result<Config> {
    use std::process::Command;

    // Create a Python script to execute the config and output JSON
    let python_script = format!(r#"
import sys
import json
import os
sys.path.insert(0, os.path.dirname(r'{}'))

# Execute the config file
config_globals = {{'__file__': r'{}'}}
with open(r'{}', 'r') as f:
    exec(f.read(), config_globals)

# Extract configuration
result = {{}}
if 'tool_paths' in config_globals:
    result['tool_paths'] = config_globals['tool_paths']
if 'extension' in config_globals:
    result['extension'] = config_globals['extension']

print(json.dumps(result))
"#,
        config_path.display(),
        config_path.display(),
        config_path.display()
    );

    // Try different Python executables
    let python_commands = ["python", "python3", "py"];

    for python_cmd in &python_commands {
        let output = Command::new(python_cmd)
            .arg("-c")
            .arg(&python_script)
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                return parse_json_config(&stdout);
            }
            Ok(output) => {
                debug!("Python execution failed with {}: {}",
                       python_cmd, String::from_utf8_lossy(&output.stderr));
            }
            Err(e) => {
                debug!("Failed to run {}: {}", python_cmd, e);
            }
        }
    }

    Err(RezToolsError::ConfigError(
        "No working Python interpreter found".to_string()
    ))
}

/// Parse JSON output from Python execution
fn parse_json_config(json_str: &str) -> Result<Config> {
    use serde_json::Value;

    let value: Value = serde_json::from_str(json_str.trim())
        .map_err(|e| RezToolsError::ConfigError(format!("Invalid JSON from Python: {}", e)))?;

    let mut config = Config::default();
    config.tool_paths.clear();

    if let Some(tool_paths) = value.get("tool_paths").and_then(|v| v.as_array()) {
        for path in tool_paths {
            if let Some(path_str) = path.as_str() {
                config.tool_paths.push(PathBuf::from(path_str));
            }
        }
    }

    if let Some(extension) = value.get("extension").and_then(|v| v.as_str()) {
        config.extension = extension.to_string();
    }

    // If no tool_paths were found, use default
    if config.tool_paths.is_empty() {
        config = Config::default();
    }

    Ok(config)
}

/// Parse TOML configuration file
fn parse_toml_config(content: &str) -> Result<Config> {
    let config: Config = toml::from_str(content)
        .map_err(|e| RezToolsError::ConfigError(format!("Invalid TOML config: {}", e)))?;

    Ok(config)
}

/// Simple parser for basic Python config syntax (fallback)
fn parse_python_config_simple(content: &str) -> Result<Config> {
    let mut config = Config::default();
    config.tool_paths.clear(); // Clear default paths

    let mut in_tool_paths = false;
    let mut bracket_count = 0;

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        // Look for tool_paths assignment
        if line.starts_with("tool_paths") && line.contains('=') {
            in_tool_paths = true;
            // Check if it's a single line assignment
            if line.contains('[') && line.contains(']') {
                // Single line list
                let list_content = extract_list_content(line)?;
                config.tool_paths = parse_path_list(&list_content)?;
                in_tool_paths = false;
            } else if line.contains('[') {
                bracket_count = 1;
            }
            continue;
        }

        // Look for extension assignment
        if line.starts_with("extension") && line.contains('=') {
            if let Some(value) = extract_string_value(line) {
                config.extension = value;
            }
            continue;
        }

        // Handle multi-line tool_paths
        if in_tool_paths {
            bracket_count += line.chars().filter(|&c| c == '[').count();
            bracket_count -= line.chars().filter(|&c| c == ']').count();

            // Extract path from this line
            if let Some(path) = extract_path_from_line(line) {
                config.tool_paths.push(PathBuf::from(expand_path(&path)));
            }

            if bracket_count == 0 {
                in_tool_paths = false;
            }
        }
    }

    // If no tool_paths were found, use default
    if config.tool_paths.is_empty() {
        warn!("No tool_paths found in config, using default");
        config = Config::default();
    }

    Ok(config)
}

/// Extract list content from a single line
fn extract_list_content(line: &str) -> Result<String> {
    if let Some(start) = line.find('[') {
        if let Some(end) = line.rfind(']') {
            return Ok(line[start + 1..end].to_string());
        }
    }
    Err(RezToolsError::ConfigError(
        "Invalid list format in config".to_string(),
    ))
}

/// Parse a comma-separated list of paths
fn parse_path_list(content: &str) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    for item in content.split(',') {
        let item = item.trim();
        if let Some(path) = extract_string_value(item) {
            paths.push(PathBuf::from(expand_path(&path)));
        }
    }

    Ok(paths)
}

/// Extract a string value from a line (handles quotes)
fn extract_string_value(line: &str) -> Option<String> {
    let line = line.trim();

    // Handle quoted strings
    if (line.starts_with('"') && line.ends_with('"')) ||
       (line.starts_with('\'') && line.ends_with('\'')) {
        return Some(line[1..line.len() - 1].to_string());
    }

    // Handle assignment
    if let Some(pos) = line.find('=') {
        let value = line[pos + 1..].trim();
        return extract_string_value(value);
    }

    // Handle bare string
    if !line.contains('=') && !line.is_empty() {
        return Some(line.to_string());
    }

    None
}

/// Extract path from a line in a multi-line list
fn extract_path_from_line(line: &str) -> Option<String> {
    let line = line.trim();

    // Skip lines that are just brackets or empty
    if line == "]" || line == "[" || line.is_empty() {
        return None;
    }

    // Remove leading comma if present
    let line = if line.starts_with(',') {
        line[1..].trim()
    } else {
        line
    };

    // Remove trailing comma if present
    let line = if line.ends_with(',') {
        line[..line.len() - 1].trim()
    } else {
        line
    };

    extract_string_value(line)
}

/// Expand path expressions like os.path.expanduser("~/packages")
fn expand_path(path: &str) -> String {
    // Handle os.path.expanduser("~/...")
    if path.contains("expanduser") {
        if let Some(start) = path.find('"') {
            if let Some(end) = path.rfind('"') {
                let inner_path = &path[start + 1..end];
                if inner_path.starts_with("~/") {
                    if let Some(home) = dirs::home_dir() {
                        return home.join(&inner_path[2..]).to_string_lossy().to_string();
                    }
                }
                return inner_path.to_string();
            }
        }
    }

    // Handle os.path.dirname(__file__)
    if path.contains("dirname(__file__)") {
        // This would need the actual file path context
        // For now, return current directory
        return ".".to_string();
    }

    path.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_config() {
        let config_content = r#"
# Tool paths configuration
tool_paths = [
    "/path/to/tools",
    "~/packages",
    "/another/path"
]

extension = ".rt"
"#;

        let config = parse_python_config(config_content, None).unwrap();
        assert_eq!(config.tool_paths.len(), 3);
        assert_eq!(config.extension, ".rt");
    }

    #[test]
    fn test_extract_string_value() {
        assert_eq!(extract_string_value("\"hello\""), Some("hello".to_string()));
        assert_eq!(extract_string_value("'world'"), Some("world".to_string()));
        assert_eq!(extract_string_value("extension = \".rt\""), Some(".rt".to_string()));
    }
}
