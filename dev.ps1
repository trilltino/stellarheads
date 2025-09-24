# Stellar Heads Development Script
# This script starts the development environment

Write-Host "🛠️ Starting Stellar Heads development environment..." -ForegroundColor Green

# Check if WASM tools are available
if (!(Get-Command "wasm-pack" -ErrorAction SilentlyContinue)) {
    Write-Host "❌ wasm-pack not found. Please install it first:" -ForegroundColor Red
    Write-Host "   cargo install wasm-pack" -ForegroundColor Yellow
    exit 1
}

if (!(Get-Command "trunk" -ErrorAction SilentlyContinue)) {
    Write-Host "❌ trunk not found. Please install it first:" -ForegroundColor Red
    Write-Host "   cargo install trunk" -ForegroundColor Yellow
    exit 1
}

# Build WASM game if needed
if (!(Test-Path "game/pkg/stellar_heads_game.js")) {
    Write-Host "🎮 Building WASM game module (first time)..." -ForegroundColor Yellow
    Set-Location game
    wasm-pack build --target web --dev
    Set-Location ..
}

# Ensure backend static directory exists
if (!(Test-Path "backend/static/game")) {
    New-Item -Path "backend/static/game" -ItemType Directory -Force
    Copy-Item -Path "game/pkg/*" -Destination "backend/static/game/" -Force -Recurse
}

Write-Host "🌐 Starting development servers..." -ForegroundColor Yellow

# Start backend server in background
Write-Host "🔧 Starting backend server on port 3000..." -ForegroundColor Cyan
Start-Process PowerShell -ArgumentList "-Command", "cd backend; cargo run" -WindowStyle Minimized

# Wait a bit for backend to start
Start-Sleep -Seconds 3

# Start frontend server
Write-Host "🎨 Starting frontend server on port 8080..." -ForegroundColor Cyan
Write-Host "📱 Frontend will be available at: http://localhost:8080" -ForegroundColor Green
Write-Host "🎮 Backend API available at: http://localhost:3000" -ForegroundColor Green
Write-Host "🎯 Game WASM served from: http://localhost:3000/game/" -ForegroundColor Green

Set-Location yew-frontend
trunk serve --port 8080