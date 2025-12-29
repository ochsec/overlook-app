# Overlook Release Process

Complete guide for building, packaging, and releasing Overlook binaries to users via Linode Object Storage.

## Quick Start

For a complete release (macOS + Linux):

```bash
# One-time setup
brew install awscli
cat >> ~/.aws/credentials << 'EOF'

[overlook-releases]
aws_access_key_id = J3KTSADQSFK4YSQASRCP
aws_secret_access_key = GjmJoZvBUHkFgGlQpQJmkoek5My7723ovrVSkoVM
EOF
chmod 600 ~/.aws/credentials

# For each release
./scripts/build.sh                           # Build macOS (arm64 + x86_64)
./scripts/build-linux-docker.sh              # Build Linux (x86_64 + arm64) in Docker
./scripts/package-release.sh 1.0.0           # Package all binaries
AWS_PROFILE=overlook-releases ./scripts/upload-release.sh  # Upload to Linode
```

This creates packages for macOS and Linux. Windows is deferred for now.

For detailed instructions, see below.

## Overview

The release process has three steps:

1. **Build** - Compile binaries for all platforms using `./scripts/build.sh`
2. **Package** - Bundle binaries into distribution formats using `./scripts/package-release.sh`
3. **Upload** - Push files to Linode Object Storage using `./scripts/upload-release.sh`

All files are served from: `https://overlook-downloads.us-ord-1.linodeobjects.com/`

Distribution includes 6 binaries:
- `overlook-macos-arm64.dmg` - macOS Apple Silicon
- `overlook-macos-x86_64.dmg` - macOS Intel
- `overlook-windows-x86_64.exe` - Windows 64-bit
- `overlook-windows-arm64.exe` - Windows ARM
- `overlook-linux-x86_64.tar.gz` - Linux 64-bit
- `overlook-linux-arm64.tar.gz` - Linux ARM

## Prerequisites

### Required Tools

- **Rust/Cargo** - `rustc --version` should show 1.70+
- **Git** - For version control
- **AWS CLI** - For uploading to Linode Object Storage

Install AWS CLI:
```bash
# macOS
brew install awscli

# Linux (Ubuntu/Debian)
sudo apt-get install awscli

# Or via pip (all platforms)
pip install awscli
```

### Credentials

You need to store your Linode Object Storage credentials. Choose ONE of these approaches:

#### Option 1: Named AWS Profile (Recommended)

Create an isolated profile so your Overlook credentials don't affect other AWS projects:

```bash
cat >> ~/.aws/credentials << 'EOF'

[overlook-releases]
aws_access_key_id = J3KTSADQSFK4YSQASRCP
aws_secret_access_key = GjmJoZvBUHkFgGlQpQJmkoek5My7723ovrVSkoVM
EOF

chmod 600 ~/.aws/credentials
```

Then use when uploading:
```bash
AWS_PROFILE=overlook-releases ./scripts/upload-release.sh
```

**Pros:** Isolated from other projects, credentials persisted, easy to use  
**Cons:** Credentials stored on disk (but with restricted permissions)

#### Option 2: Environment Variables

Set credentials temporarily for a single session:

```bash
export AWS_ACCESS_KEY_ID="J3KTSADQSFK4YSQASRCP"
export AWS_SECRET_ACCESS_KEY="GjmJoZvBUHkFgGlQpQJmkoek5My7723ovrVSkoVM"
./scripts/upload-release.sh
```

**Pros:** No credentials stored on disk, session-specific, good for CI/CD  
**Cons:** Must set each session, visible in shell history if not careful

To avoid shell history:
```bash
set +o history
export AWS_ACCESS_KEY_ID="J3KTSADQSFK4YSQASRCP"
export AWS_SECRET_ACCESS_KEY="GjmJoZvBUHkFgGlQpQJmkoek5My7723ovrVSkoVM"
./scripts/upload-release.sh
set -o history
```

#### Option 3: Interactive Prompt

No credentials to store - the script will ask for them:

```bash
./scripts/upload-release.sh
# Script prompts: Enter Linode Access Key: [you type]
#               Enter Linode Secret Key: [you type - hidden]
```

**Pros:** Zero stored credentials, fresh input each time  
**Cons:** Must type credentials each time, not suitable for automation

### Platform-Specific Requirements

**macOS Builds:**
- Must run on macOS to create native binaries
- Optional: `brew install create-dmg` for better DMG creation

