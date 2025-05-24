"""
Core functionality for rez-tools Python wrapper.
"""

import os
import platform
import subprocess
import sys
from pathlib import Path
from typing import List, Optional, Union

import requests
import platformdirs


class RezTools:
    """Python wrapper for the rez-tools Rust binary."""
    
    def __init__(self, binary_path: Optional[Union[str, Path]] = None):
        """Initialize RezTools wrapper.
        
        Args:
            binary_path: Path to the rt binary. If None, will auto-detect.
        """
        self.binary_path = Path(binary_path) if binary_path else get_binary_path()
        
    def run(self, args: List[str], **kwargs) -> subprocess.CompletedProcess:
        """Run rt command with given arguments.
        
        Args:
            args: Command line arguments to pass to rt
            **kwargs: Additional arguments to pass to subprocess.run
            
        Returns:
            CompletedProcess instance
        """
        cmd = [str(self.binary_path)] + args
        return subprocess.run(cmd, **kwargs)
    
    def list_plugins(self) -> List[str]:
        """List available plugins.
        
        Returns:
            List of plugin names
        """
        result = self.run(["list"], capture_output=True, text=True)
        if result.returncode != 0:
            raise RuntimeError(f"Failed to list plugins: {result.stderr}")
        
        plugins = []
        for line in result.stdout.splitlines():
            if line.strip() and not line.startswith("Available plugins:") and not line.startswith("No plugins"):
                # Extract plugin name from "  plugin_name    description" format
                parts = line.strip().split()
                if parts:
                    plugins.append(parts[0])
        
        return plugins
    
    def check_rez(self) -> dict:
        """Check rez environment status.
        
        Returns:
            Dictionary with rez environment information
        """
        result = self.run(["check-rez"], capture_output=True, text=True)
        
        info = {
            "installed": False,
            "version": None,
            "path": None,
            "package_paths": []
        }
        
        for line in result.stdout.splitlines():
            line = line.strip()
            if "Rez is installed" in line:
                info["installed"] = True
            elif line.startswith("Version:"):
                info["version"] = line.split(":", 1)[1].strip()
            elif line.startswith("Path:"):
                info["path"] = line.split(":", 1)[1].strip()
            elif line.startswith("- "):
                info["package_paths"].append(line[2:].strip())
        
        return info
    
    def install_rez(self) -> bool:
        """Install rez automatically.
        
        Returns:
            True if installation succeeded, False otherwise
        """
        result = self.run(["install-rez"], capture_output=True, text=True)
        return result.returncode == 0


def get_platform_info() -> tuple[str, str]:
    """Get platform and architecture information.
    
    Returns:
        Tuple of (platform, architecture)
    """
    system = platform.system().lower()
    machine = platform.machine().lower()
    
    # Normalize platform names
    if system == "darwin":
        system = "macos"
    
    # Normalize architecture names
    if machine in ("x86_64", "amd64"):
        machine = "x86_64"
    elif machine in ("arm64", "aarch64"):
        machine = "aarch64"
    
    return system, machine


def get_binary_name() -> str:
    """Get the expected binary name for the current platform.
    
    Returns:
        Binary filename
    """
    system, arch = get_platform_info()
    
    if system == "windows":
        return f"rt-{system}-{arch}.exe"
    else:
        return f"rt-{system}-{arch}"


def get_binary_path() -> Path:
    """Get the path to the rt binary.
    
    Returns:
        Path to the binary
        
    Raises:
        FileNotFoundError: If binary is not found
    """
    # First check if binary is already installed
    binary_dir = platformdirs.user_data_dir("rez-tools", "rez-tools")
    binary_path = Path(binary_dir) / "rt"
    
    system, _ = get_platform_info()
    if system == "windows":
        binary_path = binary_path.with_suffix(".exe")
    
    if binary_path.exists():
        return binary_path
    
    # Try to find in PATH
    import shutil
    path_binary = shutil.which("rt")
    if path_binary:
        return Path(path_binary)
    
    # Binary not found, try to download it
    if not ensure_binary_installed():
        raise FileNotFoundError("rt binary not found and could not be downloaded")
    
    return binary_path


def ensure_binary_installed() -> bool:
    """Ensure the rt binary is installed.
    
    Returns:
        True if binary is available, False otherwise
    """
    try:
        get_binary_path()
        return True
    except FileNotFoundError:
        pass
    
    # Download binary
    return download_binary()


def download_binary() -> bool:
    """Download the rt binary for the current platform.
    
    Returns:
        True if download succeeded, False otherwise
    """
    system, arch = get_platform_info()
    binary_name = get_binary_name()
    
    # Get latest release info from GitHub
    try:
        response = requests.get(
            "https://api.github.com/repos/loonghao/rez-tools/releases/latest",
            timeout=30
        )
        response.raise_for_status()
        release_info = response.json()
    except Exception as e:
        print(f"Failed to get release info: {e}", file=sys.stderr)
        return False
    
    # Find the appropriate asset
    download_url = None
    asset_name = binary_name
    if system == "windows":
        asset_name += ".zip"
    else:
        asset_name += ".tar.gz"
    
    for asset in release_info.get("assets", []):
        if asset["name"] == asset_name:
            download_url = asset["browser_download_url"]
            break
    
    if not download_url:
        print(f"No binary found for platform {system}-{arch}", file=sys.stderr)
        return False
    
    # Download and extract
    try:
        binary_dir = Path(platformdirs.user_data_dir("rez-tools", "rez-tools"))
        binary_dir.mkdir(parents=True, exist_ok=True)
        
        print(f"Downloading {asset_name}...")
        response = requests.get(download_url, timeout=300)
        response.raise_for_status()
        
        archive_path = binary_dir / asset_name
        archive_path.write_bytes(response.content)
        
        # Extract archive
        if system == "windows":
            import zipfile
            with zipfile.ZipFile(archive_path, 'r') as zip_ref:
                zip_ref.extractall(binary_dir)
        else:
            import tarfile
            with tarfile.open(archive_path, 'r:gz') as tar_ref:
                tar_ref.extractall(binary_dir)
        
        # Make binary executable on Unix systems
        binary_path = binary_dir / "rt"
        if system == "windows":
            binary_path = binary_path.with_suffix(".exe")
        
        if system != "windows":
            binary_path.chmod(0o755)
        
        # Clean up archive
        archive_path.unlink()
        
        print(f"Successfully installed rt binary to {binary_path}")
        return True
        
    except Exception as e:
        print(f"Failed to download binary: {e}", file=sys.stderr)
        return False
