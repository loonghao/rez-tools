use crate::error::{Result, RezToolsError};
use crate::platform::{python_standalone::PythonStandalone, Platform};
use log::{debug, info, warn};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::process::Command as AsyncCommand;

/// Install rez using the best available method
pub async fn install_rez() -> Result<()> {
    info!("Installing rez...");

    // Try different installation methods in order of preference
    if try_install_with_uv().await.is_ok() {
        info!("Successfully installed rez using uv");
        return Ok(());
    }

    if try_install_with_pip().await.is_ok() {
        info!("Successfully installed rez using pip");
        return Ok(());
    }

    if try_install_python_build_standalone().await.is_ok() {
        info!("Successfully installed Python Build Standalone with rez");
        return Ok(());
    }

    Err(RezToolsError::ConfigError(
        "Failed to install rez using any available method".to_string(),
    ))
}

/// Try to install rez using uv
async fn try_install_with_uv() -> Result<()> {
    debug!("Attempting to install rez with uv");

    // Check if uv is available
    let uv_check = AsyncCommand::new("uv").arg("--version").output().await;

    if let Err(e) = uv_check {
        debug!("uv not found: {}", e);
        return Err(RezToolsError::ConfigError("uv not found".to_string()));
    }

    // Create virtual environment
    let venv_path = get_rez_tools_dir().join("venv");

    let output = AsyncCommand::new("uv")
        .args(["venv", venv_path.to_string_lossy().as_ref()])
        .output()
        .await
        .map_err(|e| RezToolsError::ConfigError(format!("Failed to create venv: {}", e)))?;

    if !output.status.success() {
        return Err(RezToolsError::ConfigError(format!(
            "uv venv failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    // Install rez in the virtual environment
    let pip_path = get_venv_pip_path(&venv_path)?;
    let output = AsyncCommand::new(&pip_path)
        .args(["install", "rez"])
        .output()
        .await
        .map_err(|e| RezToolsError::ConfigError(format!("Failed to install rez: {}", e)))?;

    if !output.status.success() {
        return Err(RezToolsError::ConfigError(format!(
            "pip install rez failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    // Find the Python executable in the venv
    let platform = Platform::detect();
    let python_exe = if platform.os == "windows" {
        venv_path.join("Scripts").join("python.exe")
    } else {
        venv_path.join("bin").join("python")
    };

    // Create production install marker to avoid pip installation warnings
    create_rez_production_marker(&python_exe).await?;

    // Create rez wrapper script
    create_rez_wrapper(&python_exe).await?;

    Ok(())
}

/// Try to install rez using system pip
async fn try_install_with_pip() -> Result<()> {
    debug!("Attempting to install rez with pip");

    // Check if we're in a virtual environment
    let in_venv =
        std::env::var("VIRTUAL_ENV").is_ok() || std::env::var("CONDA_DEFAULT_ENV").is_ok();

    let mut args = vec!["install"];
    if !in_venv {
        args.push("--user");
    }
    args.push("rez");

    let output = AsyncCommand::new("pip")
        .args(&args)
        .output()
        .await
        .map_err(|e| {
            debug!("Failed to run pip: {}", e);
            RezToolsError::ConfigError(format!("Failed to run pip: {}", e))
        })?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        debug!("pip install failed: {}", stderr);
        Err(RezToolsError::ConfigError(format!(
            "pip install failed: {}",
            stderr
        )))
    }
}

/// Install Python Build Standalone and then rez
async fn try_install_python_build_standalone() -> Result<()> {
    debug!("Attempting to install Python Build Standalone");

    let install_dir = get_rez_tools_dir().join("python-build-standalone");
    let python_standalone = PythonStandalone::new(install_dir);

    // Check if already installed
    if python_standalone.is_installed().await {
        info!("Python Build Standalone already installed");
        let python_exe = python_standalone.get_python_executable()?;
        return install_rez_with_python_exe(&python_exe).await;
    }

    // Install Python Build Standalone
    let python_exe = python_standalone.install().await?;

    // Install rez using the standalone Python
    install_rez_with_python_exe(&python_exe).await
}

/// Install rez using a specific Python executable
async fn install_rez_with_python_exe(python_exe: &PathBuf) -> Result<()> {
    info!("Installing rez using Python: {}", python_exe.display());

    let output = AsyncCommand::new(python_exe)
        .args(["-m", "pip", "install", "rez"])
        .output()
        .await
        .map_err(|e| RezToolsError::ConfigError(format!("Failed to run pip: {}", e)))?;

    if !output.status.success() {
        return Err(RezToolsError::ConfigError(format!(
            "Failed to install rez: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    // Create production install marker to avoid pip installation warnings
    create_rez_production_marker(python_exe).await?;

    // Create rez wrapper script
    create_rez_wrapper(python_exe).await?;

    info!("Successfully installed rez");
    Ok(())
}

/// Create a rez wrapper script that uses our Python installation
async fn create_rez_wrapper(python_exe: &Path) -> Result<()> {
    let platform = Platform::detect();
    let rez_tools_dir = get_rez_tools_dir();
    let bin_dir = rez_tools_dir.join("bin");

    // Create bin directory
    fs::create_dir_all(&bin_dir).await?;

    let wrapper_name = if platform.os == "windows" {
        "rez.bat"
    } else {
        "rez"
    };
    let wrapper_path = bin_dir.join(wrapper_name);

    let wrapper_content = if platform.os == "windows" {
        format!("@echo off\n\"{}\" -m rez %*\n", python_exe.display())
    } else {
        format!(
            "#!/bin/bash\nexec \"{}\" -m rez \"$@\"\n",
            python_exe.display()
        )
    };

    fs::write(&wrapper_path, wrapper_content).await?;

    // Make executable on Unix systems
    if platform.os != "windows" {
        let output = AsyncCommand::new("chmod")
            .args(["+x", &wrapper_path.to_string_lossy()])
            .output()
            .await?;

        if !output.status.success() {
            warn!("Failed to make rez wrapper executable");
        }
    }

    info!("Created rez wrapper at: {}", wrapper_path.display());

    // Provide instructions to user
    if platform.os == "windows" {
        info!(
            "Add {} to your PATH to use 'rez' command directly",
            bin_dir.display()
        );
    } else {
        info!(
            "Add {} to your PATH to use 'rez' command directly",
            bin_dir.display()
        );
        info!("Or run: export PATH=\"{}:$PATH\"", bin_dir.display());
    }

    Ok(())
}

/// Create .rez_production_install marker file to avoid pip installation warnings
async fn create_rez_production_marker(python_exe: &Path) -> Result<()> {
    let platform = Platform::detect();

    // Find the Scripts/bin directory where rez is installed
    let python_dir = python_exe
        .parent()
        .ok_or_else(|| RezToolsError::ConfigError("Invalid Python executable path".to_string()))?;

    let scripts_dir = if platform.os == "windows" {
        python_dir.join("Scripts")
    } else {
        python_dir.join("bin")
    };

    let marker_file = scripts_dir.join(".rez_production_install");

    // Create the marker file (empty file)
    fs::write(&marker_file, "").await?;

    info!(
        "Created rez production install marker: {}",
        marker_file.display()
    );
    debug!("This marker file prevents rez from showing pip installation warnings");

    Ok(())
}

/// Get rez-tools directory
fn get_rez_tools_dir() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        home.join(".rez-tools")
    } else {
        PathBuf::from(".rez-tools")
    }
}

/// Get pip path in virtual environment
fn get_venv_pip_path(venv_path: &Path) -> Result<PathBuf> {
    let platform = Platform::detect();
    let pip_path = if platform.os == "windows" {
        venv_path.join("Scripts").join("pip.exe")
    } else {
        venv_path.join("bin").join("pip")
    };

    if pip_path.exists() {
        Ok(pip_path)
    } else {
        Err(RezToolsError::ConfigError(
            "pip not found in venv".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_get_rez_tools_dir() {
        let dir = get_rez_tools_dir();
        assert!(dir.to_string_lossy().contains(".rez-tools"));
    }

    #[tokio::test]
    async fn test_create_rez_production_marker() {
        let temp_dir = TempDir::new().unwrap();
        let python_exe = temp_dir.path().join("python").join("python.exe");
        let scripts_dir = temp_dir.path().join("python").join("Scripts");

        // Create directory structure using async fs
        tokio::fs::create_dir_all(&scripts_dir).await.unwrap();
        tokio::fs::write(&python_exe, "fake python").await.unwrap();

        let result = create_rez_production_marker(&python_exe).await;
        assert!(result.is_ok());

        // Check that marker file was created
        let marker_file = scripts_dir.join(".rez_production_install");
        assert!(marker_file.exists());

        // Check that it's empty
        let content = tokio::fs::read_to_string(&marker_file).await.unwrap();
        assert!(content.is_empty());
    }

    #[tokio::test]
    async fn test_create_rez_production_marker_unix() {
        // Skip this test on Windows since it tests Unix-specific behavior
        if cfg!(windows) {
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let python_exe = temp_dir.path().join("python").join("bin").join("python3");
        let bin_dir = temp_dir.path().join("python").join("bin");

        // Create directory structure using async fs
        tokio::fs::create_dir_all(&bin_dir).await.unwrap();
        tokio::fs::write(&python_exe, "fake python").await.unwrap();

        let result = create_rez_production_marker(&python_exe).await;
        assert!(result.is_ok());

        // Check that marker file was created
        let marker_file = bin_dir.join(".rez_production_install");
        assert!(marker_file.exists());
    }

    #[tokio::test]
    async fn test_create_rez_production_marker_invalid_path() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_python_exe = temp_dir.path().join("nonexistent").join("python.exe");

        let result = create_rez_production_marker(&invalid_python_exe).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_get_venv_pip_path_windows() {
        let temp_dir = TempDir::new().unwrap();
        let venv_path = temp_dir.path().to_path_buf();
        let scripts_dir = venv_path.join("Scripts");
        let pip_exe = scripts_dir.join("pip.exe");

        // Create directory structure and pip executable
        fs::create_dir_all(&scripts_dir).unwrap();
        fs::write(&pip_exe, "fake pip").unwrap();

        if cfg!(windows) {
            let result = get_venv_pip_path(&venv_path);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), pip_exe);
        }
    }

    #[test]
    fn test_get_venv_pip_path_unix() {
        let temp_dir = TempDir::new().unwrap();
        let venv_path = temp_dir.path().to_path_buf();
        let bin_dir = venv_path.join("bin");
        let pip_exe = bin_dir.join("pip");

        // Create directory structure and pip executable
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(&pip_exe, "fake pip").unwrap();

        if !cfg!(windows) {
            let result = get_venv_pip_path(&venv_path);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), pip_exe);
        }
    }

    #[test]
    fn test_get_venv_pip_path_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let venv_path = temp_dir.path().to_path_buf();

        // Don't create pip executable
        let result = get_venv_pip_path(&venv_path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("pip not found in venv"));
    }

    #[test]
    fn test_virtual_environment_detection() {
        // Test virtual environment detection logic
        let original_venv = std::env::var("VIRTUAL_ENV").ok();
        let original_conda = std::env::var("CONDA_DEFAULT_ENV").ok();

        // Test with VIRTUAL_ENV set
        std::env::set_var("VIRTUAL_ENV", "/path/to/venv");
        let in_venv =
            std::env::var("VIRTUAL_ENV").is_ok() || std::env::var("CONDA_DEFAULT_ENV").is_ok();
        assert!(in_venv);

        // Test with CONDA_DEFAULT_ENV set
        std::env::remove_var("VIRTUAL_ENV");
        std::env::set_var("CONDA_DEFAULT_ENV", "myenv");
        let in_conda =
            std::env::var("VIRTUAL_ENV").is_ok() || std::env::var("CONDA_DEFAULT_ENV").is_ok();
        assert!(in_conda);

        // Test with neither set
        std::env::remove_var("VIRTUAL_ENV");
        std::env::remove_var("CONDA_DEFAULT_ENV");
        let not_in_venv =
            std::env::var("VIRTUAL_ENV").is_ok() || std::env::var("CONDA_DEFAULT_ENV").is_ok();
        assert!(!not_in_venv);

        // Restore original environment
        if let Some(venv) = original_venv {
            std::env::set_var("VIRTUAL_ENV", venv);
        }
        if let Some(conda) = original_conda {
            std::env::set_var("CONDA_DEFAULT_ENV", conda);
        }
    }

    #[test]
    fn test_pip_args_construction() {
        // Test pip arguments construction based on virtual environment
        let base_args = vec!["install"];

        // In virtual environment - no --user flag
        let in_venv = true;
        let mut args = base_args.clone();
        if !in_venv {
            args.push("--user");
        }
        args.push("rez");

        assert_eq!(args, vec!["install", "rez"]);

        // Not in virtual environment - add --user flag
        let in_venv = false;
        let mut args = base_args.clone();
        if !in_venv {
            args.push("--user");
        }
        args.push("rez");

        assert_eq!(args, vec!["install", "--user", "rez"]);
    }
}
