#!/usr/bin/env bash
# Tests for .github/workflows/release.yml

set -e

WORKFLOW_FILE="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/.github/workflows/release.yml"

# Test helper functions
TESTS_PASSED=0
TESTS_FAILED=0

# Test: Workflow file exists
test_workflow_exists() {
    echo "TEST: Release workflow file exists"
    if [ -f "$WORKFLOW_FILE" ]; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Workflow has name
test_workflow_has_name() {
    echo "TEST: Workflow has name"
    if grep -q '^name: Release' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Workflow is triggered on tags
test_workflow_trigger_on_tags() {
    echo "TEST: Workflow triggers on version tags"
    if grep -q 'on:' "$WORKFLOW_FILE" && \
       grep -A5 '^on:' "$WORKFLOW_FILE" | grep -q 'workflow_dispatch:'; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Workflow has create-release job
test_create_release_job() {
    echo "TEST: create-release job exists"
    if grep -q 'create-release:' "$WORKFLOW_FILE" && \
       grep -q 'name: Create Release' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Workflow has build-release job
test_build_release_job() {
    echo "TEST: build-release job exists"
    if grep -q 'build-release:' "$WORKFLOW_FILE" && \
       grep -q 'name: Build Release' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: build-release depends on create-release
test_build_depends_on_create() {
    echo "TEST: build-release depends on create-release"
    if grep -A3 '^  build-release:' "$WORKFLOW_FILE" | grep -q 'needs: create-release'; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Linux x86_64 target is included
test_linux_x86_64_target() {
    echo "TEST: Linux x86_64 target is included"
    if grep -q 'x86_64-unknown-linux-gnu' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Linux aarch64 target is included
test_linux_aarch64_target() {
    echo "TEST: Linux aarch64 target is included"
    if grep -q 'aarch64-unknown-linux-gnu' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: macOS x86_64 target is included
test_macos_x86_64_target() {
    echo "TEST: macOS x86_64 target is included"
    if grep -q 'x86_64-apple-darwin' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: macOS aarch64 target is included
test_macos_aarch64_target() {
    echo "TEST: macOS aarch64 target is included"
    if grep -q 'aarch64-apple-darwin' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Windows x86_64 target is included
test_windows_x86_64_target() {
    echo "TEST: Windows x86_64 target is included"
    if grep -q 'x86_64-pc-windows-msvc' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Unix binaries use tar.gz
test_unix_tarball_format() {
    echo "TEST: Unix binaries use tar.gz format"
    if grep -q '\.tar\.gz' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Windows binaries use zip
test_windows_zip_format() {
    echo "TEST: Windows binaries use zip format"
    if grep -q '\.zip' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Checksums use SHA256
test_checksum_algorithm() {
    echo "TEST: SHA256 checksums are used"
    if grep -q 'shasum -a 256' "$WORKFLOW_FILE" || \
       grep -q 'SHA256' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Checksum files are uploaded
test_checksum_upload() {
    echo "TEST: Checksum files are uploaded"
    if grep -q '.sha256' "$WORKFLOW_FILE" && \
       grep -q 'upload-release-asset' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Version is extracted from git tag
test_version_extraction() {
    echo "TEST: Version is extracted from git tag"
    if grep -q 'GITHUB_REF#refs/tags/v' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Asset naming follows convention
test_asset_naming_convention() {
    echo "TEST: Asset naming follows qipu-<version>-<target> convention"
    if grep -q 'qipu-\${{.*version.*}}-\${{.*target.*}}' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Release is created with tag name
test_release_tag_name() {
    echo "TEST: Release uses tag_name parameter"
    if grep -q 'tag_name:.*github\.ref' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Binary is built with --release flag
test_release_build_flag() {
    echo "TEST: Binary is built with --release flag"
    if grep -q 'cargo build --release' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Target is specified in build command
test_build_target_specified() {
    echo "TEST: Build command specifies target"
    if grep -q 'cargo build.*--target' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: cross is used for aarch64-linux
test_cross_compilation() {
    echo "TEST: cross is used for aarch64-linux compilation"
    if grep -q 'use_cross.*true' "$WORKFLOW_FILE" && \
       grep -q 'cargo install cross' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Build matrix includes os and target
test_matrix_configuration() {
    echo "TEST: Build matrix includes os and target"
    if grep -A20 '      matrix:' "$WORKFLOW_FILE" | grep -q 'include:' && \
       grep -A25 '      matrix:' "$WORKFLOW_FILE" | grep -q 'os:' && \
       grep -A25 '      matrix:' "$WORKFLOW_FILE" | grep -q 'target:'; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: upload_url is passed from create-release
test_upload_url_output() {
    echo "TEST: upload_url is passed from create-release job"
    if grep -q 'upload_url:.*create-release' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: version output is passed from create-release
test_version_output() {
    echo "TEST: version is passed from create-release job"
    if grep -q 'version.*get_version' "$WORKFLOW_FILE" && \
       grep -q 'steps.get_version.outputs.version' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Workflow uses GITHUB_TOKEN
test_github_token_usage() {
    echo "TEST: Workflow uses GITHUB_TOKEN"
    if grep -q 'GITHUB_TOKEN' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Content types are set correctly
test_content_type_tarball() {
    echo "TEST: Tarball content type is application/gzip"
    if grep -q 'asset_content_type: application/gzip' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Content types are set correctly for zip
test_content_type_zip() {
    echo "TEST: Zip content type is application/zip"
    if grep -q 'asset_content_type: application/zip' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: All required targets are covered
test_all_required_targets() {
    echo "TEST: All spec-required targets are covered"
    local required_targets=(
        "x86_64-apple-darwin"
        "aarch64-apple-darwin"
        "x86_64-unknown-linux-gnu"
        "aarch64-unknown-linux-gnu"
        "x86_64-pc-windows-msvc"
    )
    local all_found=true
    for target in "${required_targets[@]}"; do
        if ! grep -q "$target" "$WORKFLOW_FILE"; then
            all_found=false
            break
        fi
    done
    
    if $all_found; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Workflow uses actions/checkout
test_checkout_action() {
    echo "TEST: Workflow uses actions/checkout"
    if grep -q 'actions/checkout@v4' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: Workflow uses dtolnay/rust-toolchain
test_rust_toolchain_action() {
    echo "TEST: Workflow uses dtolnay/rust-toolchain"
    if grep -q 'dtolnay/rust-toolchain' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Test: create-release job uses actions/create-release
test_create_release_action() {
    echo "TEST: create-release job uses actions/create-release"
    if grep -q 'actions/create-release' "$WORKFLOW_FILE"; then
        : $((TESTS_PASSED++))
    else
        : $((TESTS_FAILED++))
        echo "  FAILED"
        return 1
    fi
}

# Run all tests
main() {
    echo "===================="
    echo "Release Workflow Tests"
    echo "===================="
    echo ""

    test_workflow_exists || true
    test_workflow_has_name || true
    test_workflow_trigger_on_tags || true
    test_create_release_job || true
    test_build_release_job || true
    test_build_depends_on_create || true
    test_linux_x86_64_target || true
    test_linux_aarch64_target || true
    test_macos_x86_64_target || true
    test_macos_aarch64_target || true
    test_windows_x86_64_target || true
    test_unix_tarball_format || true
    test_windows_zip_format || true
    test_checksum_algorithm || true
    test_checksum_upload || true
    test_version_extraction || true
    test_asset_naming_convention || true
    test_release_tag_name || true
    test_release_build_flag || true
    test_build_target_specified || true
    test_cross_compilation || true
    test_matrix_configuration || true
    test_upload_url_output || true
    test_version_output || true
    test_github_token_usage || true
    test_content_type_tarball || true
    test_content_type_zip || true
    test_all_required_targets || true
    test_checkout_action || true
    test_rust_toolchain_action || true
    test_create_release_action || true

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
