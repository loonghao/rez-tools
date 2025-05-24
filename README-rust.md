# rez-tools (Rust Implementation)

A high-performance, cross-platform suite tool command line for [rez](https://github.com/nerdvegas/rez), rewritten in Rust.

## Features

- 🚀 **High Performance**: Written in Rust for maximum speed and efficiency
- 📦 **Single Binary**: Statically linked executable with no external dependencies
- 🔧 **Full Compatibility**: 100% compatible with existing `.rt` YAML configuration files
- 🌐 **Cross Platform**: Supports Windows, Linux, and macOS
- 🛡️ **Memory Safe**: Rust's ownership system prevents common bugs and security issues
- ⚡ **Fast Startup**: Minimal overhead for quick command execution

## Installation

### From Release (Recommended)

Download the latest binary for your platform from the [releases page](https://github.com/loonghao/rez-tools/releases).

### From Source

```bash
# Clone the repository
git clone https://github.com/loonghao/rez-tools.git
cd rez-tools

# Switch to the Rust implementation branch
git checkout rust-rewrite

# Build the release binary
cargo build --release

# The binary will be available at target/release/rt (or rt.exe on Windows)
```

### Using Cargo

```bash
cargo install --git https://github.com/loonghao/rez-tools.git --branch rust-rewrite
```

## Quick Start

1. **Set up configuration** (optional):
   ```bash
   export REZ_TOOL_CONFIG=/path/to/your/reztoolsconfig.py
   ```

2. **List available tools**:
   ```bash
   rt list
   ```

3. **Run a tool**:
   ```bash
   rt maya
   rt python --ignore-cmd python -c "print('Hello from rez!')"
   ```

## Configuration

The Rust implementation maintains full compatibility with the existing Python configuration format:

### reztoolsconfig.py
```python
import os

# Paths to search for .rt files
tool_paths = [
    os.path.normpath(os.path.expanduser("~/packages")),
    "/shared/tools",
]

# File extension for tool files
extension = ".rt"
```

### Tool Definition (.rt files)

```yaml
# maya.rt
command: maya
short_help: Autodesk Maya
requires:
  - maya-2023
  - maya_usd
  - maya_mtoa
run_detached: true
```

## Command Line Interface

### Global Options

- `-v, --verbose`: Enable verbose logging
- `-q, --quiet`: Suppress output
- `--help`: Show help information
- `--version`: Show version information

### Tool Options

Every tool supports these hidden options:

- `--ignore-cmd`: Ignore the standard tool command and use provided arguments as the command
- `--print`: Print plugin details in JSON format and exit
- `--run-detached`: Run the command in detached mode

### Examples

```bash
# List all available tools
rt list

# Run Maya
rt maya

# Run Python with custom script
rt python --ignore-cmd python -c "print('Hello World')"

# Print tool details
rt maya --print

# Run tool in detached mode
rt maya --run-detached
```

## Migration from Python Version

The Rust implementation is designed as a drop-in replacement for the Python version:

1. **Same Configuration**: Uses the same `reztoolsconfig.py` format
2. **Same .rt Files**: No changes needed to existing tool definitions
3. **Same CLI**: Identical command-line interface and options
4. **Same Behavior**: Maintains the same execution semantics

### Performance Improvements

- **Startup Time**: ~10x faster startup compared to Python version
- **Memory Usage**: ~5x lower memory footprint
- **Binary Size**: Single ~5MB executable vs Python + dependencies
- **Cross-Platform**: No need for Python runtime on target machines

## Development

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run integration tests
cargo test --test integration_tests
```

### Project Structure

```
src/
├── main.rs              # Main entry point
├── lib.rs               # Library root
├── cli/                 # Command-line interface
│   ├── mod.rs
│   └── commands.rs
├── config/              # Configuration handling
│   ├── mod.rs
│   └── loader.rs
├── plugin/              # Plugin system
│   ├── mod.rs
│   ├── parser.rs
│   └── scanner.rs
├── rez/                 # Rez integration
│   ├── mod.rs
│   └── executor.rs
└── error.rs             # Error handling
```

### Testing

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration_tests

# Test with coverage
cargo tarpaulin --out Html
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests for your changes
5. Ensure all tests pass (`cargo test`)
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Original Python implementation by [Long Hao](https://github.com/loonghao)
- [rez](https://github.com/nerdvegas/rez) package manager by Allan Johns
- Rust community for excellent tooling and libraries
