use crate::error::{Result, RezToolsError};
use crate::rez::RezCommand;
use log::{debug, info};
use std::process::{Command, Stdio};
use tokio::process::Command as AsyncCommand;

/// Execute a rez command
pub async fn execute_rez_command(rez_cmd: RezCommand) -> Result<i32> {
    let command_parts = rez_cmd.build_command();
    let command_line = shell_escape::escape(command_parts.join(" ").into()).to_string();

    info!("Executing rez command: {}", command_line);
    debug!("Command parts: {:?}", command_parts);

    if rez_cmd.detached {
        execute_detached(&command_parts).await
    } else {
        execute_attached(&command_parts).await
    }
}

/// Execute command in attached mode (wait for completion)
async fn execute_attached(command_parts: &[String]) -> Result<i32> {
    if command_parts.is_empty() {
        return Err(RezToolsError::RezExecutionError(
            "Empty command".to_string(),
        ));
    }

    let mut cmd = AsyncCommand::new(&command_parts[0]);
    cmd.args(&command_parts[1..]);

    // Inherit stdio so the user can interact with the command
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    debug!(
        "Starting attached process: {} {:?}",
        command_parts[0],
        &command_parts[1..]
    );

    let output = cmd.status().await.map_err(|e| {
        RezToolsError::RezExecutionError(format!(
            "Failed to execute command '{}': {}",
            command_parts[0], e
        ))
    })?;

    let exit_code = output.code().unwrap_or(-1);
    debug!("Process exited with code: {}", exit_code);

    Ok(exit_code)
}

/// Execute command in detached mode (don't wait for completion)
async fn execute_detached(command_parts: &[String]) -> Result<i32> {
    if command_parts.is_empty() {
        return Err(RezToolsError::RezExecutionError(
            "Empty command".to_string(),
        ));
    }

    let mut cmd = AsyncCommand::new(&command_parts[0]);
    cmd.args(&command_parts[1..]);

    // Detach from parent process
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());

    debug!(
        "Starting detached process: {} {:?}",
        command_parts[0],
        &command_parts[1..]
    );

    let _child = cmd.spawn().map_err(|e| {
        RezToolsError::RezExecutionError(format!(
            "Failed to spawn detached command '{}': {}",
            command_parts[0], e
        ))
    })?;

    info!("Process started in detached mode");
    Ok(0)
}

/// Execute a rez command synchronously (for compatibility)
pub fn execute_rez_command_sync(rez_cmd: RezCommand) -> Result<i32> {
    let command_parts = rez_cmd.build_command();

    if command_parts.is_empty() {
        return Err(RezToolsError::RezExecutionError(
            "Empty command".to_string(),
        ));
    }

    let mut cmd = Command::new(&command_parts[0]);
    cmd.args(&command_parts[1..]);

    if rez_cmd.detached {
        // For detached mode, spawn and don't wait
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());

        let _child = cmd.spawn().map_err(|e| {
            RezToolsError::RezExecutionError(format!(
                "Failed to spawn detached command '{}': {}",
                command_parts[0], e
            ))
        })?;

        Ok(0)
    } else {
        // For attached mode, inherit stdio and wait
        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());

        let status = cmd.status().map_err(|e| {
            RezToolsError::RezExecutionError(format!(
                "Failed to execute command '{}': {}",
                command_parts[0], e
            ))
        })?;

        Ok(status.code().unwrap_or(-1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::Plugin;
    use std::path::PathBuf;

    fn create_test_plugin() -> Plugin {
        Plugin {
            command: "echo".to_string(),
            name: Some("test".to_string()),
            short_help: Some("Test plugin".to_string()),
            requires: vec!["test-package".to_string()],
            run_detached: false,
            inherits_from: None,
            file_path: PathBuf::from("test.rt"),
        }
    }

    #[test]
    fn test_build_command() {
        let plugin = create_test_plugin();
        let rez_cmd = RezCommand::new(plugin);

        let command = rez_cmd.build_command();

        // The first element should be either "rez" or a path to rez executable
        assert!(command[0] == "rez" || command[0].contains("rez"));
        assert_eq!(command[1], "env");
        assert_eq!(command[2], "-q");
        assert_eq!(command[3], "test-package");
        assert_eq!(command[4], "--");
        assert_eq!(command[5], "echo");
    }

    #[test]
    fn test_build_command_with_args() {
        let plugin = create_test_plugin();
        let rez_cmd =
            RezCommand::new(plugin).with_args(vec!["hello".to_string(), "world".to_string()]);

        let command = rez_cmd.build_command();
        assert_eq!(command[6], "hello");
        assert_eq!(command[7], "world");
    }

    #[test]
    fn test_build_command_ignore_cmd() {
        let plugin = create_test_plugin();
        let rez_cmd = RezCommand::new(plugin)
            .with_ignore_cmd(true)
            .with_args(vec![
                "python".to_string(),
                "-c".to_string(),
                "print('hello')".to_string(),
            ]);

        let command = rez_cmd.build_command();
        assert_eq!(command[5], "python");
        assert_eq!(command[6], "-c");
        assert_eq!(command[7], "print('hello')");
    }
}
