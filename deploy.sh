#!/bin/bash

# 🚀 Stellar Heads Deployment Script
# This script safely builds and deploys the entire application

echo "🌟 Starting Stellar Heads deployment..."
echo "======================================"

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to safely run commands with error checking
safe_run() {
    echo "🔧 Running: $*"
    if ! "$@"; then
        echo "❌ Command failed: $*"
        echo "🛑 Deployment stopped due to error"
        exit 1
    fi
    echo "✅ Command completed successfully"
    echo ""
}

# Check dependencies
echo "🔍 Checking dependencies..."
if ! command_exists trunk; then
    echo "❌ trunk not found. Install with: cargo install trunk"
    exit 1
fi

if ! command_exists wasm-pack; then
    echo "⚠️  wasm-pack not found but not required for trunk build"
fi

echo "✅ Dependencies OK"
echo ""

# Step 1: Clean previous builds
echo "🧹 Cleaning previous builds..."
safe_run rm -rf game/dist
safe_run rm -rf yew-frontend/dist
echo "✅ Clean completed"
echo ""

# Step 2: Build the game WASM first
echo "🎮 Building game WASM..."
cd game || exit 1
safe_run trunk build --release
echo "✅ Game WASM built successfully"
echo ""

# Step 3: Build the frontend
echo "🖥️  Building frontend..."
cd ../yew-frontend || exit 1
safe_run trunk build --release
echo "✅ Frontend built successfully"
echo ""

# Step 4: Verify builds exist
echo "🔍 Verifying builds..."
cd ..

if [ ! -d "game/dist" ]; then
    echo "❌ Game dist directory not found"
    exit 1
fi

if [ ! -d "yew-frontend/dist" ]; then
    echo "❌ Frontend dist directory not found"
    exit 1
fi

echo "✅ All builds verified"
echo ""

# Step 5: Build backend (just check, don't build release)
echo "🔧 Verifying backend compiles..."
cd backend || exit 1
safe_run cargo check
echo "✅ Backend verified"
echo ""

echo "🎉 DEPLOYMENT COMPLETE!"
echo "======================================"
echo ""
echo "🚀 To start the application:"
echo "   cd backend && cargo run"
echo ""
echo "🌐 Then open: http://localhost:3000"
echo ""
echo "📁 File structure:"
echo "   - Frontend:     http://localhost:3000/"
echo "   - Game:         http://localhost:3000/game"
echo "   - API:          http://localhost:3000/api/*"
echo "   - Database:     postgresql://localhost:5432/stellar_heads"
echo ""
echo "✅ Everything should work from a single backend server!"