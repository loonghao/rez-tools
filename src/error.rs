use std::fmt;

/// Custom error types for rez-tools
#[derive(Debug)]
pub enum RezToolsError {
    /// Configuration file not found or invalid
    ConfigError(String),
    /// Plugin file parsing error
    PluginParseError(String),
    /// Plugin validation error
    PluginValidationError(String),
    /// Rez command execution error
    RezExecutionError(String),
    /// IO error
    IoError(std::io::Error),
    /// YAML parsing error
    YamlError(serde_yaml::Error),
    /// Regex error
    RegexError(regex::Error),
}

impl fmt::Display for RezToolsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RezToolsError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            RezToolsError::PluginParseError(msg) => write!(f, "Plugin parse error: {}", msg),
            RezToolsError::PluginValidationError(msg) => write!(f, "Plugin validation error: {}", msg),
            RezToolsError::RezExecutionError(msg) => write!(f, "Rez execution error: {}", msg),
            RezToolsError::IoError(err) => write!(f, "IO error: {}", err),
            RezToolsError::YamlError(err) => write!(f, "YAML error: {}", err),
            RezToolsError::RegexError(err) => write!(f, "Regex error: {}", err),
        }
    }
}

impl std::error::Error for RezToolsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RezToolsError::IoError(err) => Some(err),
            RezToolsError::YamlError(err) => Some(err),
            RezToolsError::RegexError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for RezToolsError {
    fn from(err: std::io::Error) -> Self {
        RezToolsError::IoError(err)
    }
}

impl From<serde_yaml::Error> for RezToolsError {
    fn from(err: serde_yaml::Error) -> Self {
        RezToolsError::YamlError(err)
    }
}

impl From<regex::Error> for RezToolsError {
    fn from(err: regex::Error) -> Self {
        RezToolsError::RegexError(err)
    }
}

pub type Result<T> = std::result::Result<T, RezToolsError>;
