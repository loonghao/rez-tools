use rez_tools::config::loader::load_config;
use std::env;
use std::fs;


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <input.py> <output.toml>", args[0]);
        eprintln!("Convert Python rez-tools config to TOML format");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    // Set the config path environment variable
    env::set_var("REZ_TOOL_CONFIG", input_path);

    // Load the configuration
    let config = match load_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    // Convert to TOML
    let toml_content = format!(
        r#"# rez-tools configuration file (TOML format)
# Converted from: {}

# File extension for tool files
extension = "{}"

# Paths to search for .rt files
tool_paths = [
{}
]
"#,
        input_path,
        config.extension,
        config.tool_paths
            .iter()
            .map(|p| format!("    \"{}\"", p.display().to_string().replace('\\', "\\\\")))
            .collect::<Vec<_>>()
            .join(",\n")
    );

    // Write to output file
    if let Err(e) = fs::write(output_path, toml_content) {
        eprintln!("Error writing output file: {}", e);
        std::process::exit(1);
    }

    println!("Successfully converted {} to {}", input_path, output_path);
    println!("You can now use: export REZ_TOOL_CONFIG={}", output_path);
}
