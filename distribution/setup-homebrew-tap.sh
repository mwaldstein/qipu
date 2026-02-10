#!/bin/bash

# Script to set up the Homebrew tap repository for qipu
# This script should be run from the qipu repository root

set -e

TAP_NAME="homebrew-qipu"
TAP_DIR="distribution/${TAP_NAME}"
GITHUB_USER="mwaldstein"

echo "Setting up Homebrew tap repository..."
echo ""

# Check if the tap directory exists
if [ ! -d "${TAP_DIR}" ]; then
    echo "Error: Tap directory not found: ${TAP_DIR}"
    exit 1
fi

echo "Tap files created in: ${TAP_DIR}"
echo ""
echo "Next steps to create the tap repository:"
echo ""
echo "1. Create a new GitHub repository: ${GITHUB_USER}/${TAP_NAME}"
echo ""
echo "2. Initialize the repository and push the tap files:"
echo "   cd ${TAP_DIR}"
echo "   git init"
echo "   git add ."
echo "   git commit -m 'Initial commit'"
echo "   git branch -M main"
echo "   git remote add origin git@github.com:${GITHUB_USER}/${TAP_NAME}.git"
echo "   git push -u origin main"
echo ""
echo "3. After the repository is created, users can install qipu via:"
echo "   brew tap ${GITHUB_USER}/qipu"
echo "   brew install qipu"
echo ""
echo "4. To update the Formula in the future:"
echo "   - Update the version in Formula/qipu.rb"
echo "   - Update the SHA256 hash (use: shasum -a 256 <tarball>)"
echo "   - Commit and push the changes to the tap repository"
echo ""
echo "The Formula uses pre-built binaries for Intel and Apple Silicon Macs."
echo "No build dependencies required - binaries are extracted directly."