**Windows Builds:**
- Best built on Windows with MSVC toolchain
- Cross-compilation from macOS/Linux is not reliable
- If cross-compiling on Linux, requires MinGW properly configured

**Linux Builds:**
- Best built natively on Linux
- Cross-compilation from macOS requires musl-tools setup (not recommended)

## Step-by-Step Release Guide

### 1. Prepare Release

Update version in `Cargo.toml`:
```toml
[package]
name = "overlook"
version = "1.0.0"  # Update this
```

Commit your changes:
```bash
git add Cargo.toml
git commit -m "Bump version to 1.0.0"
```

### 2. Build Binaries

#### Step 2A: Build macOS Binaries

Build both Apple Silicon (arm64) and Intel (x86_64) binaries:

```bash
./scripts/build.sh all
```

Or build a specific architecture:
```bash
./scripts/build.sh arm64       # Apple Silicon only
./scripts/build.sh x86_64      # Intel only
```

Expected output:
```
ℹ Building macOS binaries for both architectures...
✓ Built macOS ARM64
✓ Built macOS x86_64

✓ macOS build complete
Binaries located at:
  target/aarch64-apple-darwin/release/overlook (ARM64)
  target/x86_64-apple-darwin/release/overlook (x86_64)
```

Build time: 2-5 minutes per architecture.

#### Step 2B: Build Linux Binaries (Using Docker)

Build Linux binaries for both x86_64 and ARM64 architectures in Docker:

```bash
./scripts/build-linux-docker.sh all
```

Or build a specific architecture:
```bash
./scripts/build-linux-docker.sh x86_64      # x86_64 only
./scripts/build-linux-docker.sh arm64       # ARM64 only
```

**Requirements:** Docker Desktop must be running

Expected output:
```
ℹ Checking prerequisites...
✓ Docker is available and running

ℹ Building Docker image: overlook-linux-builder
✓ Docker image built successfully

ℹ Extracting binaries from Docker image...
✓ Extracted x86_64 binary
✓ Extracted aarch64 binary

✓ Linux build complete
Binaries located at:
  target/x86_64-unknown-linux-gnu/release/overlook
  target/aarch64-unknown-linux-gnu/release/overlook
```

Build time: 5-15 minutes (first build slower due to Docker image creation).

**Note:** Windows binaries are deferred. They will be built via CI/CD in the future.

### 3. Package Binaries

Create distribution packages:
```bash
./scripts/package-release.sh 1.0.0
```

The version argument is optional (defaults to "dev").

Expected output (on macOS):
```
ℹ Packaging Linux binaries...
✓ Packaged: overlook-linux-x86_64.tar.gz
✓ Packaged: overlook-linux-arm64.tar.gz

ℹ Packaging Windows binaries...
⚠ Windows binaries not found (built on Windows machine)

ℹ Packaging macOS binaries...
✓ Created: overlook-macos-arm64.dmg
✓ Created: overlook-macos-x86_64.dmg

ℹ Release Summary
Files in releases/:
  overlook-linux-x86_64.tar.gz         450MB
  overlook-linux-arm64.tar.gz          420MB
  overlook-macos-arm64.dmg             520MB
  overlook-macos-x86_64.dmg            510MB

✓ Binaries ready for upload
```

Files are packaged to `releases/` directory. Windows binaries must be built separately on a Windows machine.

### 4. Upload to Linode

Upload all packaged files:

**If using named profile (Option 1):**
```bash
AWS_PROFILE=overlook-releases ./scripts/upload-release.sh
```

**If using environment variables (Option 2):**
```bash
./scripts/upload-release.sh
```

**If using interactive prompt (Option 3):**
```bash
./scripts/upload-release.sh
```

The script will:
1. Display files to upload
2. Ask for confirmation
3. Upload files to Linode Object Storage
4. Verify uploads
5. Show download URLs

