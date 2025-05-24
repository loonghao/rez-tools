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
pub use error::{Result, RezToolsError};
pub use plugin::Plugin;
pub use config::Config;
