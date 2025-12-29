#!/bin/bash

# Overlook Build Script - macOS Binary Builder
# Builds native macOS binaries for both Apple Silicon (arm64) and Intel (x86_64)
#
# Usage:
#   ./scripts/build.sh              # Build both architectures
#   ./scripts/build.sh arm64        # Build ARM64 only
#   ./scripts/build.sh x86_64       # Build x86_64 only
#
# Note: Linux builds use Docker. See build-linux-docker.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Default: build both
ARCH="${1:-all}"

log_info() {
    echo -e "${BLUE}ℹ ${1}${NC}"
}

log_success() {
    echo -e "${GREEN}✓ ${1}${NC}"
}

log_error() {
    echo -e "${RED}✗ ${1}${NC}"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found. Please install Rust."
        exit 1
    fi
    
    if ! command -v rustc &> /dev/null; then
        log_error "Rustc not found. Please install Rust."
        exit 1
    fi
    
    log_success "Rust toolchain found"
}

# Install cross-compilation target
install_target() {
    local target=$1
    
    if ! rustup target list | grep -q "^${target} (installed)"; then
        log_info "Installing target: $target"
        rustup target add "$target"
    fi
}

# Build for a specific target
build_target() {
    local target=$1
    local arch=$2
    
    log_info "Building for macOS $arch..."
    
    if cargo build --release --target "$target" 2>&1 | grep -E "Finished|error:" | tail -1; then
        log_success "Built macOS $arch"
        return 0
    else
        log_error "Failed to build macOS $arch"
        return 1
    fi
}

# Show usage
show_usage() {
    cat << EOF
Overlook macOS Build Script

Usage: $0 [ARCH]

ARCH options:
    all     - Build both ARM64 and x86_64 (default)
    arm64   - Build ARM64 only (Apple Silicon)
    x86_64  - Build x86_64 only (Intel Mac)

Examples:
    $0                  # Build both architectures
    $0 arm64           # Build ARM64 only
    $0 x86_64          # Build x86_64 only

Notes:
    - Runs on macOS only
    - Produces binaries at: target/{aarch64,x86_64}-apple-darwin/release/overlook
    - For Linux builds, use: ./scripts/build-linux-docker.sh
    - Windows builds deferred until Linux verified

EOF
}

# Main build
main() {
    case "$ARCH" in
        -h|--help)
            show_usage
            exit 0
            ;;
        all)
            check_prerequisites
            log_info "Building macOS binaries for both architectures..."
            echo
            
            install_target "aarch64-apple-darwin"
            install_target "x86_64-apple-darwin"
            
            build_target "aarch64-apple-darwin" "ARM64" || true
            build_target "x86_64-apple-darwin" "x86_64" || true
            
            echo
            log_success "macOS build complete"
            log_info "Binaries located at:"
            log_info "  target/aarch64-apple-darwin/release/overlook (ARM64)"
            log_info "  target/x86_64-apple-darwin/release/overlook (x86_64)"
            ;;
        arm64)
            check_prerequisites
            install_target "aarch64-apple-darwin"
            build_target "aarch64-apple-darwin" "ARM64"
            ;;
        x86_64)
            check_prerequisites
            install_target "x86_64-apple-darwin"
            build_target "x86_64-apple-darwin" "x86_64"
            ;;
        *)
            log_error "Unknown architecture: $ARCH"
            show_usage
            exit 1
            ;;
    esac
}

main
