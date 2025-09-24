# Stellar Heads Production Build Script
# This script builds the entire application for production deployment

Write-Host "🚀 Starting Stellar Heads production build..." -ForegroundColor Green

# Step 1: Clean previous builds
Write-Host "🧹 Cleaning previous builds..." -ForegroundColor Yellow
if (Test-Path "game/pkg") { Remove-Item -Path "game/pkg" -Recurse -Force }
if (Test-Path "yew-frontend/dist") { Remove-Item -Path "yew-frontend/dist" -Recurse -Force }
if (Test-Path "backend/static/game") { Remove-Item -Path "backend/static/game" -Recurse -Force }

# Step 2: Build WASM game module
Write-Host "🎮 Building WASM game module..." -ForegroundColor Yellow
Set-Location game
try {
    wasm-pack build --target web --release --out-dir pkg
    if ($LASTEXITCODE -ne 0) { throw "WASM build failed" }
    Write-Host "✅ WASM build successful" -ForegroundColor Green
} catch {
    Write-Host "❌ WASM build failed: $_" -ForegroundColor Red
    exit 1
} finally {
    Set-Location ..
}

# Step 3: Create backend static directory and copy WASM files
Write-Host "📁 Setting up backend static assets..." -ForegroundColor Yellow
if (!(Test-Path "backend/static")) { New-Item -Path "backend/static" -ItemType Directory -Force }
if (!(Test-Path "backend/static/game")) { New-Item -Path "backend/static/game" -ItemType Directory -Force }

Copy-Item -Path "game/pkg/*" -Destination "backend/static/game/" -Force -Recurse
Write-Host "✅ WASM files copied to backend" -ForegroundColor Green

# Step 4: Build frontend
Write-Host "🌐 Building frontend..." -ForegroundColor Yellow
Set-Location yew-frontend
try {
    trunk build --release
    if ($LASTEXITCODE -ne 0) { throw "Frontend build failed" }
    Write-Host "✅ Frontend build successful" -ForegroundColor Green
} catch {
    Write-Host "❌ Frontend build failed: $_" -ForegroundColor Red
    exit 1
} finally {
    Set-Location ..
}

# Step 5: Copy game assets to frontend dist
Write-Host "🎯 Copying game assets to frontend..." -ForegroundColor Yellow
if (!(Test-Path "yew-frontend/dist/game")) { New-Item -Path "yew-frontend/dist/game" -ItemType Directory -Force }
Copy-Item -Path "game/pkg/stellar_heads_game.js" -Destination "yew-frontend/dist/game/" -Force
Copy-Item -Path "game/pkg/stellar_heads_game_bg.wasm" -Destination "yew-frontend/dist/game/" -Force

# Step 6: Verify build
Write-Host "🔍 Verifying build..." -ForegroundColor Yellow
$requiredFiles = @(
    "game/pkg/stellar_heads_game.js",
    "game/pkg/stellar_heads_game_bg.wasm",
    "yew-frontend/dist/index.html",
    "yew-frontend/dist/game/stellar_heads_game.js",
    "yew-frontend/dist/game/stellar_heads_game_bg.wasm",
    "backend/static/game/stellar_heads_game.js",
    "backend/static/game/stellar_heads_game_bg.wasm"
)

$allExists = $true
foreach ($file in $requiredFiles) {
    if (!(Test-Path $file)) {
        Write-Host "❌ Missing: $file" -ForegroundColor Red
        $allExists = $false
    } else {
        Write-Host "✅ Found: $file" -ForegroundColor Green
    }
}

if ($allExists) {
    Write-Host "🎉 Production build completed successfully!" -ForegroundColor Green
    Write-Host "📦 Ready for deployment!" -ForegroundColor Cyan

    # Display file sizes
    Write-Host "`n📊 Build Statistics:" -ForegroundColor Cyan
    $wasmSize = (Get-Item "game/pkg/stellar_heads_game_bg.wasm").Length / 1KB
    $jsSize = (Get-Item "game/pkg/stellar_heads_game.js").Length / 1KB
    Write-Host "   WASM size: $([math]::Round($wasmSize, 2)) KB" -ForegroundColor White
    Write-Host "   JS size: $([math]::Round($jsSize, 2)) KB" -ForegroundColor White

    Write-Host "`n🚀 To start the production server, run:" -ForegroundColor Cyan
    Write-Host "   cd backend && cargo run --release" -ForegroundColor Yellow
} else {
    Write-Host "❌ Production build incomplete. Please check the errors above." -ForegroundColor Red
    exit 1
}