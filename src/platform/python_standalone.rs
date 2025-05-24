use crate::error::{Result, RezToolsError};
use crate::platform::{download::DownloadClient, extract::Extractor, Platform};
use log::{debug, info};
use serde_json::Value;
use std::path::PathBuf;
use tokio::fs;

/// Python Build Standalone manager
pub struct PythonStandalone {
    download_client: DownloadClient,
    install_dir: PathBuf,
}

impl PythonStandalone {
    /// Create a new Python Build Standalone manager
    pub fn new(install_dir: PathBuf) -> Self {
        Self {
            download_client: DownloadClient::new(),
            install_dir,
        }
    }

    /// Install Python Build Standalone for the current platform
    pub async fn install(&self) -> Result<PathBuf> {
        info!("Installing Python Build Standalone...");

        let platform = Platform::detect();

        // Get the download URL and filename
        let (download_url, filename) = self.get_download_info(&platform).await?;

        // Download the archive
        let archive_path = self.install_dir.join(&filename);
        self.download_client.download_file(&download_url, &archive_path).await?;

        // Extract the archive
        let extract_dir = self.install_dir.join("python");
        Extractor::extract(&archive_path, &extract_dir).await?;

        // Clean up the archive
        fs::remove_file(&archive_path).await?;

        // Find the Python executable
        let python_exe = self.find_python_executable(&extract_dir, &platform)?;

        info!("Python Build Standalone installed at: {}", python_exe.display());
        Ok(python_exe)
    }

    /// Get download information for the current platform
    async fn get_download_info(&self, platform: &Platform) -> Result<(String, String)> {
        info!("Fetching Python Build Standalone release information...");

        // Get latest release info from GitHub API
        let api_url = "https://api.github.com/repos/astral-sh/python-build-standalone/releases/latest";
        let release_info: Value = self.download_client
            .download_bytes(api_url)
            .await
            .and_then(|bytes| {
                serde_json::from_slice(&bytes)
                    .map_err(|e| RezToolsError::ConfigError(format!("Failed to parse JSON: {}", e)))
            })?;

        // Find the appropriate asset for our platform
        let target_pattern = self.get_target_pattern(platform)?;
        let assets = release_info["assets"].as_array()
            .ok_or_else(|| RezToolsError::ConfigError("No assets found in release".to_string()))?;

        // Look for the best matching asset
        let (download_url, filename) = self.find_best_asset(assets, &target_pattern)?;

        info!("Selected Python distribution: {}", filename);
        debug!("Download URL: {}", download_url);

        Ok((download_url, filename))
    }

    /// Get the target pattern for the current platform
    fn get_target_pattern(&self, platform: &Platform) -> Result<String> {
        let pattern = match (platform.os.as_str(), platform.arch.as_str()) {
            ("windows", "x86_64") => "x86_64-pc-windows-msvc-install_only",
            ("linux", "x86_64") => "x86_64-unknown-linux-gnu-install_only",
            ("macos", "x86_64") => "x86_64-apple-darwin-install_only",
            ("macos", "aarch64") => "aarch64-apple-darwin-install_only",
            _ => return Err(RezToolsError::ConfigError(format!(
                "Unsupported platform: {}-{}",
                platform.os, platform.arch
            ))),
        };

        Ok(pattern.to_string())
    }

    /// Find the best matching asset from the release
    fn find_best_asset(&self, assets: &[Value], target_pattern: &str) -> Result<(String, String)> {
        // Preferred Python versions in order
        let preferred_versions = ["3.11", "3.12", "3.10", "3.9"];

        for version in &preferred_versions {
            for asset in assets {
                let asset_name = asset["name"].as_str().unwrap_or("");

                // Check if this asset matches our criteria
                if asset_name.contains(target_pattern)
                    && asset_name.contains(version)
                    && asset_name.ends_with(".tar.gz") {

                    let download_url = asset["browser_download_url"]
                        .as_str()
                        .ok_or_else(|| RezToolsError::ConfigError("No download URL found".to_string()))?;

                    return Ok((download_url.to_string(), asset_name.to_string()));
                }
            }
        }

        Err(RezToolsError::ConfigError(format!(
            "No suitable Python Build Standalone found for pattern: {}",
            target_pattern
        )))
    }

