use crate::error::{Result, RezToolsError};
use crate::plugin::{parser::parse_plugin_file, Plugin};
use glob::glob;
use log::{debug, warn};
use std::collections::HashMap;
use std::path::Path;

/// Scan for .rt files in the given paths and return a map of plugin name to Plugin
pub fn scan_plugins<P: AsRef<Path>>(
    tool_paths: &[P],
    extension: &str,
) -> Result<HashMap<String, Plugin>> {
    let mut plugins = HashMap::new();
    let mut inheriting_plugins = Vec::new();

    // Process paths in reverse order (like the Python version)
    for path in tool_paths.iter().rev() {
        let path = path.as_ref();
        
        if !path.exists() {
            debug!("Tool path does not exist: {}", path.display());
            continue;
        }

        // Create glob pattern for .rt files
        let pattern = path.join(format!("*{}", extension));
        let pattern_str = pattern.to_string_lossy();

        debug!("Scanning for plugins in: {}", pattern_str);

        // Use glob to find matching files
        let entries = glob(&pattern_str).map_err(|e| {
            RezToolsError::ConfigError(format!("Invalid glob pattern '{}': {}", pattern_str, e))
        })?;

        for entry in entries {
            let plugin_file_path = match entry {
                Ok(path) => path,
                Err(e) => {
                    warn!("Error reading glob entry: {}", e);
                    continue;
                }
            };

            debug!("Found plugin file: {}", plugin_file_path.display());

            // Parse the plugin file
            let plugin = match parse_plugin_file(&plugin_file_path) {
                Ok(plugin) => plugin,
                Err(e) => {
                    warn!("Failed to parse plugin '{}': {}", plugin_file_path.display(), e);
                    continue;
                }
            };

            // Check if this plugin inherits from another
            if plugin.inherits_from.is_some() {
                debug!("Deferring load of sub-plugin {}", plugin.get_name());
                inheriting_plugins.push(plugin);
                continue;
            }

            let plugin_name = plugin.get_name();
            plugins.insert(plugin_name, plugin);
        }
    }

    // TODO: Handle inheriting plugins (for future implementation)
    if !inheriting_plugins.is_empty() {
        warn!("Plugin inheritance is not yet implemented. {} plugins deferred.", inheriting_plugins.len());
    }

    Ok(plugins)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_scan_plugins() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a test plugin file
        let plugin_file = temp_path.join("test_tool.rt");
        let mut file = fs::File::create(&plugin_file).unwrap();
        writeln!(
            file,
            r#"
command: test-command
short_help: Test tool
requires:
  - test-package
"#
        ).unwrap();

        // Scan for plugins
        let plugins = scan_plugins(&[temp_path], ".rt").unwrap();

        assert_eq!(plugins.len(), 1);
        assert!(plugins.contains_key("test_tool"));
        
        let plugin = &plugins["test_tool"];
        assert_eq!(plugin.command, "test-command");
        assert_eq!(plugin.get_short_help(), "Test tool");
        assert_eq!(plugin.requires, vec!["test-package"]);
    }

    #[test]
    fn test_scan_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let plugins = scan_plugins(&[temp_dir.path()], ".rt").unwrap();
        assert!(plugins.is_empty());
    }

    #[test]
    fn test_scan_nonexistent_directory() {
        let plugins = scan_plugins(&[Path::new("/nonexistent/path")], ".rt").unwrap();
        assert!(plugins.is_empty());
    }
}
