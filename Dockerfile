FROM rust:1.81-slim-bookworm AS builder

# Install build tools
RUN apt-get update && apt-get install -y curl unzip libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*

# Download LibTorch (CPU)
WORKDIR /opt
RUN curl -L https://download.pytorch.org/libtorch/cpu/libtorch-cxx11-abi-shared-with-deps-2.4.0%2Bcpu.zip -o libtorch.zip \
    && unzip libtorch.zip && rm libtorch.zip

# Set environment variables for build
ENV LIBTORCH=/opt/libtorch
ENV LD_LIBRARY_PATH=${LIBTORCH}/lib:$LD_LIBRARY_PATH

# Copy your Rust project
WORKDIR /app
COPY . .

# Build your project
RUN cargo build --release

# ---- Runtime stage ----
FROM debian:bookworm-slim

# Copy LibTorch
COPY --from=builder /opt/libtorch /opt/libtorch
ENV LIBTORCH=/opt/libtorch
ENV LD_LIBRARY_PATH=${LIBTORCH}/lib

# Copy binary
COPY --from=builder /app/target/release/your_binary /usr/local/bin/your_binary

CMD ["your_binary"]