    /// Find the Python executable in the extracted directory
    fn find_python_executable(&self, extract_dir: &PathBuf, platform: &Platform) -> Result<PathBuf> {
        // Python Build Standalone extracts to a nested structure
        // Check for the nested python directory first
        let nested_python_dir = extract_dir.join("python");

        // Common Python executable paths
        let possible_paths = if platform.os == "windows" {
            vec![
                // Check nested structure first (Python Build Standalone)
                nested_python_dir.join("python.exe"),
                nested_python_dir.join("Scripts").join("python.exe"),
                // Then check direct paths
                extract_dir.join("python.exe"),
                extract_dir.join("bin").join("python.exe"),
                extract_dir.join("Scripts").join("python.exe"),
            ]
        } else {
            vec![
                // Check nested structure first (Python Build Standalone)
                nested_python_dir.join("bin").join("python3"),
                nested_python_dir.join("bin").join("python"),
                nested_python_dir.join("python3"),
                nested_python_dir.join("python"),
                // Then check direct paths
                extract_dir.join("bin").join("python3"),
                extract_dir.join("bin").join("python"),
                extract_dir.join("python3"),
                extract_dir.join("python"),
            ]
        };

        for path in possible_paths {
            if path.exists() {
                debug!("Found Python executable: {}", path.display());
                return Ok(path);
            }
        }

        // If not found in common locations, search recursively but limit depth
        self.search_python_executable_limited(extract_dir, platform, 0, 3)
    }

    /// Recursively search for Python executable with depth limit
    fn search_python_executable_limited(&self, dir: &PathBuf, platform: &Platform, current_depth: usize, max_depth: usize) -> Result<PathBuf> {
        let exe_names = if platform.os == "windows" {
            vec!["python.exe", "python3.exe"]
        } else {
            vec!["python3", "python"]
        };

        fn search_recursive_limited(dir: &PathBuf, exe_names: &[&str], current_depth: usize, max_depth: usize) -> Option<PathBuf> {
            if current_depth > max_depth {
                return None;
            }

            if let Ok(entries) = std::fs::read_dir(dir) {
                // First pass: look for files in current directory
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                            if exe_names.contains(&filename) {
                                // Prefer executables not in venv or test directories
                                let path_str = path.to_string_lossy().to_lowercase();
                                if !path_str.contains("venv") && !path_str.contains("test") && !path_str.contains("lib") {
                                    return Some(path);
                                }
                            }
                        }
                    }
                }

                // Second pass: recurse into subdirectories
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            // Skip certain directories that are unlikely to contain the main Python executable
                            if let Some(dir_name) = path.file_name().and_then(|s| s.to_str()) {
                                let dir_name_lower = dir_name.to_lowercase();
                                if dir_name_lower.contains("test") ||
                                   dir_name_lower.contains("venv") ||
                                   dir_name_lower.contains("lib") ||
                                   dir_name_lower == "scripts" {
                                    continue;
                                }
                            }

