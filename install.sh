#!/bin/bash
# rez-tools installation script for Unix systems

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO="loonghao/rez-tools"
INSTALL_DIR="${HOME}/.local/bin"
BINARY_NAME="rt"

# Detect platform
detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)
    
    case "$os" in
        linux*)
            os="linux"
            ;;
        darwin*)
            os="macos"
            ;;
        *)
            echo -e "${RED}Error: Unsupported operating system: $os${NC}"
            exit 1
            ;;
    esac
    
    case "$arch" in
        x86_64|amd64)
            arch="x86_64"
            ;;
        arm64|aarch64)
            arch="aarch64"
            ;;
        *)
            echo -e "${RED}Error: Unsupported architecture: $arch${NC}"
            exit 1
            ;;
    esac
    
    echo "${os}-${arch}"
}

# Get latest release info
get_latest_release() {
    local api_url="https://api.github.com/repos/${REPO}/releases/latest"
    
    if command -v curl >/dev/null 2>&1; then
        curl -s "$api_url"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- "$api_url"
    else
        echo -e "${RED}Error: Neither curl nor wget is available${NC}"
        exit 1
    fi
}

# Download and install binary
install_binary() {
    local platform="$1"
    local asset_name="rt-${platform}.tar.gz"
    
    echo -e "${BLUE}Detecting platform: ${platform}${NC}"
    
    # Get release info
    echo -e "${BLUE}Fetching latest release information...${NC}"
    local release_info=$(get_latest_release)
    local download_url=$(echo "$release_info" | grep -o "\"browser_download_url\":[[:space:]]*\"[^\"]*${asset_name}\"" | cut -d'"' -f4)
    
    if [ -z "$download_url" ]; then
        echo -e "${RED}Error: Could not find download URL for ${asset_name}${NC}"
        echo -e "${YELLOW}Available assets:${NC}"
        echo "$release_info" | grep -o "\"name\":[[:space:]]*\"[^\"]*\"" | cut -d'"' -f4
        exit 1
    fi
    
    # Create install directory
    mkdir -p "$INSTALL_DIR"
    
    # Download binary
    echo -e "${BLUE}Downloading ${asset_name}...${NC}"
    local temp_file=$(mktemp)
    
    if command -v curl >/dev/null 2>&1; then
        curl -L -o "$temp_file" "$download_url"
    elif command -v wget >/dev/null 2>&1; then
        wget -O "$temp_file" "$download_url"
    fi
    
    # Extract binary
    echo -e "${BLUE}Extracting binary...${NC}"
    tar -xzf "$temp_file" -C "$INSTALL_DIR"
    rm "$temp_file"
    
    # Make executable
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    
    echo -e "${GREEN}✅ Successfully installed rez-tools to ${INSTALL_DIR}/${BINARY_NAME}${NC}"
}

# Check if binary is in PATH
check_path() {
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo -e "${YELLOW}Warning: ${INSTALL_DIR} is not in your PATH${NC}"
        echo -e "${YELLOW}Add the following line to your shell profile:${NC}"
        echo -e "${BLUE}export PATH=\"\$PATH:${INSTALL_DIR}\"${NC}"
        echo
    fi
}

# Test installation
test_installation() {
    echo -e "${BLUE}Testing installation...${NC}"
    if "${INSTALL_DIR}/${BINARY_NAME}" --version >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Installation test passed${NC}"
        echo -e "${BLUE}Run '${BINARY_NAME} --help' to get started${NC}"
    else
        echo -e "${RED}❌ Installation test failed${NC}"
        exit 1
    fi
}

# Main installation process
main() {
    echo -e "${BLUE}rez-tools Installation Script${NC}"
    echo "=============================="
    echo
    
    # Check dependencies
    if ! command -v tar >/dev/null 2>&1; then
        echo -e "${RED}Error: tar is required but not installed${NC}"
        exit 1
    fi
    
    # Detect platform and install
    local platform=$(detect_platform)
    install_binary "$platform"
    
    # Check PATH and test
    check_path
    test_installation
    
    echo
    echo -e "${GREEN}🎉 rez-tools has been successfully installed!${NC}"
    echo
    echo -e "${BLUE}Next steps:${NC}"
    echo "1. Add ${INSTALL_DIR} to your PATH if not already done"
    echo "2. Run 'rt check-rez' to verify your rez environment"
    echo "3. Run 'rt install-rez' if rez is not installed"
    echo "4. Run 'rt list' to see available tools"
}

# Run main function
main "$@"
