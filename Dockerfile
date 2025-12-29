# Overlook Linux Build Container
# Builds Linux binaries for both x86_64 and aarch64 architectures

FROM rust:latest

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    gcc-aarch64-linux-gnu \
    libc6-dev-arm64-cross \
    && rm -rf /var/lib/apt/lists/*

# Set up Rust targets
RUN rustup target add x86_64-unknown-linux-gnu
RUN rustup target add aarch64-unknown-linux-gnu

# Create working directory
WORKDIR /build

# Copy source code
COPY . .

# Build x86_64 binary
RUN cargo build --release --target x86_64-unknown-linux-gnu

# Build aarch64 binary
RUN cargo build --release --target aarch64-unknown-linux-gnu

# Output directory - binaries will be available after build
# Located at: target/x86_64-unknown-linux-gnu/release/overlook
#             target/aarch64-unknown-linux-gnu/release/overlook