                            if let Some(result) = search_recursive_limited(&path, exe_names, current_depth + 1, max_depth) {
                                return Some(result);
                            }
                        }
                    }
                }
            }
            None
        }

        search_recursive_limited(dir, &exe_names, current_depth, max_depth)
            .ok_or_else(|| RezToolsError::ConfigError(
                "Python executable not found in extracted archive".to_string()
            ))
    }



    /// Check if Python Build Standalone is already installed
    pub async fn is_installed(&self) -> bool {
        let python_dir = self.install_dir.join("python");
        if !python_dir.exists() {
            return false;
        }

        let platform = Platform::detect();
        match self.find_python_executable(&python_dir, &platform) {
            Ok(exe_path) => {
                debug!("Found existing Python installation: {}", exe_path.display());
                exe_path.exists()
            }
            Err(_) => false,
        }
    }

    /// Get the path to the installed Python executable
    pub fn get_python_executable(&self) -> Result<PathBuf> {
        let platform = Platform::detect();
        let python_dir = self.install_dir.join("python");

        if !python_dir.exists() {
            return Err(RezToolsError::ConfigError(
                "Python Build Standalone not installed".to_string()
            ));
        }

        self.find_python_executable(&python_dir, &platform)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_python_standalone_new() {
        let install_dir = PathBuf::from("/test/path");
        let python_standalone = PythonStandalone::new(install_dir.clone());
        assert_eq!(python_standalone.install_dir, install_dir);
    }

    #[test]
    fn test_get_target_pattern_windows() {
        let install_dir = PathBuf::from("/test");
        let python_standalone = PythonStandalone::new(install_dir);

        let platform = Platform {
            os: "windows".to_string(),
            arch: "x86_64".to_string(),
            target_triple: "x86_64-pc-windows-msvc".to_string(),
        };

        let pattern = python_standalone.get_target_pattern(&platform).unwrap();
        assert_eq!(pattern, "x86_64-pc-windows-msvc-install_only");
    }

    #[test]
    fn test_get_target_pattern_linux() {
        let install_dir = PathBuf::from("/test");
        let python_standalone = PythonStandalone::new(install_dir);

        let platform = Platform {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
        };

        let pattern = python_standalone.get_target_pattern(&platform).unwrap();
        assert_eq!(pattern, "x86_64-unknown-linux-gnu-install_only");
    }

    #[test]
    fn test_get_target_pattern_macos_x86_64() {
        let install_dir = PathBuf::from("/test");
        let python_standalone = PythonStandalone::new(install_dir);

        let platform = Platform {
            os: "macos".to_string(),
            arch: "x86_64".to_string(),
            target_triple: "x86_64-apple-darwin".to_string(),
        };

        let pattern = python_standalone.get_target_pattern(&platform).unwrap();
        assert_eq!(pattern, "x86_64-apple-darwin-install_only");
    }

    #[test]
    fn test_get_target_pattern_macos_aarch64() {
        let install_dir = PathBuf::from("/test");
        let python_standalone = PythonStandalone::new(install_dir);

        let platform = Platform {
            os: "macos".to_string(),
            arch: "aarch64".to_string(),
            target_triple: "aarch64-apple-darwin".to_string(),
        };

        let pattern = python_standalone.get_target_pattern(&platform).unwrap();
        assert_eq!(pattern, "aarch64-apple-darwin-install_only");
    }

    #[test]
    fn test_get_target_pattern_unsupported() {
        let install_dir = PathBuf::from("/test");
        let python_standalone = PythonStandalone::new(install_dir);

        let platform = Platform {
            os: "unsupported".to_string(),
            arch: "unknown".to_string(),
            target_triple: "unknown-unknown-unknown".to_string(),
        };

        let result = python_standalone.get_target_pattern(&platform);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported platform"));
    }

    #[test]
    fn test_find_python_executable_nested_windows() {
        let temp_dir = TempDir::new().unwrap();
        let install_dir = temp_dir.path().to_path_buf();
        let python_standalone = PythonStandalone::new(install_dir.clone());

        // Create nested Python directory structure
        let nested_python_dir = install_dir.join("python");
        fs::create_dir_all(&nested_python_dir).unwrap();

        // Create Python executable
        let python_exe = nested_python_dir.join("python.exe");
        fs::write(&python_exe, "fake python executable").unwrap();

        let platform = Platform {
            os: "windows".to_string(),
            arch: "x86_64".to_string(),
            target_triple: "x86_64-pc-windows-msvc".to_string(),
        };

        let result = python_standalone.find_python_executable(&install_dir, &platform);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), python_exe);
    }

    #[test]
    fn test_find_python_executable_nested_unix() {
        let temp_dir = TempDir::new().unwrap();
        let install_dir = temp_dir.path().to_path_buf();
        let python_standalone = PythonStandalone::new(install_dir.clone());

        // Create nested Python directory structure
        let nested_python_dir = install_dir.join("python");
        let bin_dir = nested_python_dir.join("bin");
        fs::create_dir_all(&bin_dir).unwrap();

        // Create Python executable
        let python_exe = bin_dir.join("python3");
        fs::write(&python_exe, "fake python executable").unwrap();

        let platform = Platform {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
        };

        let result = python_standalone.find_python_executable(&install_dir, &platform);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), python_exe);
    }

    #[test]
    fn test_find_python_executable_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let install_dir = temp_dir.path().to_path_buf();
        let python_standalone = PythonStandalone::new(install_dir.clone());

        let platform = Platform {
            os: "windows".to_string(),
            arch: "x86_64".to_string(),
            target_triple: "x86_64-pc-windows-msvc".to_string(),
        };

        let result = python_standalone.find_python_executable(&install_dir, &platform);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Python executable not found"));
    }

    #[test]
    fn test_search_python_executable_limited_depth() {
        let temp_dir = TempDir::new().unwrap();
        let install_dir = temp_dir.path().to_path_buf();
        let python_standalone = PythonStandalone::new(install_dir.clone());

        // Create deep nested structure that should be skipped
        let deep_dir = install_dir.join("level1").join("level2").join("level3").join("level4");
        fs::create_dir_all(&deep_dir).unwrap();

        let deep_python = deep_dir.join("python.exe");
        fs::write(&deep_python, "deep python").unwrap();

        // Create shallow Python executable
        let shallow_python = install_dir.join("python").join("python.exe");
        fs::create_dir_all(shallow_python.parent().unwrap()).unwrap();
        fs::write(&shallow_python, "shallow python").unwrap();

        let platform = Platform {
            os: "windows".to_string(),
            arch: "x86_64".to_string(),
            target_triple: "x86_64-pc-windows-msvc".to_string(),
        };

        let result = python_standalone.search_python_executable_limited(&install_dir, &platform, 0, 3);
        assert!(result.is_ok());
        // Should find the shallow one, not the deep one
        assert_eq!(result.unwrap(), shallow_python);
    }

    #[test]
    fn test_find_best_asset() {
        let install_dir = PathBuf::from("/test");
        let python_standalone = PythonStandalone::new(install_dir);

        // Mock asset data
        let assets = vec![
            serde_json::json!({
                "name": "cpython-3.10.12+20250517-x86_64-pc-windows-msvc-install_only.tar.gz",
                "browser_download_url": "https://example.com/python310.tar.gz"
            }),
            serde_json::json!({
                "name": "cpython-3.11.12+20250517-x86_64-pc-windows-msvc-install_only.tar.gz",
                "browser_download_url": "https://example.com/python311.tar.gz"
            }),
            serde_json::json!({
                "name": "cpython-3.12.1+20250517-x86_64-pc-windows-msvc-install_only.tar.gz",
                "browser_download_url": "https://example.com/python312.tar.gz"
            }),
        ];

        let target_pattern = "x86_64-pc-windows-msvc-install_only";
        let result = python_standalone.find_best_asset(&assets, target_pattern);

        assert!(result.is_ok());
        let (url, filename) = result.unwrap();
        // Should prefer 3.11 (first in preferred_versions list)
        assert!(filename.contains("3.11"));
        assert_eq!(url, "https://example.com/python311.tar.gz");
    }

    #[test]
    fn test_find_best_asset_no_match() {
        let install_dir = PathBuf::from("/test");
        let python_standalone = PythonStandalone::new(install_dir);

        // Mock asset data with no matching pattern
        let assets = vec![
            serde_json::json!({
                "name": "some-other-file.tar.gz",
                "browser_download_url": "https://example.com/other.tar.gz"
            }),
        ];

        let target_pattern = "x86_64-pc-windows-msvc-install_only";
        let result = python_standalone.find_best_asset(&assets, target_pattern);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No suitable Python Build Standalone found"));
    }
}
