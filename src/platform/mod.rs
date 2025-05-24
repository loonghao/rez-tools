pub mod detection;
pub mod download;
pub mod extract;
pub mod installer;
pub mod python_standalone;
pub mod rez_path;

use crate::error::{Result, RezToolsError};
use std::path::PathBuf;

/// Platform information
#[derive(Debug, Clone)]
pub struct Platform {
    pub os: String,
    pub arch: String,
    pub target_triple: String,
}

impl Platform {
    /// Detect current platform
    pub fn detect() -> Self {
        let os = std::env::consts::OS.to_string();
        let arch = std::env::consts::ARCH.to_string();
        let target_triple = format!("{}-{}", arch, os);

        Self {
            os,
            arch,
            target_triple,
        }
    }

    /// Get platform-specific executable extension
    pub fn exe_extension(&self) -> &str {
        match self.os.as_str() {
            "windows" => ".exe",
            _ => "",
        }
    }

    /// Get platform-specific library extension
    pub fn lib_extension(&self) -> &str {
        match self.os.as_str() {
            "windows" => ".dll",
            "macos" => ".dylib",
            _ => ".so",
        }
    }

    /// Check if platform is supported
    pub fn is_supported(&self) -> bool {
        matches!(
            (self.os.as_str(), self.arch.as_str()),
            ("windows", "x86_64")
                | ("linux", "x86_64")
                | ("macos", "x86_64")
                | ("macos", "aarch64")
        )
    }
}

/// Rez environment information
#[derive(Debug, Clone)]
pub struct RezEnvironment {
    pub rez_path: Option<PathBuf>,
    pub python_path: Option<PathBuf>,
    pub packages_path: Vec<PathBuf>,
    pub is_installed: bool,
    pub version: Option<String>,
}

impl RezEnvironment {
    /// Detect existing rez installation
    pub fn detect() -> Result<Self> {
        detection::detect_rez_environment()
    }

    /// Install rez if not present
    pub async fn ensure_installed(&mut self) -> Result<()> {
        if !self.is_installed {
            installer::install_rez().await?;
            *self = Self::detect()?;
        }
        Ok(())
    }

    /// Get rez command path
    pub fn rez_command(&self) -> Result<PathBuf> {
        if let Some(ref rez_path) = self.rez_path {
            let platform = Platform::detect();
            let rez_exe = rez_path
                .join("bin")
                .join(format!("rez{}", platform.exe_extension()));
            if rez_exe.exists() {
                return Ok(rez_exe);
            }
        }

        // Try to find rez in PATH
        if let Ok(output) = std::process::Command::new("which").arg("rez").output() {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let path_str = output_str.trim();
                return Ok(PathBuf::from(path_str));
            }
        }

        Err(RezToolsError::ConfigError(
            "Rez command not found. Please install rez or run 'rt install-rez'".to_string(),
        ))
    }
}
