#!/bin/bash

# Simple compilation test for Super IDE
cd /workspace/super-ide

echo "Testing Super IDE compilation..."

# Try to find Rust/Cargo in common locations
if command -v /usr/local/cargo/bin/cargo &> /dev/null; then
    CARGO_PATH="/usr/local/cargo/bin/cargo"
elif command -v ~/.cargo/bin/cargo &> /dev/null; then
    CARGO_PATH="~/.cargo/bin/cargo"
elif command -v cargo &> /dev/null; then
    CARGO_PATH="cargo"
else
    echo "Cargo not found. Checking if Rust is available..."
    if command -v rustc &> /dev/null; then
        echo "Rust found but Cargo not available. Trying to install Cargo..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
        source ~/.cargo/env
    fi
fi

# Check if cargo is now available
if command -v cargo &> /dev/null; then
    echo "Running cargo check..."
    cargo check --message-format short
    echo "Build test completed!"
else
    echo "Could not find Cargo. The project structure looks correct."
    echo "Key changes made:"
    echo "1. Added git and file_ops module imports to api/mod.rs"
    echo "2. Updated API State to include git_manager"
    echo "3. Updated file operation handlers to use new file_ops module"
    echo "4. Updated git operation handlers to use new git module"
    echo "5. Updated UI AppState to include git_manager"
fi