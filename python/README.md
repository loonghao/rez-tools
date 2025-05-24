# rez-tools

A high-performance suite tool command line for [rez](https://github.com/nerdvegas/rez), written in Rust with Python bindings.

## Installation

### Via pip (Recommended)

```bash
pip install rez-tools
```

This will automatically download and install the appropriate binary for your platform.

### Via conda

```bash
conda install -c conda-forge rez-tools
```

### Manual Installation

Download the latest binary from the [releases page](https://github.com/loonghao/rez-tools/releases) and place it in your PATH.

## Quick Start

### Check if rez is installed

```bash
rt check-rez
```

### Install rez automatically (if not present)

```bash
rt install-rez
```

### List available tools

```bash
rt list
```

### Run a tool

```bash
rt maya
rt python --ignore-cmd python -c "print('Hello from rez!')"
```

## Configuration

rez-tools supports both Python and TOML configuration formats:

### Python Configuration (reztoolsconfig.py)

```python
import os

tool_paths = [
    os.path.expanduser("~/packages"),
    "/shared/tools",
]

extension = ".rt"
```

### TOML Configuration (reztoolsconfig.toml)

```toml
extension = ".rt"

tool_paths = [
    "~/packages",
    "/shared/tools",
]
```

### Convert Configuration

Convert existing Python config to TOML:

```bash
rt-convert-config reztoolsconfig.py reztoolsconfig.toml
```

## Tool Definition (.rt files)

```yaml
# maya.rt
command: maya
short_help: Autodesk Maya
requires:
  - maya-2023
  - maya_usd
run_detached: true
```

## Python API

```python
from rez_tools import RezTools

# Create instance
rt = RezTools()

# List plugins
plugins = rt.list_plugins()
print(f"Available plugins: {plugins}")

# Check rez environment
rez_info = rt.check_rez()
if rez_info["installed"]:
    print(f"Rez version: {rez_info['version']}")
else:
    print("Installing rez...")
    rt.install_rez()

# Run a command
result = rt.run(["maya", "--help"])
```

## Platform Support

- **Windows** (x86_64)
- **Linux** (x86_64)
- **macOS** (x86_64, ARM64)

## Performance

Compared to the original Python implementation:

- **10x faster startup time**
- **5x lower memory usage**
- **Single binary deployment**
- **Zero Python dependencies** (for the binary)

## License

MIT License - see [LICENSE](LICENSE) file for details.
