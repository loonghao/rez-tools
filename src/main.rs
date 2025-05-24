use rez_tools::cli::{commands, CliApp};
use std::process;

#[tokio::main]
async fn main() {
    // Setup basic logging first
    commands::setup_logging(false, false);

    // Create and run the main CLI application
    let app = match CliApp::new() {
        Ok(app) => app,
        Err(e) => {
            eprintln!("Error initializing application: {}", e);
            process::exit(1);
        }
    };

    let exit_code = match app.run().await {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Error: {}", e);
            1
        }
    };

    process::exit(exit_code);
}
