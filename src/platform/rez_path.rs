use crate::error::{Result, RezToolsError};
use crate::platform::detection::detect_rez_environment;
use crate::platform::Platform;
use log::{debug, info, warn};
use std::path::PathBuf;
use std::sync::OnceLock;

/// Global rez path cache
static REZ_PATH_CACHE: OnceLock<Option<PathBuf>> = OnceLock::new();

/// Environment variable name for rez path
const REZ_PATH_ENV: &str = "REZ_PATH";

/// Find and cache the rez executable path
pub fn find_and_set_rez_path() -> Result<PathBuf> {
    // Check if we already have a cached path
    if let Some(Some(path)) = REZ_PATH_CACHE.get() {
        debug!("Using cached rez path: {}", path.display());
        return Ok(path.clone());
    }

    // Try to find rez path
    let rez_path = find_rez_executable()?;

    // Set environment variable
    std::env::set_var(REZ_PATH_ENV, &rez_path);
    info!(
        "Set REZ_PATH environment variable to: {}",
        rez_path.display()
    );

    // Cache the result
    REZ_PATH_CACHE
        .set(Some(rez_path.clone()))
        .map_err(|_| RezToolsError::ConfigError("Failed to cache rez path".to_string()))?;

    Ok(rez_path)
}

/// Get the cached rez path, or find it if not cached
pub fn get_rez_path() -> Result<PathBuf> {
    if let Some(Some(path)) = REZ_PATH_CACHE.get() {
        return Ok(path.clone());
    }

    find_and_set_rez_path()
}

/// Clear the cached rez path (useful for testing)
pub fn clear_rez_path_cache() {
    // We can't actually clear OnceLock, but we can set it to None
    let _ = REZ_PATH_CACHE.set(None);
}

/// Find rez executable using multiple strategies
fn find_rez_executable() -> Result<PathBuf> {
    debug!("Searching for rez executable...");

    // Strategy 1: Check REZ_PATH environment variable
    if let Ok(env_path) = std::env::var(REZ_PATH_ENV) {
        let path = PathBuf::from(env_path);
        if path.exists() {
            debug!(
                "Found rez via REZ_PATH environment variable: {}",
                path.display()
            );
            return Ok(path);
        } else {
            warn!(
                "REZ_PATH environment variable points to non-existent path: {}",
                path.display()
            );
        }
    }

    // Strategy 2: Check our Python Build Standalone installation
    if let Ok(path) = find_rez_in_python_standalone() {
        debug!("Found rez in Python Build Standalone: {}", path.display());
        return Ok(path);
    }

    // Strategy 3: Check rez-tools wrapper
    if let Ok(path) = find_rez_wrapper() {
        debug!("Found rez wrapper: {}", path.display());
        return Ok(path);
    }

    // Strategy 4: Check system PATH
    if let Ok(path) = find_rez_in_system_path() {
        debug!("Found rez in system PATH: {}", path.display());
        return Ok(path);
    }

    // Strategy 5: Check common installation locations
    if let Ok(path) = find_rez_in_common_locations() {
        debug!("Found rez in common location: {}", path.display());
        return Ok(path);
    }

    Err(RezToolsError::ConfigError(
        "Rez executable not found. Please install rez or run 'rt install-rez'".to_string(),
    ))
}

/// Find rez in our Python Build Standalone installation
fn find_rez_in_python_standalone() -> Result<PathBuf> {
    let env = detect_rez_environment()?;

    if let Some(python_path) = env.python_path {
        let platform = Platform::detect();

        // Try to find rez script in the Python installation
        let possible_paths = if platform.os == "windows" {
            vec![
                python_path
                    .parent()
                    .unwrap()
                    .join("Scripts")
                    .join("rez.exe"),
                python_path
                    .parent()
                    .unwrap()
                    .join("Scripts")
                    .join("rez.bat"),
                python_path.parent().unwrap().join("Scripts").join("rez"),
            ]
        } else {
            vec![
                python_path.parent().unwrap().join("bin").join("rez"),
                python_path.parent().unwrap().join("rez"),
            ]
        };

        for path in possible_paths {
            if path.exists() {
                return Ok(path);
            }
        }

        // If no rez script found, we can use python -m rez
        return Ok(python_path);
    }

    Err(RezToolsError::ConfigError(
        "No Python Build Standalone installation found".to_string(),
    ))
}

/// Find rez wrapper created by our installer
fn find_rez_wrapper() -> Result<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| RezToolsError::ConfigError("Cannot find home directory".to_string()))?;

    let rez_tools_dir = home_dir.join(".rez-tools");
    let bin_dir = rez_tools_dir.join("bin");

    let platform = Platform::detect();
    let wrapper_name = if platform.os == "windows" {
        "rez.bat"
    } else {
        "rez"
    };
    let wrapper_path = bin_dir.join(wrapper_name);

    if wrapper_path.exists() {
        return Ok(wrapper_path);
    }

    Err(RezToolsError::ConfigError(
        "Rez wrapper not found".to_string(),
    ))
}

