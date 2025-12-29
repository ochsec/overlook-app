#!/bin/bash

# Overlook Release Packaging Script
# Packages built binaries into distribution formats
# Creates: .dmg (macOS), .exe (Windows), .tar.gz (Linux)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_ROOT/target/release"
RELEASE_DIR="$PROJECT_ROOT/releases"
TARGET_DIR="$PROJECT_ROOT/target"
VERSION="${1:-dev}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}ℹ ${1}${NC}"
}

log_success() {
    echo -e "${GREEN}✓ ${1}${NC}"
}

log_error() {
    echo -e "${RED}✗ ${1}${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠ ${1}${NC}"
}

# Initialize release directory
init_release_dir() {
    mkdir -p "$RELEASE_DIR"
    log_info "Release directory: $RELEASE_DIR"
}

# Package Linux binaries
package_linux() {
    log_info "Packaging Linux binaries..."
    
    local x86_64_binary="$TARGET_DIR/x86_64-unknown-linux-gnu/release/overlook"
    local arm64_binary="$TARGET_DIR/aarch64-unknown-linux-gnu/release/overlook"
    
    if [ -f "$x86_64_binary" ]; then
        log_info "Creating overlook-linux-x86_64.tar.gz..."
        cd "$(dirname "$x86_64_binary")"
        tar -czf "$RELEASE_DIR/overlook-linux-x86_64.tar.gz" overlook
        log_success "Packaged: overlook-linux-x86_64.tar.gz"
        cd - > /dev/null
    else
        log_warning "x86_64 Linux binary not found"
        log_info "Build with: ./scripts/build-linux-docker.sh"
    fi
    
    if [ -f "$arm64_binary" ]; then
        log_info "Creating overlook-linux-arm64.tar.gz..."
        cd "$(dirname "$arm64_binary")"
        tar -czf "$RELEASE_DIR/overlook-linux-arm64.tar.gz" overlook
        log_success "Packaged: overlook-linux-arm64.tar.gz"
        cd - > /dev/null
    else
        log_warning "ARM64 Linux binary not found"
        log_info "Build with: ./scripts/build-linux-docker.sh"
    fi
}

# Package Windows binaries
package_windows() {
    log_info "Packaging Windows binaries..."
    
    local x86_64_binary="$BUILD_DIR/x86_64-pc-windows-gnu/release/overlook.exe"
    local arm64_binary="$BUILD_DIR/aarch64-pc-windows-gnu/release/overlook.exe"
    
    if [ -f "$x86_64_binary" ]; then
        cp "$x86_64_binary" "$RELEASE_DIR/overlook-windows-x86_64.exe"
        log_success "Packaged: overlook-windows-x86_64.exe"
    else
        log_warning "x86_64 Windows binary not found at $x86_64_binary"
    fi
    
    if [ -f "$arm64_binary" ]; then
        cp "$arm64_binary" "$RELEASE_DIR/overlook-windows-arm64.exe"
        log_success "Packaged: overlook-windows-arm64.exe"
    else
        log_warning "ARM64 Windows binary not found at $arm64_binary"
    fi
}

# Package macOS binaries
package_macos() {
    log_info "Packaging macOS binaries..."
    
    local project_root="$(dirname "$SCRIPT_DIR")"
    
    # Locations where binaries can be found
    local arm64_native="$project_root/target/release/overlook"
    local arm64_target="$project_root/target/aarch64-apple-darwin/release/overlook"
    local x86_64_target="$project_root/target/x86_64-apple-darwin/release/overlook"
    
    # ARM64 binary (native on Apple Silicon)
    if [ -f "$arm64_native" ] && [[ "$(uname -m)" == "arm64" ]]; then
        log_info "Found native ARM64 macOS binary"
        create_macos_bundle "$arm64_native" "arm64"
    elif [ -f "$arm64_target" ]; then
        log_info "Found ARM64 macOS binary"
        create_macos_bundle "$arm64_target" "arm64"
    else
        log_warning "ARM64 macOS binary not found"
    fi
    
    # x86_64 binary (cross-compiled on Apple Silicon, or native on Intel)
    if [ -f "$x86_64_target" ]; then
        log_info "Found x86_64 macOS binary"
        create_macos_bundle "$x86_64_target" "x86_64"
    else
        log_warning "x86_64 macOS binary not found"
    fi
}

# Create macOS application bundle
create_macos_bundle() {
    local binary=$1
    local arch=$2
    local app_name="Overlook"
    local bundle_name="overlook-macos-${arch}"
    local bundle_dir="$RELEASE_DIR/${bundle_name}.app"
    local contents_dir="$bundle_dir/Contents"
    local macos_dir="$contents_dir/MacOS"
    local resources_dir="$contents_dir/Resources"
    
    # Create bundle structure
    mkdir -p "$macos_dir"
    mkdir -p "$resources_dir"
    
    # Copy binary
    cp "$binary" "$macos_dir/overlook"
    chmod +x "$macos_dir/overlook"
    
    # Create Info.plist
    cat > "$contents_dir/Info.plist" << 'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>overlook</string>
    <key>CFBundleIdentifier</key>
    <string>com.replmade.overlook</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>Overlook</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright 2025 REPL Made LLC. All rights reserved.</string>
    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
</dict>
</plist>
PLIST
    
    # Create DMG (requires create-dmg tool)
    if command -v create-dmg &> /dev/null; then
        log_info "Creating DMG for $arch..."
        create-dmg \
            --volname "Overlook" \
            --window-pos 200 120 \
            --window-size 800 400 \
            --icon-size 100 \
            --icon "Overlook.app" 200 190 \
            --hide-extension "Overlook.app" \
            --app-drop-link 600 190 \
            "$RELEASE_DIR/overlook-macos-${arch}.dmg" \
            "$bundle_dir"
        log_success "Created: overlook-macos-${arch}.dmg"
    else
        log_warning "create-dmg not found. Install with: brew install create-dmg"
        log_info "Alternatively, compress the .app bundle for distribution"
        # Fallback: create compressed archive
        cd "$RELEASE_DIR"
        tar -czf "${bundle_name}.tar.gz" "${bundle_name}.app"
        log_success "Created fallback archive: ${bundle_name}.tar.gz"
        cd - > /dev/null
    fi
}

# Show summary
show_summary() {
    echo
    log_info "Release Summary"
    echo "============================================"
    
    if [ -d "$RELEASE_DIR" ]; then
        echo "Files in $RELEASE_DIR:"
        ls -lh "$RELEASE_DIR" | awk 'NR>1 {printf "  %-40s %10s\n", $9, $5}'
        echo
        log_success "All files ready for upload"
    else
        log_error "Release directory not found"
    fi
}

# Main execution
main() {
    if [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
        cat << EOF
Overlook Release Packaging Script

Usage: $0 [VERSION]

Arguments:
    VERSION - Version number (e.g., 1.0.0, optional, defaults to 'dev')

Examples:
    $0              # Package with version 'dev'
    $0 1.0.0       # Package with version '1.0.0'

Notes:
    - Requires binaries to already be built
    - Run ./scripts/build.sh first to build binaries
    - Linux binaries are tar.gz archives
    - Windows binaries are .exe files
    - macOS binaries are .app bundles (or .dmg if create-dmg is installed)

EOF
        exit 0
    fi
    
    init_release_dir
    package_linux
    package_windows
    package_macos
    show_summary
}

main "$@"
