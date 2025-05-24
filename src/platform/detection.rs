use crate::error::{Result, RezToolsError};
use crate::platform::RezEnvironment;
use log::{debug, info, warn};
use std::env;
use std::path::PathBuf;
use std::process::Command;

/// Detect existing rez installation
pub fn detect_rez_environment() -> Result<RezEnvironment> {
    info!("Detecting rez environment...");

    let mut env = RezEnvironment {
        rez_path: None,
        python_path: None,
        packages_path: Vec::new(),
        is_installed: false,
        version: None,
    };

    // Check REZ_PATH environment variable
    if let Ok(rez_path) = env::var("REZ_PATH") {
        debug!("Found REZ_PATH: {}", rez_path);
        env.rez_path = Some(PathBuf::from(rez_path));
    }

    // Try to detect rez installation
    if let Ok(rez_version) = detect_rez_version() {
        info!("Found rez installation: {}", rez_version);
        env.is_installed = true;
        env.version = Some(rez_version);

        // Get rez configuration
        if let Ok(config) = get_rez_config() {
            env.packages_path = config.packages_path;
            env.python_path = config.python_path;
        }
    } else {
        // Check for our Python Build Standalone installation with rez
        if let Ok(standalone_env) = detect_standalone_rez_installation() {
            if standalone_env.is_installed {
                info!("Found rez in Python Build Standalone installation");
                env.is_installed = true;
                env.version = standalone_env.version;
                env.python_path = standalone_env.python_path;
            }
        } else {
            warn!("Rez not found in system");
        }
    }

    // Check for Python Build Standalone
    if env.python_path.is_none() {
        if let Some(python_path) = detect_python_build_standalone() {
            debug!("Found Python Build Standalone: {}", python_path.display());
            env.python_path = Some(python_path);
        }
    }

    Ok(env)
}

/// Detect rez version
fn detect_rez_version() -> Result<String> {
    let output = Command::new("rez")
        .arg("--version")
        .output()
        .map_err(|e| RezToolsError::ConfigError(format!("Failed to run rez --version: {}", e)))?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        Ok(version.trim().to_string())
    } else {
        Err(RezToolsError::ConfigError("Rez command failed".to_string()))
    }
}

/// Get rez configuration
fn get_rez_config() -> Result<RezConfig> {
    let output = Command::new("rez")
        .args(&["config", "--print"])
        .output()
        .map_err(|e| RezToolsError::ConfigError(format!("Failed to get rez config: {}", e)))?;

    if output.status.success() {
        let config_str = String::from_utf8_lossy(&output.stdout);
        parse_rez_config(&config_str)
    } else {
        Err(RezToolsError::ConfigError("Failed to get rez config".to_string()))
    }
}

/// Parse rez configuration output
fn parse_rez_config(config_str: &str) -> Result<RezConfig> {
    let mut config = RezConfig {
        packages_path: Vec::new(),
        python_path: None,
    };

    for line in config_str.lines() {
        let line = line.trim();

        if line.starts_with("packages_path:") {
            // Parse packages_path list
            if let Some(paths_str) = line.strip_prefix("packages_path:") {
                let paths_str = paths_str.trim();
                if paths_str.starts_with('[') && paths_str.ends_with(']') {
                    let paths_content = &paths_str[1..paths_str.len()-1];
                    for path in paths_content.split(',') {
                        let path = path.trim().trim_matches('"').trim_matches('\'');
                        if !path.is_empty() {
                            config.packages_path.push(PathBuf::from(path));
                        }
                    }
                }
            }
        } else if line.starts_with("python_executable:") {
            if let Some(python_str) = line.strip_prefix("python_executable:") {
                let python_path = python_str.trim().trim_matches('"').trim_matches('\'');
                if !python_path.is_empty() {
                    config.python_path = Some(PathBuf::from(python_path));
                }
            }
        }
    }

    Ok(config)
}

/// Detect our Python Build Standalone installation with rez
fn detect_standalone_rez_installation() -> Result<RezEnvironment> {
    // Get the rez-tools directory
    let rez_tools_dir = if let Some(home) = dirs::home_dir() {
        home.join(".rez-tools")
    } else {
        return Err(RezToolsError::ConfigError("Cannot find home directory".to_string()));
    };

    let install_dir = rez_tools_dir.join("python-build-standalone");
    let python_dir = install_dir.join("python");

    // Check if Python Build Standalone is installed
    if !python_dir.exists() {
        return Ok(RezEnvironment {
            rez_path: None,
            python_path: None,
            packages_path: Vec::new(),
            is_installed: false,
            version: None,
        });
    }

    // Find Python executable (handle nested structure)
    let python_exe = find_python_executable_in_dir(&python_dir)?;

    // Check if rez is installed in this Python environment
    let output = Command::new(&python_exe)
        .args(&["-c", "import rez; print(rez.__version__)"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            debug!("Found rez version in Python Build Standalone: {}", version);

            Ok(RezEnvironment {
                rez_path: None,
                python_path: Some(python_exe),
                packages_path: Vec::new(),
                is_installed: true,
                version: Some(format!("Rez {}", version)),
            })
        }
        _ => {
            debug!("Rez not found in Python Build Standalone installation");
            Ok(RezEnvironment {
                rez_path: None,
                python_path: Some(python_exe),
                packages_path: Vec::new(),
                is_installed: false,
                version: None,
            })
        }
    }
}

