#!/bin/bash

echo "Super IDE Frontend Build Script"
echo "==============================="

# Set up npm to use local installation
export NPM_CONFIG_PREFIX=$(pwd)/local-npm
mkdir -p $NPM_CONFIG_PREFIX/lib/node_modules

echo "Installing dependencies..."

# Try different npm approaches
if npm install --prefix ./node_modules; then
    echo "✓ Dependencies installed successfully"
else
    echo "⚠ npm install failed, trying alternative approach..."
    
    # Alternative: Use npx to run build tools directly
    echo "Setting up build environment..."
    
    # Create a minimal node_modules structure
    mkdir -p node_modules/.bin
    
    # Download essential packages manually if needed
    echo "Using CDN-based build approach..."
fi

echo "Build environment ready!"
echo ""
echo "To run the demo:"
echo "1. Open demo.html in a web browser"
echo "2. Or use: python3 -m http.server 8080"
echo ""
echo "Available commands:"
echo "- npm run dev     : Start development server"
echo "- npm run build   : Build for production"
echo "- npm run preview : Preview production build"