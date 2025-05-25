//! # rez-tools
//!
//! A suite tool command line for [rez](https://github.com/nerdvegas/rez).
//!
//! This library provides functionality to scan for `.rt` files, parse them as YAML,
//! and dynamically generate command-line tools that integrate with the rez package manager.
//!
//! ## Features
//!
//! - Scan directories for `.rt` tool definition files
//! - Parse YAML-based tool configurations
//! - Generate dynamic command-line interfaces
//! - Execute tools within rez environments
//! - Support for both attached and detached execution modes
//!
//! ## Example
//!
//! ```rust,no_run
//! use rez_tools::cli::CliApp;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let app = CliApp::new()?;
//!     let exit_code = app.run().await?;
//!     std::process::exit(exit_code);
//! }
//! ```

pub mod cli;
pub mod config;
pub mod error;
pub mod platform;
pub mod plugin;
pub mod rez;

// Re-export commonly used types
pub use config::Config;
pub use error::{Result, RezToolsError};
pub use plugin::Plugin;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = platform::Platform::detect();

        // Verify platform detection works on all supported platforms
        assert!(!platform.os.is_empty(), "OS should be detected");
        assert!(!platform.arch.is_empty(), "Architecture should be detected");
        assert!(
            !platform.target_triple.is_empty(),
            "Target triple should be generated"
        );

        // Verify we're on a supported platform
        let supported_platforms = [
            ("windows", "x86_64"),
            ("linux", "x86_64"),
            ("macos", "x86_64"),
            ("macos", "aarch64"),
        ];

        let is_supported = supported_platforms
            .iter()
            .any(|(os, arch)| platform.os == *os && platform.arch == *arch);

        assert!(
            is_supported,
            "Platform {}-{} should be supported",
            platform.os, platform.arch
        );
    }

    #[test]
    fn test_error_types() {
        // Test that our error types work correctly
        let config_error = RezToolsError::ConfigError("test".to_string());
        assert!(config_error.to_string().contains("test"));

        let io_error = RezToolsError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "test file not found",
        ));
        assert!(io_error.to_string().contains("test file not found"));
    }
}
