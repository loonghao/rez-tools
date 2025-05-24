use crate::error::{Result, RezToolsError};
use crate::plugin::Plugin;
use std::fs;
use std::path::Path;

/// Parse a .rt file into a Plugin struct
pub fn parse_plugin_file<P: AsRef<Path>>(file_path: P) -> Result<Plugin> {
    let file_path = file_path.as_ref();

    // Read the file content
    let content = fs::read_to_string(file_path).map_err(|e| {
        RezToolsError::PluginParseError(format!(
            "Failed to read plugin file '{}': {}",
            file_path.display(),
            e
        ))
    })?;

    // Parse YAML content
    let mut plugin: Plugin = serde_yaml::from_str(&content).map_err(|e| {
        RezToolsError::PluginParseError(format!(
            "Failed to parse YAML in '{}': {}",
            file_path.display(),
            e
        ))
    })?;

    // Set the file path
    plugin.file_path = file_path.to_path_buf();

    // Validate the plugin
    plugin.validate()?;

    Ok(plugin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_plugin() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let plugin_file = temp_dir.path().join("python_tool.rt");
        let mut file = std::fs::File::create(&plugin_file).unwrap();
        writeln!(
            file,
            r#"
command: python
short_help: Python3.
requires:
  - python-3
  - cmake
"#
        )
        .unwrap();

        let plugin = parse_plugin_file(&plugin_file).unwrap();
        assert_eq!(plugin.command, "python");
        assert_eq!(plugin.short_help, Some("Python3.".to_string()));
        assert_eq!(plugin.requires, vec!["python-3", "cmake"]);
        assert!(!plugin.run_detached);
        assert_eq!(plugin.get_name(), "python_tool");
    }

    #[test]
    fn test_parse_plugin_with_name() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let plugin_file = temp_dir.path().join("any_name.rt");
        let mut file = std::fs::File::create(&plugin_file).unwrap();
        writeln!(
            file,
            r#"
name: custom_python
command: python
requires:
  - python-3
"#
        )
        .unwrap();

        let plugin = parse_plugin_file(&plugin_file).unwrap();
        assert_eq!(plugin.get_name(), "custom_python");
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "invalid: yaml: content:").unwrap();

        let result = parse_plugin_file(temp_file.path());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RezToolsError::PluginParseError(_)
        ));
    }

    #[test]
    fn test_parse_missing_required_fields() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"
short_help: Missing command and requires
"#
        )
        .unwrap();

        let result = parse_plugin_file(temp_file.path());
        assert!(result.is_err());
    }
}