Example output:
```
Files to upload:
  overlook-linux-x86_64.tar.gz         450MB
  overlook-linux-arm64.tar.gz          420MB
  overlook-windows-x86_64.exe          480MB
  overlook-windows-arm64.exe           400MB
  overlook-macos-arm64.dmg             520MB
  overlook-macos-x86_64.dmg            510MB

Continue with upload? (y/n) y

✓ Uploaded overlook-linux-x86_64.tar.gz
✓ Uploaded overlook-linux-arm64.tar.gz
✓ Uploaded overlook-windows-x86_64.exe
✓ Uploaded overlook-windows-arm64.exe
✓ Uploaded overlook-macos-arm64.dmg
✓ Uploaded overlook-macos-x86_64.dmg

ℹ Download URLs
============================================
https://overlook-downloads.us-ord-1.linodeobjects.com/overlook-linux-x86_64.tar.gz
https://overlook-downloads.us-ord-1.linodeobjects.com/overlook-linux-arm64.tar.gz
https://overlook-downloads.us-ord-1.linodeobjects.com/overlook-windows-x86_64.exe
https://overlook-downloads.us-ord-1.linodeobjects.com/overlook-windows-arm64.exe
https://overlook-downloads.us-ord-1.linodeobjects.com/overlook-macos-arm64.dmg
https://overlook-downloads.us-ord-1.linodeobjects.com/overlook-macos-x86_64.dmg
```

Files are now live and available for download!

### 5. Verify Website

Test the download links on your website:
1. Go to https://yoursite.com/downloads.html
2. Click each download button
3. Verify files download correctly

The website uses static filenames, so old versions are automatically replaced.

## Complete Release in One Command

Combine all build, package, and upload steps:

```bash
./scripts/build.sh all && \
./scripts/build-linux-docker.sh all && \
./scripts/package-release.sh 1.0.0 && \
AWS_PROFILE=overlook-releases ./scripts/upload-release.sh
```

Or with line breaks for clarity:
```bash
./scripts/build.sh all
./scripts/build-linux-docker.sh all
./scripts/package-release.sh 1.0.0
AWS_PROFILE=overlook-releases ./scripts/upload-release.sh
```

This builds macOS binaries, Linux binaries in Docker, packages everything, and uploads to Linode.

## Troubleshooting

### Build Fails - "error: linker 'cc' not found"

**Solution:** Install build tools
```bash
# macOS
xcode-select --install

# Linux (Ubuntu/Debian)
sudo apt-get install build-essential

# Linux (RedHat/Fedora)
sudo dnf install gcc
```

### Windows Cross-Compilation Fails - "mingw32-gcc: command not found"

**Solution:** Install MinGW
```bash
# macOS
brew install mingw-w64

# Linux (Ubuntu/Debian)
sudo apt-get install mingw-w64
```

### Upload Fails - "Unable to locate credentials"

**Solution:** Provide credentials using one of the methods above:

1. **Named AWS Profile:**
   ```bash
   cat >> ~/.aws/credentials << 'EOF'
   
   [overlook-releases]
   aws_access_key_id = J3KTSADQSFK4YSQASRCP
   aws_secret_access_key = GjmJoZvBUHkFgGlQpQJmkoek5My7723ovrVSkoVM
   EOF
   chmod 600 ~/.aws/credentials
   AWS_PROFILE=overlook-releases ./scripts/upload-release.sh
   ```

2. **Environment Variables:**
   ```bash
   export AWS_ACCESS_KEY_ID="J3KTSADQSFK4YSQASRCP"
   export AWS_SECRET_ACCESS_KEY="GjmJoZvBUHkFgGlQpQJmkoek5My7723ovrVSkoVM"
   ./scripts/upload-release.sh
   ```

3. **Interactive Prompt:**
   ```bash
   ./scripts/upload-release.sh
   # Type credentials when prompted
   ```

### Upload Fails - "Connection refused"

**Problem:** Cannot connect to Linode

**Solution:** Verify credentials and bucket name:
```bash
# Test connection
aws s3 ls s3://overlook-downloads/ \
  --endpoint-url https://us-ord-1.linodeobjects.com \
  --region us-ord-1 \
  --profile overlook-releases
```

### macOS DMG Creation Fails - "create-dmg: command not found"

**Solution:** Install create-dmg
```bash
brew install create-dmg
```

Or use fallback (creates .tar.gz instead):
```bash
./scripts/package-release.sh
# Creates overlook-macos-arm64.tar.gz and overlook-macos-x86_64.tar.gz
```

## Manual File Management

### List Current Files on Linode

```bash
aws s3 ls s3://overlook-downloads/ \
  --endpoint-url https://us-ord-1.linodeobjects.com \
  --region us-ord-1 \
  --profile overlook-releases
```

### Delete Old Version

```bash
aws s3 rm s3://overlook-downloads/overlook-linux-x86_64.tar.gz \
  --endpoint-url https://us-ord-1.linodeobjects.com \
  --region us-ord-1 \
  --profile overlook-releases
```

### Download File from Linode