/// Find Python executable in a directory
fn find_python_executable_in_dir(dir: &PathBuf) -> Result<PathBuf> {
    use crate::platform::Platform;

    let platform = Platform::detect();

    // Python Build Standalone extracts to a nested structure
    // Check for the nested python directory first
    let nested_python_dir = dir.join("python");

    // Common Python executable paths
    let possible_paths = if platform.os == "windows" {
        vec![
            // Check nested structure first (Python Build Standalone)
            nested_python_dir.join("python.exe"),
            nested_python_dir.join("Scripts").join("python.exe"),
            // Then check direct paths
            dir.join("python.exe"),
            dir.join("bin").join("python.exe"),
            dir.join("Scripts").join("python.exe"),
        ]
    } else {
        vec![
            // Check nested structure first (Python Build Standalone)
            nested_python_dir.join("bin").join("python3"),
            nested_python_dir.join("bin").join("python"),
            nested_python_dir.join("python3"),
            nested_python_dir.join("python"),
            // Then check direct paths
            dir.join("bin").join("python3"),
            dir.join("bin").join("python"),
            dir.join("python3"),
            dir.join("python"),
        ]
    };

    for path in possible_paths {
        if path.exists() {
            debug!("Found Python executable: {}", path.display());
            return Ok(path);
        }
    }

    Err(RezToolsError::ConfigError(
        "Python executable not found in Python Build Standalone installation".to_string()
    ))
}

/// Detect Python Build Standalone installation
fn detect_python_build_standalone() -> Option<PathBuf> {
    // Check common installation paths
    let possible_paths = [
        "~/.local/share/python-build-standalone",
        "/opt/python-build-standalone",
        "/usr/local/python-build-standalone",
    ];

    for path_str in &possible_paths {
        let path = if path_str.starts_with('~') {
            if let Some(home) = dirs::home_dir() {
                home.join(&path_str[2..])
            } else {
                continue;
            }
        } else {
            PathBuf::from(path_str)
        };

        if path.exists() {
            // Look for Python executable
            let python_exe = path.join("bin").join("python3");
            if python_exe.exists() {
                return Some(python_exe);
            }
        }
    }

    None
}

#[derive(Debug)]
struct RezConfig {
    packages_path: Vec<PathBuf>,
    python_path: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_rez_environment() {
        let env = detect_rez_environment();
        assert!(env.is_ok());
    }

    #[test]
    fn test_parse_rez_config() {
        let config_str = r#"
packages_path: ["/path/to/packages", "/another/path"]
python_executable: "/usr/bin/python3"
"#;
        let config = parse_rez_config(config_str).unwrap();
        assert_eq!(config.packages_path.len(), 2);
        assert!(config.python_path.is_some());
        assert_eq!(config.packages_path[0], PathBuf::from("/path/to/packages"));
        assert_eq!(config.packages_path[1], PathBuf::from("/another/path"));
        assert_eq!(config.python_path.unwrap(), PathBuf::from("/usr/bin/python3"));
    }

    #[test]
    fn test_parse_rez_config_empty() {
        let config_str = "";
        let config = parse_rez_config(config_str).unwrap();
        assert_eq!(config.packages_path.len(), 0);
        assert!(config.python_path.is_none());
    }

    #[test]
    fn test_parse_rez_config_malformed() {
        let config_str = r#"
packages_path: ["/path/to/packages"
python_executable:
"#;
        let config = parse_rez_config(config_str).unwrap();
        // Should handle malformed config gracefully
        assert_eq!(config.packages_path.len(), 0);
        assert!(config.python_path.is_none());
    }

    #[test]
    fn test_find_python_executable_in_dir_nested() {
        let temp_dir = TempDir::new().unwrap();
        let python_dir = temp_dir.path().join("python");
        let nested_python_dir = python_dir.join("python");

        // Create nested directory structure
        fs::create_dir_all(&nested_python_dir).unwrap();

        // Create Python executable in nested directory
        let python_exe = if cfg!(windows) {
            nested_python_dir.join("python.exe")
        } else {
            nested_python_dir.join("python3")
        };

        fs::write(&python_exe, "fake python executable").unwrap();

        // Test finding the executable
        let result = find_python_executable_in_dir(&python_dir);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), python_exe);
    }

    #[test]
    fn test_find_python_executable_in_dir_direct() {
        let temp_dir = TempDir::new().unwrap();
        let python_dir = temp_dir.path().to_path_buf();

        // Create Python executable directly in directory
        let python_exe = if cfg!(windows) {
            python_dir.join("python.exe")
        } else {
            python_dir.join("python3")
        };

        fs::write(&python_exe, "fake python executable").unwrap();

        // Test finding the executable
        let result = find_python_executable_in_dir(&python_dir);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), python_exe);
    }

    #[test]
    fn test_find_python_executable_in_dir_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let python_dir = temp_dir.path().to_path_buf();

        // Don't create any Python executable
        let result = find_python_executable_in_dir(&python_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Python executable not found"));
    }

    #[test]
    fn test_detect_standalone_rez_installation_not_installed() {
        // This test will fail if there's no home directory, but that's expected
        let result = detect_standalone_rez_installation();
        // Should return Ok with is_installed = false if no installation found
        match result {
            Ok(env) => {
                // If we get an environment, it should be valid
                assert!(env.version.is_some() || !env.is_installed);
            }
            Err(_) => {
                // Error is acceptable if home directory not found or other issues
            }
        }
    }
}
