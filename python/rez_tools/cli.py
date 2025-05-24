"""
Command-line interface for rez-tools Python wrapper.
"""

import sys
from typing import List, Optional

from .core import RezTools, ensure_binary_installed


def main(args: Optional[List[str]] = None) -> int:
    """Main entry point for rt command.
    
    Args:
        args: Command line arguments. If None, uses sys.argv[1:]
        
    Returns:
        Exit code
    """
    if args is None:
        args = sys.argv[1:]
    
    # Ensure binary is installed
    if not ensure_binary_installed():
        print("Error: Failed to install rt binary", file=sys.stderr)
        return 1
    
    # Create RezTools instance and run command
    try:
        rt = RezTools()
        result = rt.run(args)
        return result.returncode
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


def convert_config(args: Optional[List[str]] = None) -> int:
    """Entry point for rt-convert-config command.
    
    Args:
        args: Command line arguments. If None, uses sys.argv[1:]
        
    Returns:
        Exit code
    """
    if args is None:
        args = sys.argv[1:]
    
    # Ensure binary is installed
    if not ensure_binary_installed():
        print("Error: Failed to install rt-convert-config binary", file=sys.stderr)
        return 1
    
    # For now, we'll use the main rt binary with a convert-config subcommand
    # In the future, we might want to download the separate rt-convert-config binary
    try:
        rt = RezTools()
        result = rt.run(["convert-config"] + args)
        return result.returncode
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