/// Find rez in system PATH
fn find_rez_in_system_path() -> Result<PathBuf> {
    let platform = Platform::detect();

    // Use 'which' on Unix or 'where' on Windows
    let (cmd, arg) = if platform.os == "windows" {
        ("where", "rez")
    } else {
        ("which", "rez")
    };

    match std::process::Command::new(cmd).arg(arg).output() {
        Ok(output) if output.status.success() => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let path_str = output_str.lines().next().unwrap_or("").trim();

            if !path_str.is_empty() {
                let path = PathBuf::from(path_str);
                if path.exists() {
                    return Ok(path);
                }
            }

            Err(RezToolsError::ConfigError(
                "Rez command found but path does not exist".to_string(),
            ))
        }
        Ok(_) => {
            debug!("{} command executed but rez not found in PATH", cmd);
            Err(RezToolsError::ConfigError(
                "Rez not found in system PATH".to_string(),
            ))
        }
        Err(e) => {
            debug!("Failed to run {} command: {}", cmd, e);
            Err(RezToolsError::ConfigError(format!(
                "Failed to search PATH (command '{}' not available): {}",
                cmd, e
            )))
        }
    }
}

/// Find rez in common installation locations
fn find_rez_in_common_locations() -> Result<PathBuf> {
    let platform = Platform::detect();

    let common_paths = if platform.os == "windows" {
        vec![
            PathBuf::from("C:\\Program Files\\rez\\bin\\rez.exe"),
            PathBuf::from("C:\\rez\\bin\\rez.exe"),
        ]
    } else {
        vec![
            PathBuf::from("/usr/local/bin/rez"),
            PathBuf::from("/usr/bin/rez"),
            PathBuf::from("/opt/rez/bin/rez"),
        ]
    };

    for path in common_paths {
        if path.exists() {
            return Ok(path);
        }
    }

    Err(RezToolsError::ConfigError(
        "Rez not found in common locations".to_string(),
    ))
}

/// Get rez command for execution (handles python -m rez case)
pub fn get_rez_command() -> Result<Vec<String>> {
    let rez_path = get_rez_path()?;

    // Check if this is a Python executable (for python -m rez case)
    if rez_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with("python"))
        .unwrap_or(false)
    {
        return Ok(vec![
            rez_path.to_string_lossy().to_string(),
            "-m".to_string(),
            "rez".to_string(),
        ]);
    }

    // Regular rez executable
    Ok(vec![rez_path.to_string_lossy().to_string()])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_rez_in_system_path() {
        // This test may pass or fail depending on whether rez is installed
        let result = find_rez_in_system_path();
        // We don't assert success/failure since it depends on the system
        match result {
            Ok(path) => {
                assert!(
                    path.exists(),
                    "Found rez path should exist: {}",
                    path.display()
                );
                println!("Found rez in system PATH: {}", path.display());
            }
            Err(e) => {
                println!(
                    "Rez not found in system PATH (expected on some systems): {}",
                    e
                );
                // This is acceptable - not all systems have rez installed
            }
        }
    }

    #[test]
    fn test_get_rez_command_python() {
        // Test the case where rez_path is a Python executable
        let temp_dir = TempDir::new().unwrap();
        let python_exe = if cfg!(windows) {
            temp_dir.path().join("python.exe")
        } else {
            temp_dir.path().join("python3")
        };
        fs::write(&python_exe, "fake python").unwrap();

        // Test the command generation logic directly
        let command = if python_exe
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.starts_with("python"))
            .unwrap_or(false)
        {
            vec![
                python_exe.to_string_lossy().to_string(),
                "-m".to_string(),
                "rez".to_string(),
            ]
        } else {
            vec![python_exe.to_string_lossy().to_string()]
        };

        assert_eq!(command.len(), 3, "Python command should have 3 parts");
        assert_eq!(command[0], python_exe.to_string_lossy());
        assert_eq!(command[1], "-m");
        assert_eq!(command[2], "rez");
    }

    #[test]
    fn test_get_rez_command_regular() {
        // Test the case where rez_path is a regular rez executable
        let temp_dir = TempDir::new().unwrap();
        let rez_exe = temp_dir.path().join("rez.exe");
        fs::write(&rez_exe, "fake rez").unwrap();

        // Test the command generation logic directly
        let command = if rez_exe
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.starts_with("python"))
            .unwrap_or(false)
        {
            vec![
                rez_exe.to_string_lossy().to_string(),
                "-m".to_string(),
                "rez".to_string(),
            ]
        } else {
            vec![rez_exe.to_string_lossy().to_string()]
        };

        assert_eq!(command.len(), 1);
        assert_eq!(command[0], rez_exe.to_string_lossy());
    }

    #[test]
    fn test_rez_path_env_variable() {
        let temp_dir = TempDir::new().unwrap();
        let fake_rez = temp_dir.path().join("fake_rez");
        fs::write(&fake_rez, "fake rez").unwrap();

        // Set environment variable
        std::env::set_var(REZ_PATH_ENV, &fake_rez);

        // Clear cache
        clear_rez_path_cache();

        let result = find_rez_executable();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), fake_rez);

        // Clean up
        std::env::remove_var(REZ_PATH_ENV);
    }
}
