pub mod executor;

use crate::plugin::Plugin;
use crate::platform::rez_path;
use std::process::Command;

/// Represents a rez command to be executed
#[derive(Debug, Clone)]
pub struct RezCommand {
    /// The plugin this command is based on
    pub plugin: Plugin,
    /// Additional arguments passed to the command
    pub args: Vec<String>,
    /// Whether to run detached
    pub detached: bool,
    /// Whether to ignore the default command and use args as the command
    pub ignore_cmd: bool,
}

impl RezCommand {
    /// Create a new RezCommand from a plugin
    pub fn new(plugin: Plugin) -> Self {
        Self {
            detached: plugin.run_detached,
            plugin,
            args: Vec::new(),
            ignore_cmd: false,
        }
    }

    /// Set additional arguments
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Set detached mode
    pub fn with_detached(mut self, detached: bool) -> Self {
        self.detached = detached;
        self
    }

    /// Set ignore command mode
    pub fn with_ignore_cmd(mut self, ignore_cmd: bool) -> Self {
        self.ignore_cmd = ignore_cmd;
        self
    }

    /// Build the complete rez command line
    pub fn build_command(&self) -> Vec<String> {
        // Use the unified rez path management
        let mut command = match rez_path::get_rez_command() {
            Ok(rez_cmd) => rez_cmd,
            Err(_) => {
                // Fallback to system rez command
                vec!["rez".to_string()]
            }
        };

        // Add rez env arguments
        command.extend(vec!["env".to_string(), "-q".to_string()]);

        // Add required packages
        command.extend(self.plugin.requires.iter().cloned());

        // Add separator
        command.push("--".to_string());

        // Add the actual command to execute
        if self.ignore_cmd && !self.args.is_empty() {
            // Use args as the command
            command.extend(self.args.iter().cloned());
        } else {
            // Use the plugin's command
            command.push(self.plugin.command.clone());
            // Add any additional args
            command.extend(self.args.iter().cloned());
        }

        command
    }



    /// Convert to a Command object for execution
    pub fn to_command(&self) -> Command {
        let command_parts = self.build_command();
        let mut cmd = Command::new(&command_parts[0]);
        cmd.args(&command_parts[1..]);
        cmd
    }
}
