#!/bin/bash

# Build script for Stellar Heads game integration

echo "🎮 Building Stellar Heads Game for Web Integration..."

# Step 1: Build the game WASM
echo "📦 Building game WASM..."
cd game
trunk build --release --public-url "/"

# Step 2: Copy WASM files to yew-frontend dist
echo "📁 Copying WASM files to frontend..."
cd ..

# Create yew-frontend/dist directory if it doesn't exist
mkdir -p yew-frontend/dist

# Copy the built WASM files
cp game/dist/*.js yew-frontend/dist/ 2>/dev/null || echo "⚠️  No JS files to copy"
cp game/dist/*.wasm yew-frontend/dist/ 2>/dev/null || echo "⚠️  No WASM files to copy"

# Also copy to root for serving
cp game/dist/*.js yew-frontend/ 2>/dev/null || echo "⚠️  No JS files to copy to root"
cp game/dist/*.wasm yew-frontend/ 2>/dev/null || echo "⚠️  No WASM files to copy to root"

echo "✅ Build complete!"
echo ""
echo "Next steps:"
echo "1. cd yew-frontend && trunk serve --port 8080"
echo "2. Navigate to http://localhost:8080/game"
echo "3. Click 'Load Game' to initialize the Bevy game"