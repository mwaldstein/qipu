#!/usr/bin/env bash
# Qipu installer script for Unix systems (macOS and Linux)
# Usage: curl -fsSL https://raw.githubusercontent.com/mwaldstein/qipu/main/scripts/install.sh | bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
REPO="mwaldstein/qipu"
BINARY_NAME="qipu"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Detect platform and architecture
detect_platform() {
    local os="$(uname -s)"
    local arch="$(uname -m)"
    
    case "$os" in
        Linux*)
            OS="unknown-linux-gnu"
            ;;
        Darwin*)
            OS="apple-darwin"
            ;;
        *)
            echo -e "${RED}Error: Unsupported operating system: $os${NC}"
            exit 1
            ;;
    esac
    
    case "$arch" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        *)
            echo -e "${RED}Error: Unsupported architecture: $arch${NC}"
            exit 1
            ;;
    esac
    
    TARGET="${ARCH}-${OS}"
    echo -e "${GREEN}Detected platform: ${TARGET}${NC}"
}

# Get the latest release version
get_latest_version() {
    echo "Fetching latest release version..."
    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"v([^"]+)".*/\1/')
    
    if [ -z "$VERSION" ]; then
        echo -e "${RED}Error: Could not determine latest version${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}Latest version: v${VERSION}${NC}"
}

# Download and verify binary
download_binary() {
    local filename="${BINARY_NAME}-${VERSION}-${TARGET}.tar.gz"
    local url="https://github.com/${REPO}/releases/download/v${VERSION}/${filename}"
    local checksum_url="${url}.sha256"
    
    echo "Downloading ${filename}..."
    
    # Create temporary directory
    TMPDIR=$(mktemp -d)
    trap "rm -rf $TMPDIR" EXIT
    
    cd "$TMPDIR"
    
    # Download binary and checksum
    if ! curl -fsSL -o "$filename" "$url"; then
        echo -e "${RED}Error: Failed to download binary${NC}"
        exit 1
    fi
    
    if ! curl -fsSL -o "${filename}.sha256" "$checksum_url"; then
        echo -e "${YELLOW}Warning: Could not download checksum file${NC}"
    else
        echo "Verifying checksum..."
        # Extract just the hash from the checksum file
        expected_hash=$(cat "${filename}.sha256" | awk '{print $1}')
        actual_hash=$(shasum -a 256 "$filename" | awk '{print $1}')
        
        if [ "$expected_hash" != "$actual_hash" ]; then
            echo -e "${RED}Error: Checksum verification failed${NC}"
            echo "Expected: $expected_hash"
            echo "Actual:   $actual_hash"
            exit 1
        fi
        echo -e "${GREEN}Checksum verified${NC}"
    fi
    
    # Extract binary
    echo "Extracting binary..."
    tar xzf "$filename"
}

# Install binary
install_binary() {
    echo "Installing to ${INSTALL_DIR}..."
    
    # Create install directory if it doesn't exist
    mkdir -p "$INSTALL_DIR"
    
    # Move binary
    mv "$TMPDIR/${BINARY_NAME}" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/${BINARY_NAME}"
    
    echo -e "${GREEN}Successfully installed ${BINARY_NAME} to ${INSTALL_DIR}${NC}"
}

# Check if binary is in PATH
check_path() {
    if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
        echo ""
        echo -e "${YELLOW}Warning: ${INSTALL_DIR} is not in your PATH${NC}"
        echo "Add the following line to your shell configuration file:"
        echo ""
        echo "  export PATH=\"\$PATH:${INSTALL_DIR}\""
        echo ""
        
        # Detect shell and provide specific instructions
        if [ -n "$BASH_VERSION" ]; then
            echo "For bash, add it to ~/.bashrc or ~/.bash_profile"
        elif [ -n "$ZSH_VERSION" ]; then
            echo "For zsh, add it to ~/.zshrc"
        fi
        echo ""
    else
        echo ""
        echo -e "${GREEN}${BINARY_NAME} is ready to use!${NC}"
        echo "Run '${BINARY_NAME} --help' to get started."
    fi
}

# Verify installation
verify_installation() {
    if [ -x "$INSTALL_DIR/${BINARY_NAME}" ]; then
        local installed_version=$("$INSTALL_DIR/${BINARY_NAME}" --version 2>/dev/null | head -1 || echo "unknown")
        echo ""
        echo -e "${GREEN}Installation verified: ${installed_version}${NC}"
    else
        echo -e "${RED}Error: Installation verification failed${NC}"
        exit 1
    fi
}

# Main installation flow
main() {
    echo "Qipu Installer"
    echo "=============="
    echo ""
    
    detect_platform
    get_latest_version
    download_binary
    install_binary
    verify_installation
    check_path
}

main