```bash
aws s3 cp s3://overlook-downloads/overlook-linux-x86_64.tar.gz . \
  --endpoint-url https://us-ord-1.linodeobjects.com \
  --region us-ord-1 \
  --profile overlook-releases
```

## File Naming Convention

All files follow a consistent naming pattern:

```
overlook-[platform]-[arch].[extension]
```

Where:
- **platform**: `macos`, `windows`, `linux`
- **arch**: `x86_64`, `arm64`
- **extension**: `.dmg`, `.exe`, `.tar.gz`

Examples:
- `overlook-macos-arm64.dmg` - macOS for Apple Silicon (M1+)
- `overlook-macos-x86_64.dmg` - macOS for Intel
- `overlook-windows-x86_64.exe` - Windows 64-bit
- `overlook-windows-arm64.exe` - Windows ARM (Surface Pro X)
- `overlook-linux-x86_64.tar.gz` - Linux 64-bit
- `overlook-linux-arm64.tar.gz` - Linux ARM (Raspberry Pi)

## Current Build Strategy

### What You Can Build Now

- ✓ macOS ARM64 & x86_64 - Built locally on your Mac
- ✓ Linux x86_64 & ARM64 - Built in Docker on your Mac
- ✗ Windows x86_64 & ARM64 - Deferred for future CI/CD implementation

### Your Release Workflow

For each release:

```bash
# Step 1: Build macOS
./scripts/build.sh all

# Step 2: Build Linux in Docker
./scripts/build-linux-docker.sh all

# Step 3: Package all binaries
./scripts/package-release.sh 1.0.0

# Step 4: Upload to Linode
AWS_PROFILE=overlook-releases ./scripts/upload-release.sh
```

This produces:
- `overlook-macos-arm64.tar.gz`
- `overlook-macos-x86_64.tar.gz`
- `overlook-linux-x86_64.tar.gz`
- `overlook-linux-arm64.tar.gz`

### Future: Windows Binaries

Windows builds are planned for a future phase and will use:
- GitLab CI/CD runners for automated Windows builds
- Or a self-hosted Windows machine
- Full details to be added in the future

## Version Management

The download page (`site/downloads.html`) uses static filenames. This means:

1. **Only latest version available** - Files are overwritten on each release
2. **No version history** - Old versions are automatically replaced
3. **Simple URLs** - Clean download links that don't change

If you need version history in the future, modify the upload scripts to use versioned paths like:
```
overlook-v1.0.0/overlook-macos-arm64.dmg
overlook-v1.1.0/overlook-macos-arm64.dmg
```

## Release Checklist

Before releasing, verify:

- [ ] Version updated in `Cargo.toml`
- [ ] Changes committed to Git
- [ ] All platforms built successfully
- [ ] All binaries packaged correctly
- [ ] All files uploaded to Linode
- [ ] Files verified on Linode
- [ ] Website download links work
- [ ] Test downloaded binaries work

## Installation Instructions (for your website)

For reference, users see these installation instructions:

**macOS:**
1. Download the .dmg file for your architecture
2. Open the downloaded .dmg file
3. Drag "Overlook" into the Applications folder
4. Open Applications and double-click Overlook to launch

**Windows:**
1. Download the .exe file
2. Run the installer and follow the prompts
3. Overlook will be added to your Start menu
4. Launch from the Start menu or create a desktop shortcut

**Linux:**
1. Download the .tar.gz file for your architecture
2. Extract: `tar -xzf overlook-linux-*.tar.gz`
3. Navigate to the extracted directory: `cd overlook`
4. Run: `./overlook`
5. Optionally, move to PATH: `sudo mv overlook /usr/local/bin/`

## Linode Configuration

Your bucket is already configured and ready:

- **Bucket Name**: `overlook-downloads`
- **Region**: `us-ord-1` (Chicago)
- **Endpoint**: `https://overlook-downloads.us-ord-1.linodeobjects.com`
- **Access**: Public read (downloadable without authentication)
- **Cost**: ~$6-7/month

## Future Enhancements

This manual process can be extended in the future:

**GitLab CI/CD Automation:**
- Auto-build and upload on version tags
- Zero manual steps for releases

**Auto-Update Feature:**
- Add version checking to the app
- Enable automatic updates from Linode

**Version History:**
- Store multiple versions in Linode
- Migrate to versioned paths

**Download Analytics:**
- Track downloads per platform
- Monitor user trends

## Script Help

For detailed information about any script:

```bash
./scripts/build.sh --help
./scripts/package-release.sh --help
./scripts/upload-release.sh --help
```
