#!/bin/bash

# Super IDE Startup Script
echo "🚀 Starting Super IDE - AI-Powered Development Environment"

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Please install Rust first:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Build the project
echo "🔧 Building Super IDE..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo "🌐 Starting Super IDE on http://localhost:3000"
    echo ""
    echo "Features available:"
    echo "  • 🧠 AI-powered code completion (OpenAI integration)"
    echo "  • 📝 Monaco Editor with syntax highlighting"
    echo "  • 🔍 Real-time code analysis"
    echo "  • 🎨 Modern web interface"
    echo ""
    echo "Press Ctrl+C to stop the server"
    echo ""
    
    # Run the application
    cargo run --bin super-ide
else
    echo "❌ Build failed. Please check the error messages above."
    exit 1
fi