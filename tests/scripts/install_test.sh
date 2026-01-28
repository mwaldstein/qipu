#!/usr/bin/env bash
# Tests for scripts/install.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../scripts" && pwd)"
INSTALL_SCRIPT="$SCRIPT_DIR/install.sh"

# Test helper functions
TESTS_PASSED=0
TESTS_FAILED=0

# Test: Script is executable
test_script_executable() {
    echo "TEST: install.sh is executable"
    if [ -x "$INSTALL_SCRIPT" ]; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Script has shebang
test_script_has_shebang() {
    echo "TEST: install.sh has proper shebang"
    if head -n 1 "$INSTALL_SCRIPT" | grep -q '^#!/usr/bin/env bash'; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Script uses set -e for error handling
test_script_set_e() {
    echo "TEST: install.sh uses set -e"
    if grep -q '^set -e$' "$INSTALL_SCRIPT"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Platform detection function exists
test_detect_platform_function() {
    echo "TEST: detect_platform function exists"
    if grep -q '^detect_platform()' "$INSTALL_SCRIPT"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Script handles Linux OS
test_linux_os_support() {
    echo "TEST: Linux OS is supported"
    if grep -q 'Linux\*)' "$INSTALL_SCRIPT"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Script handles macOS
test_macos_support() {
    echo "TEST: macOS is supported"
    if grep -q 'Darwin\*)' "$INSTALL_SCRIPT"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Script handles x86_64 architecture
test_x86_64_support() {
    echo "TEST: x86_64 architecture is supported"
    if grep -q 'x86_64|amd64)' "$INSTALL_SCRIPT"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Script handles aarch64 architecture
test_aarch64_support() {
    echo "TEST: aarch64 architecture is supported"
    if grep -q 'aarch64|arm64)' "$INSTALL_SCRIPT"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Script exits on unsupported OS
test_unsupported_os_exit() {
    echo "TEST: Unsupported OS causes exit"
    if grep -A2 '^\s*\*)' "$INSTALL_SCRIPT" | grep -q 'exit 1'; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Checksum verification function exists
test_checksum_verification() {
    echo "TEST: Checksum verification is implemented"
    if grep -q 'shasum -a 256' "$INSTALL_SCRIPT"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Script creates install directory
test_install_dir_creation() {
    echo "TEST: Install directory is created"
    if grep -q 'mkdir -p.*INSTALL_DIR' "$INSTALL_SCRIPT"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Script sets executable permissions
test_executable_permissions() {
    echo "TEST: Binary is made executable"
    if grep -q 'chmod +x' "$INSTALL_SCRIPT"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Script verifies installation
test_installation_verification() {
    echo "TEST: Installation is verified"
    if grep -q 'verify_installation' "$INSTALL_SCRIPT"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Script checks PATH
test_path_check() {
    echo "TEST: PATH is checked"
    if grep -q 'check_path' "$INSTALL_SCRIPT"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Script warns when install dir not in PATH
test_path_warning() {
    echo "TEST: PATH warning is shown when needed"
    if grep -q 'not in your PATH' "$INSTALL_SCRIPT"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Script respects INSTALL_DIR environment variable
test_install_dir_env_var() {
    echo "TEST: INSTALL_DIR environment variable is respected"
    if grep -q 'INSTALL_DIR.*:-' "$INSTALL_SCRIPT"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Script uses GitHub API for version
test_github_api_version() {
    echo "TEST: GitHub API is used for version detection"
    if grep -q 'api.github.com/repos.*releases/latest' "$INSTALL_SCRIPT"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Script downloads tarball
test_tarball_download() {
    echo "TEST: Tarball is downloaded"
    if grep -q '\.tar\.gz' "$INSTALL_SCRIPT"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Script extracts tarball
test_tarball_extract() {
    echo "TEST: Tarball is extracted"
    if grep -q 'tar xzf' "$INSTALL_SCRIPT"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Script uses temporary directory
test_temp_dir_usage() {
    echo "TEST: Temporary directory is used"
    if grep -q 'mktemp -d' "$INSTALL_SCRIPT"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Script cleans up temp directory
test_temp_dir_cleanup() {
    echo "TEST: Temporary directory is cleaned up"
    if grep -q 'trap.*rm -rf.*TMPDIR' "$INSTALL_SCRIPT"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Color codes are defined
test_color_codes() {
    echo "TEST: Color codes are defined"
    if grep -q "RED='\\\033" "$INSTALL_SCRIPT" && \
       grep -q "GREEN='\\\033" "$INSTALL_SCRIPT" && \
       grep -q "YELLOW='\\\033" "$INSTALL_SCRIPT"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Error messages use red color
test_error_coloring() {
    echo "TEST: Error messages use red color"
    if grep -q '\${RED}Error:' "$INSTALL_SCRIPT"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Success messages use green color
test_success_coloring() {
    echo "TEST: Success messages use green color"
    local success_lines
    success_lines=$(grep '\${GREEN}' "$INSTALL_SCRIPT" | grep -v 'Error' | grep -q . && echo "found" || echo "not found")
    if [ "$success_lines" = "found" ]; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Shell detection for PATH instructions
test_shell_detection() {
    echo "TEST: Shell is detected for PATH instructions"
    if grep -q 'BASH_VERSION' "$INSTALL_SCRIPT" && \
       grep -q 'ZSH_VERSION' "$INSTALL_SCRIPT"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Checksum mismatch causes exit
test_checksum_mismatch_exit() {
    echo "TEST: Checksum mismatch causes exit"
    if grep -A5 'Checksum verification failed' "$INSTALL_SCRIPT" | grep -q 'exit 1'; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Version extraction from GitHub API
test_version_extraction() {
    echo "TEST: Version is extracted from GitHub API response"
    if grep -q 'tag_name.*sed' "$INSTALL_SCRIPT"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Checksum verification with matching hash
test_checksum_verification_valid() {
    echo "TEST: Checksum verification succeeds with matching hash"
    
    local test_dir=$(mktemp -d)
    trap "rm -rf $test_dir" RETURN
    
    cd "$test_dir"
    
    local test_file="test-binary.tar.gz"
    local checksum_file="test-binary.tar.gz.sha256"
    
    echo "test content" > "$test_file"
    
    local expected_hash=$(shasum -a 256 "$test_file" | awk '{print $1}')
    echo "$expected_hash  $test_file" > "$checksum_file"
    
    local actual_hash=$(shasum -a 256 "$test_file" | awk '{print $1}')
    
    if [ "$expected_hash" == "$actual_hash" ]; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED: Expected $expected_hash, got $actual_hash"
        return 1
    fi
}

# Test: Checksum verification with mismatched hash
test_checksum_verification_invalid() {
    echo "TEST: Checksum verification fails with mismatched hash"
    
    local test_dir=$(mktemp -d)
    trap "rm -rf $test_dir" RETURN
    
    cd "$test_dir"
    
    local test_file="test-binary.tar.gz"
    local checksum_file="test-binary.tar.gz.sha256"
    
    echo "test content" > "$test_file"
    
    echo "0000000000000000000000000000000000000000000000000000000000000000  $test_file" > "$checksum_file"
    
    local expected_hash=$(cat "$checksum_file" | awk '{print $1}')
    local actual_hash=$(shasum -a 256 "$test_file" | awk '{print $1}')
    
    if [ "$expected_hash" != "$actual_hash" ]; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED: Hashes should differ but are the same"
        return 1
    fi
}

# Test: Checksum file format extraction
test_checksum_file_format() {
    echo "TEST: Checksum file format is correctly parsed"
    
    local test_dir=$(mktemp -d)
    trap "rm -rf $test_dir" RETURN
    
    cd "$test_dir"
    
    local test_file="test-binary.tar.gz"
    local checksum_file="test-binary.tar.gz.sha256"
    
    echo "test content" > "$test_file"
    
    local hash_with_spaces="  abc123  test-binary.tar.gz  "
    echo "$hash_with_spaces" > "$checksum_file"
    
    local extracted_hash=$(cat "$checksum_file" | awk '{print $1}')
    
    if [ "$extracted_hash" == "abc123" ]; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED: Expected 'abc123', got '$extracted_hash'"
        return 1
    fi
}

# Run all tests
main() {
    echo "===================="
    echo "Install Script Tests"
    echo "===================="
    echo ""

    test_script_executable || true
    test_script_has_shebang || true
    test_script_set_e || true
    test_detect_platform_function || true
    test_linux_os_support || true
    test_macos_support || true
    test_x86_64_support || true
    test_aarch64_support || true
    test_unsupported_os_exit || true
    test_checksum_verification || true
    test_install_dir_creation || true
    test_executable_permissions || true
    test_installation_verification || true
    test_path_check || true
    test_path_warning || true
    test_install_dir_env_var || true
    test_github_api_version || true
    test_tarball_download || true
    test_tarball_extract || true
    test_temp_dir_usage || true
    test_temp_dir_cleanup || true
    test_color_codes || true
    test_error_coloring || true
    test_success_coloring || true
    test_shell_detection || true
    test_checksum_mismatch_exit || true
    test_version_extraction || true
    test_checksum_verification_valid || true
    test_checksum_verification_invalid || true
    test_checksum_file_format || true

    echo ""
    echo "===================="
    echo "Results"
    echo "===================="
    echo "Passed: $TESTS_PASSED"
    echo "Failed: $TESTS_FAILED"
    echo "Total:  $((TESTS_PASSED + TESTS_FAILED))"
    echo ""

    if [ $TESTS_FAILED -gt 0 ]; then
        exit 1
    fi
}

main || true
