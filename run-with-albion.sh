#!/bin/bash
# Build albion_core plugin, copy to plugins/, and optionally run with Postgres
set -e

cd "$(dirname "$0")"

# Kill any existing Pumpkin cargo/run processes to avoid file lock conflicts
echo "Checking for existing Pumpkin build processes..."
pkill -f "cargo run.*Pumpkin" 2>/dev/null || true
pkill -f "cargo build -p albion_core" 2>/dev/null || true
sleep 2

echo "Building albion_core plugin (release)..."
cargo build --release -p albion_core
cp target/release/libalbion_core.so plugins/

# Ensure plugin-data exists and copy default config if missing
mkdir -p plugin-data/albion_core
[ ! -f plugin-data/albion_core/config.toml ] && [ -f plugin-src/albinomccore/albion-core/config.toml ] && \
  cp plugin-src/albinomccore/albion-core/config.toml plugin-data/albion_core/config.toml && \
  echo "Copied albion_core config to plugin-data/"

echo "Plugin built and copied to plugins/"

if [ "$1" = "--run" ]; then
    echo "Starting Postgres (if Docker available)..."
    docker compose -f docker-compose.albion.yml up -d postgres 2>/dev/null || true
    sleep 2

    echo "Starting Pumpkin server (release)..."
    cargo run --release
fi
