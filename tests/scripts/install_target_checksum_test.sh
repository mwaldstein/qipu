#!/usr/bin/env bash
# Focused behavior tests for scripts/install.sh target overrides and checksums.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../scripts" && pwd)"
INSTALL_SCRIPT="$SCRIPT_DIR/install.sh"

TESTS_PASSED=0
TESTS_FAILED=0

pass() {
    : $((TESTS_PASSED++))
}

fail() {
    : $((TESTS_FAILED++))
    echo "  FAILED"
    return 1
}

test_target_env_var() {
    echo "TEST: QIPU_TARGET environment variable is respected"
    grep -q 'QIPU_TARGET' "$INSTALL_SCRIPT" && pass || fail
}

test_target_flag() {
    echo "TEST: --target override is parsed"
    (
        source "$INSTALL_SCRIPT"
        TARGET_OVERRIDE=""
        parse_args --target x86_64-unknown-linux-musl
        [ "$TARGET_OVERRIDE" = "x86_64-unknown-linux-musl" ]
    ) && pass || fail
}

test_checksum_verification_combined_file() {
    echo "TEST: Checksum verification works with combined SHA256SUMS"
    source "$INSTALL_SCRIPT"

    local test_dir=$(mktemp -d)
    trap "rm -rf $test_dir" RETURN
    cd "$test_dir"

    local test_file="qipu-1.2.3-x86_64-unknown-linux-musl.tar.gz"
    echo "test content" > "$test_file"

    local expected_hash
    expected_hash=$(shasum -a 256 "$test_file" | awk '{print $1}')
    {
        echo "0000000000000000000000000000000000000000000000000000000000000000  qipu-1.2.3-aarch64-apple-darwin.tar.gz"
        echo "$expected_hash  $test_file"
        echo "0000000000000000000000000000000000000000000000000000000000000000  qipu-1.2.3-x86_64-pc-windows-msvc.zip"
    } > SHA256SUMS

    verify_checksum_file "$test_file" && pass || fail
}

test_checksum_missing_entry() {
    echo "TEST: Missing checksum entry fails"
    source "$INSTALL_SCRIPT"

    local test_dir=$(mktemp -d)
    trap "rm -rf $test_dir" RETURN
    cd "$test_dir"

    local test_file="qipu-1.2.3-x86_64-unknown-linux-musl.tar.gz"
    echo "test content" > "$test_file"
    echo "0000000000000000000000000000000000000000000000000000000000000000  other.tar.gz" > SHA256SUMS

    ! verify_checksum_file "$test_file" >/dev/null 2>&1 && pass || fail
}

main() {
    echo "================================"
    echo "Install Target/Checksum Tests"
    echo "================================"
    echo ""

    test_target_env_var || true
    test_target_flag || true
    test_checksum_verification_combined_file || true
    test_checksum_missing_entry || true

    echo ""
    echo "===================="
    echo "Results"
    echo "===================="
    echo "Passed: $TESTS_PASSED"
    echo "Failed: $TESTS_FAILED"
    echo "Total:  $((TESTS_PASSED + TESTS_FAILED))"

    if [ $TESTS_FAILED -gt 0 ]; then
        exit 1
    fi
}

main
