use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::io::Write;
use tempfile::TempDir;

#[test]
fn test_rt_help() {
    let mut cmd = Command::cargo_bin("rt").unwrap();
    cmd.arg("--help");
    cmd.assert().success().stdout(predicate::str::contains(
        "A high-performance Rust command-line tool suite for rez package management",
    ));
}

#[test]
fn test_rt_version() {
    let mut cmd = Command::cargo_bin("rt").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_rt_list_empty() {
    let mut cmd = Command::cargo_bin("rt").unwrap();
    cmd.arg("list");
    cmd.assert().success().stdout(
        predicate::str::contains("No plugins found")
            .or(predicate::str::contains("Available plugins")),
    );
}

#[test]
fn test_rt_with_config() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a test plugin file
    let plugin_file = temp_path.join("test_tool.rt");
    let mut file = fs::File::create(&plugin_file).unwrap();
    writeln!(
        file,
        r#"
command: echo "Hello from test tool"
short_help: Test tool for integration testing
requires:
  - test-package
"#
    )
    .unwrap();

    // Create a config file
    let config_file = temp_path.join("reztoolsconfig.py");
    let mut config = fs::File::create(&config_file).unwrap();
    writeln!(
        config,
        r#"
tool_paths = [
    "{}"
]

extension = ".rt"
"#,
        temp_path.to_string_lossy().replace('\\', "\\\\")
    )
    .unwrap();

    // Test with the config
    let mut cmd = Command::cargo_bin("rt").unwrap();
    cmd.env("REZ_TOOL_CONFIG", config_file.to_string_lossy().to_string());
    cmd.arg("list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test_tool"));
}

#[test]
fn test_rt_plugin_print() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a test plugin file
    let plugin_file = temp_path.join("test_tool.rt");
    let mut file = fs::File::create(&plugin_file).unwrap();
    writeln!(
        file,
        r#"
command: echo "Hello"
short_help: Test tool
requires:
  - test-package
"#
    )
    .unwrap();

    // Create a config file
    let config_file = temp_path.join("reztoolsconfig.py");
    let mut config = fs::File::create(&config_file).unwrap();
    writeln!(
        config,
        r#"
tool_paths = [
    "{}"
]

extension = ".rt"
"#,
        temp_path.to_string_lossy().replace('\\', "\\\\")
    )
    .unwrap();

    // Test plugin --print
    let mut cmd = Command::cargo_bin("rt").unwrap();
    cmd.env("REZ_TOOL_CONFIG", config_file.to_string_lossy().to_string());
    cmd.args(["test_tool", "--print"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("command"))
        .stdout(predicate::str::contains("echo \\\"Hello\\\""));
}

#[test]
fn test_invalid_plugin_name() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a plugin file with invalid name (starts with number)
    let plugin_file = temp_path.join("123invalid.rt");
    let mut file = fs::File::create(&plugin_file).unwrap();
    writeln!(
        file,
        r#"
command: echo "test"
requires:
  - test-package
"#
    )
    .unwrap();

    // Create a config file
    let config_file = temp_path.join("reztoolsconfig.py");
    let mut config = fs::File::create(&config_file).unwrap();
    writeln!(
        config,
        r#"
tool_paths = [
    "{}"
]

extension = ".rt"
"#,
        temp_path.to_string_lossy().replace('\\', "\\\\")
    )
    .unwrap();

    // Test that invalid plugin is not loaded
    let mut cmd = Command::cargo_bin("rt").unwrap();
    cmd.env("REZ_TOOL_CONFIG", config_file.to_string_lossy().to_string());
    cmd.arg("list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("123invalid").not());
}
