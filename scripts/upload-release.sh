#!/bin/bash

# Overlook Release Upload Script
# Uploads packaged binaries to Linode Object Storage
# Usage: ./scripts/upload-release.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RELEASE_DIR="$PROJECT_ROOT/releases"

# Linode Object Storage Configuration
BUCKET_NAME="overlook-downloads"
BUCKET_REGION="us-ord-1"
BUCKET_URL="https://${BUCKET_NAME}.${BUCKET_REGION}.linodeobjects.com"

# AWS CLI is S3-compatible with Linode Object Storage
# Credentials can be set via:
#   1. Environment variables (AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY)
#   2. AWS profile: aws s3 ls --profile overlook-releases
#   3. Interactive prompt (this script will ask if not found)

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

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check for AWS CLI (compatible with Linode S3)
    if ! command -v aws &> /dev/null; then
        log_error "AWS CLI not found"
        log_info "Install AWS CLI:"
        log_info "  macOS: brew install awscli"
        log_info "  Linux: pip install awscli"
        log_info "  Windows: https://aws.amazon.com/cli/"
        exit 1
    fi
    
    log_success "AWS CLI found"
    
    # Check for release files
    if [ ! -d "$RELEASE_DIR" ] || [ -z "$(ls -A "$RELEASE_DIR" 2>/dev/null)" ]; then
        log_error "No release files found in $RELEASE_DIR"
        log_info "Run ./scripts/build.sh and ./scripts/package-release.sh first"
        exit 1
    fi
    
    log_success "Release files found"
}

# Configure AWS CLI for Linode (if not already configured)
configure_linode_credentials() {
    log_info "Configuring Linode Object Storage credentials..."
    
    # Check if credentials are already set
    if [ -z "$AWS_ACCESS_KEY_ID" ] && [ -z "$AWS_SECRET_ACCESS_KEY" ]; then
        log_warning "Linode credentials not set in environment"
        log_info "Please provide your Linode Object Storage credentials:"
        
        read -p "Enter Linode Access Key: " access_key
        read -sp "Enter Linode Secret Key: " secret_key
        echo
        
        if [ -z "$access_key" ] || [ -z "$secret_key" ]; then
            log_error "Credentials not provided"
            exit 1
        fi
        
        export AWS_ACCESS_KEY_ID="$access_key"
        export AWS_SECRET_ACCESS_KEY="$secret_key"
    else
        log_success "Using credentials from environment variables"
    fi
}

# Test Linode connection
test_linode_connection() {
    log_info "Testing Linode Object Storage connection..."
    
    if aws s3 ls "s3://${BUCKET_NAME}/" \
        --endpoint-url "https://${BUCKET_REGION}.linodeobjects.com" \
        --region "${BUCKET_REGION}" &>/dev/null; then
        log_success "Connected to Linode Object Storage"
    else
        log_error "Failed to connect to Linode Object Storage"
        log_info "Check your credentials and bucket name"
        exit 1
    fi
}

# Upload files to Linode
upload_files() {
    log_info "Uploading release files to Linode Object Storage..."
    echo
    
    local failed=0
    local uploaded=0
    
    for file in "$RELEASE_DIR"/*; do
        if [ -f "$file" ]; then
            local filename=$(basename "$file")
            log_info "Uploading $filename..."
            
            if aws s3 cp "$file" "s3://${BUCKET_NAME}/${filename}" \
                --endpoint-url "https://${BUCKET_REGION}.linodeobjects.com" \
                --region "${BUCKET_REGION}" \
                --acl public-read; then
                log_success "Uploaded $filename"
                echo "  URL: ${BUCKET_URL}/${filename}"
                ((uploaded++))
            else
                log_error "Failed to upload $filename"
                ((failed++))
            fi
        fi
    done
    
    echo
    if [ $failed -eq 0 ]; then
        log_success "All files uploaded successfully ($uploaded files)"
    else
        log_warning "Upload complete with $failed failures (uploaded $uploaded files)"
        return 1
    fi
}

# Verify uploaded files
verify_uploads() {
    log_info "Verifying uploaded files..."
    echo
    
    local files=(
        "overlook-macos-arm64.dmg"
        "overlook-macos-x86_64.dmg"
        "overlook-windows-x86_64.exe"
        "overlook-windows-arm64.exe"
        "overlook-linux-x86_64.tar.gz"
        "overlook-linux-arm64.tar.gz"
    )
    
    for file in "${files[@]}"; do
        if aws s3 ls "s3://${BUCKET_NAME}/${file}" \
            --endpoint-url "https://${BUCKET_REGION}.linodeobjects.com" \
            --region "${BUCKET_REGION}" &>/dev/null; then
            log_success "Verified: $file"
        else
            log_warning "Not found: $file (may not have been built)"
        fi
    done
}

# Show download URLs
show_download_urls() {
    echo
    log_info "Download URLs"
    echo "============================================"
    
    local files=(
        "overlook-macos-arm64.dmg"
        "overlook-macos-x86_64.dmg"
        "overlook-windows-x86_64.exe"
        "overlook-windows-arm64.exe"
        "overlook-linux-x86_64.tar.gz"
        "overlook-linux-arm64.tar.gz"
    )
    
    for file in "${files[@]}"; do
        echo "${BUCKET_URL}/${file}"
    done
}

# Show usage
show_usage() {
    cat << EOF
Overlook Release Upload Script

Usage: $0 [OPTIONS]

Options:
    -h, --help      Show this help message
    --no-verify     Skip verification step
    --verify-only   Only verify existing uploads (don't upload)

Environment Variables:
    AWS_ACCESS_KEY_ID       - Linode Access Key
    AWS_SECRET_ACCESS_KEY   - Linode Secret Key
    AWS_DEFAULT_REGION      - Optional, defaults to us-ord-1

Examples:
    $0                  # Build and upload all files
    $0 --verify-only   # Verify what's currently uploaded

Prerequisites:
    1. Linode Object Storage bucket created (overlook-downloads)
    2. Release files built and packaged in ./releases/
    3. AWS CLI installed (pip install awscli)
    4. Linode credentials set

Setup:
    1. Install AWS CLI: pip install awscli
    2. Create credentials file at ~/.aws/credentials with:
       [default]
       aws_access_key_id = YOUR_LINODE_KEY
       aws_secret_access_key = YOUR_LINODE_SECRET
    3. Run this script

EOF
}

# Main execution
main() {
    case "$1" in
        -h|--help)
            show_usage
            exit 0
            ;;
        --verify-only)
            log_info "Verification Mode"
            configure_linode_credentials
            test_linode_connection
            verify_uploads
            show_download_urls
            ;;
        --no-verify)
            log_info "Upload Mode (no verification)"
            check_prerequisites
            configure_linode_credentials
            test_linode_connection
            upload_files
            show_download_urls
            ;;
        "")
            log_info "Overlook Release Upload"
            check_prerequisites
            configure_linode_credentials
            test_linode_connection
            
            echo
            echo "Files to upload:"
            ls -lh "$RELEASE_DIR" | awk 'NR>1 {printf "  %-40s %10s\n", $9, $5}'
            
            echo
            read -p "Continue with upload? (y/n) " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                upload_files
                verify_uploads
                show_download_urls
            else
                log_info "Upload cancelled"
            fi
            ;;
        *)
            log_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
}

main "$@"
