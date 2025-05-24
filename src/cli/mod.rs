pub mod commands;

use crate::config::loader::load_config;
use crate::platform::rez_path;
use crate::plugin::scanner::scan_plugins;
use crate::rez::{executor::execute_rez_command_sync, RezCommand};

use log::{debug, info, warn};
use std::collections::HashMap;

/// Main CLI application
pub struct CliApp {
    plugins: HashMap<String, crate::plugin::Plugin>,
}

impl CliApp {
    /// Create a new CLI application
    pub fn new() -> crate::error::Result<Self> {
        // Initialize rez path management
        match rez_path::find_and_set_rez_path() {
            Ok(path) => {
                debug!("Initialized rez path: {}", path.display());
            }
            Err(e) => {
                warn!("Could not initialize rez path: {}", e);
                debug!("Rez commands will fall back to system PATH");
            }
        }

        // Load configuration
        let config = load_config()?;
        debug!("Loaded config: {:?}", config);

        // Scan for plugins
        let plugins = scan_plugins(&config.tool_paths, &config.extension)?;
        info!("Found {} plugins", plugins.len());

        Ok(Self { plugins })
    }



    /// Run the CLI application
    pub async fn run(&self) -> crate::error::Result<i32> {
        // Get command line arguments
        let args: Vec<String> = std::env::args().collect();

        if args.len() < 2 {
            self.print_help();
            return Ok(1);
        }

        let subcommand_name = &args[1];

        // Handle special commands
        match subcommand_name.as_str() {
            "--help" | "-h" => {
                self.print_help();
                return Ok(0);
            }
            "--version" | "-V" => {
                println!("{}", env!("CARGO_PKG_VERSION"));
                return Ok(0);
            }
            "list" => {
                self.list_plugins();
                return Ok(0);
            }
            "install-rez" => {
                return self.handle_install_rez().await;
            }
            "check-rez" => {
                return self.handle_check_rez();
            }
            _ => {}
        }

        // Check if it's a plugin command
        if let Some(plugin) = self.plugins.get(subcommand_name) {
            self.handle_plugin_command(plugin, &args[2..])
        } else {
            eprintln!("Unknown command: {}", subcommand_name);
            self.print_help();
            Ok(1)
        }
    }

    /// Handle a plugin command execution
    fn handle_plugin_command(&self, plugin: &crate::plugin::Plugin, args: &[String]) -> crate::error::Result<i32> {
        let mut ignore_cmd = false;
        let mut run_detached = false;
        let mut print_details = false;
        let mut remaining_args = Vec::new();

        // Parse arguments manually
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--ignore-cmd" => ignore_cmd = true,
                "--run-detached" => run_detached = true,
                "--print" => print_details = true,
                _ => remaining_args.push(args[i].clone()),
            }
            i += 1;
        }

        // Handle --print option
        if print_details {
            self.print_plugin_details(plugin);
            return Ok(0);
        }

        // Build rez command
        let mut rez_cmd = RezCommand::new(plugin.clone());

        // Handle options
        if ignore_cmd {
            rez_cmd = rez_cmd.with_ignore_cmd(true);
        }

        if run_detached {
            rez_cmd = rez_cmd.with_detached(true);
        }

        // Handle additional arguments
        if !remaining_args.is_empty() {
            rez_cmd = rez_cmd.with_args(remaining_args);
        }

        // Execute the command
        execute_rez_command_sync(rez_cmd)
    }

    /// Print help information
    fn print_help(&self) {
        println!("rt {}", env!("CARGO_PKG_VERSION"));
        println!("{}", env!("CARGO_PKG_DESCRIPTION"));
        println!();
        println!("USAGE:");
        println!("    rt [OPTIONS] <COMMAND> [ARGS]...");
        println!();
        println!("OPTIONS:");
        println!("    -h, --help       Print help information");
        println!("    -V, --version    Print version information");
        println!();
        println!("COMMANDS:");
        println!("    list             List all available plugins");

        if !self.plugins.is_empty() {
            println!();
            println!("PLUGIN COMMANDS:");
            for (name, plugin) in &self.plugins {
                println!("    {:<20} {}", name, plugin.get_short_help());
            }
        }

        println!();
        println!("PLUGIN OPTIONS:");
        println!("    --ignore-cmd     Ignore standard tool command when running the command");
        println!("    --print          Print plugin details and exit");
        println!("    --run-detached   Run the command in detached mode");
    }

    /// Print plugin details as JSON
    fn print_plugin_details(&self, plugin: &crate::plugin::Plugin) {
        match serde_json::to_string_pretty(plugin) {
            Ok(json) => println!("{}", json),
            Err(e) => eprintln!("Error serializing plugin details: {}", e),
        }
    }

    /// List all available plugins
    pub fn list_plugins(&self) {
        if self.plugins.is_empty() {
            println!("No plugins found.");
            return;
        }

        println!("Available plugins:");
        for (name, plugin) in &self.plugins {
            println!("  {:<20} {}", name, plugin.get_short_help());
        }
    }

    /// Handle rez installation
    async fn handle_install_rez(&self) -> crate::error::Result<i32> {
        use crate::platform::installer;

        println!("Installing rez...");
        match installer::install_rez().await {
            Ok(()) => {
                println!("✅ Rez installed successfully!");
                println!("You can now use rez commands through rt.");
                Ok(0)
            }
            Err(e) => {
                eprintln!("❌ Failed to install rez: {}", e);
                eprintln!("Please install rez manually or check the documentation.");
                Ok(1)
            }
        }
    }

    /// Handle rez environment check
    fn handle_check_rez(&self) -> crate::error::Result<i32> {
        use crate::platform::detection;

        println!("Checking rez environment...");

        // Show REZ_PATH environment variable
        if let Ok(rez_path_env) = std::env::var("REZ_PATH") {
            println!("🔧 REZ_PATH: {}", rez_path_env);
        }

        // Show unified rez path
        match rez_path::get_rez_path() {
            Ok(path) => {
                println!("🎯 Unified rez path: {}", path.display());
            }
            Err(e) => {
                println!("⚠️  Could not determine rez path: {}", e);
            }
        }

        match detection::detect_rez_environment() {
            Ok(env) => {
                if env.is_installed {
                    println!("✅ Rez is installed");
                    if let Some(ref version) = env.version {
                        println!("   Version: {}", version);
                    }
                    if let Some(ref rez_path) = env.rez_path {
                        println!("   Detected path: {}", rez_path.display());
                    }
                    if !env.packages_path.is_empty() {
                        println!("   Package paths:");
                        for path in &env.packages_path {
                            println!("     - {}", path.display());
                        }
                    }
                } else {
                    println!("❌ Rez is not installed");
                    println!("Run 'rt install-rez' to install rez automatically.");
                }
                Ok(0)
            }
            Err(e) => {
                eprintln!("❌ Error checking rez environment: {}", e);
                Ok(1)
            }
        }
    }
}
