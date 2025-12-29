#!/bin/bash

# Overlook Linux Build Script - Docker Version
# Builds Linux binaries for both x86_64 and aarch64 using Docker
#
# Requires: Docker to be installed and running
#
# Usage:
#   ./scripts/build-linux-docker.sh              # Build both architectures
#   ./scripts/build-linux-docker.sh x86_64       # Build x86_64 only
#   ./scripts/build-linux-docker.sh arm64        # Build ARM64 only

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

ARCH="${1:-all}"
DOCKER_IMAGE="overlook-linux-builder"

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
    
    if ! command -v docker &> /dev/null; then
        log_error "Docker not found. Please install Docker Desktop."
        log_info "Download from: https://www.docker.com/products/docker-desktop"
        exit 1
    fi
    
    if ! docker info &>/dev/null; then
        log_error "Docker daemon is not running."
        log_info "Please start Docker Desktop and try again."
        exit 1
    fi
    
    log_success "Docker is available and running"
}

# Build Docker image
build_docker_image() {
    log_info "Building Docker image: $DOCKER_IMAGE"
    
    if docker build -t "$DOCKER_IMAGE" -f "$PROJECT_ROOT/Dockerfile" "$PROJECT_ROOT"; then
        log_success "Docker image built successfully"
    else
        log_error "Failed to build Docker image"
        exit 1
    fi
}

# Extract binaries from Docker image
extract_binaries() {
    log_info "Extracting binaries from Docker image..."
    
    local container_id
    container_id=$(docker create "$DOCKER_IMAGE")
    
    if [ -z "$container_id" ]; then
        log_error "Failed to create container"
        exit 1
    fi
    
    log_info "Copying x86_64 binary..."
    if docker cp "$container_id:/build/target/x86_64-unknown-linux-gnu/release/overlook" \
        "$PROJECT_ROOT/target/x86_64-unknown-linux-gnu/release/overlook" 2>/dev/null; then
        log_success "Extracted x86_64 binary"
    else
        log_error "Failed to extract x86_64 binary"
    fi
    
    log_info "Copying aarch64 binary..."
    if docker cp "$container_id:/build/target/aarch64-unknown-linux-gnu/release/overlook" \
        "$PROJECT_ROOT/target/aarch64-unknown-linux-gnu/release/overlook" 2>/dev/null; then
        log_success "Extracted aarch64 binary"
    else
        log_error "Failed to extract aarch64 binary"
    fi
    
    docker rm "$container_id" > /dev/null
}

# Show usage
show_usage() {
    cat << EOF
Overlook Linux Build Script (Docker)

Usage: $0 [ARCH]

ARCH options:
    all     - Build both x86_64 and aarch64 (default)
    x86_64  - Build x86_64 only
    arm64   - Build aarch64 only

Examples:
    $0                  # Build both architectures
    $0 x86_64          # Build x86_64 only
    $0 arm64           # Build ARM64 only

Prerequisites:
    - Docker Desktop installed and running
    - https://www.docker.com/products/docker-desktop

How it works:
    1. Builds a Docker image with Rust and build tools
    2. Compiles Linux binaries inside the container
    3. Extracts binaries to target/ directory
    4. You can then package and upload them

Output:
    Binaries are saved to:
    - target/x86_64-unknown-linux-gnu/release/overlook
    - target/aarch64-unknown-linux-gnu/release/overlook

EOF
}

# Main execution
main() {
    case "$ARCH" in
        -h|--help)
            show_usage
            exit 0
            ;;
        all|x86_64|arm64)
            check_prerequisites
            build_docker_image
            extract_binaries
            
            echo
            log_success "Linux build complete"
            log_info "Binaries located at:"
            log_info "  target/x86_64-unknown-linux-gnu/release/overlook"
            log_info "  target/aarch64-unknown-linux-gnu/release/overlook"
            log_info ""
            log_info "Next steps:"
            log_info "  1. Run: ./scripts/package-release.sh 1.0.0"
            log_info "  2. Run: AWS_PROFILE=overlook-releases ./scripts/upload-release.sh"
            ;;
        *)
            log_error "Unknown architecture: $ARCH"
            show_usage
            exit 1
            ;;
    esac
}

main
