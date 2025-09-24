# Stellar Heads ⚽

A physics-based 2D soccer game built with Rust and Bevy, featuring blockchain integration with Stellar for competitive leaderboards.

## 🏗️ Architecture

Stellar Heads is a complete full-stack application with the following components:

- **🎮 game**: Core Bevy game engine with physics-based soccer gameplay (WASM + Native)
- **🌐 backend**: Axum web server providing REST API and serving game assets
- **🎨 yew-frontend**: Yew-based web frontend with Freighter wallet integration
- **📦 shared**: Common data structures and DTOs used across all crates

### 🔧 Technology Stack

- **Frontend**: Yew (Rust WebAssembly), Trunk bundler
- **Backend**: Axum, Tower HTTP, PostgreSQL with SQLx
- **Game Engine**: Bevy 0.16.1, Avian2D physics
- **Blockchain**: Stellar network, Freighter wallet
- **Build Tools**: wasm-pack, trunk, cargo workspaces

## 🚀 Quick Start

### Prerequisites

1. **Rust** (latest stable version)
2. **Node.js** and **npm** (for web tools)
3. **PostgreSQL** (for database)
4. **Freighter Wallet** browser extension

### Install Required Tools

```powershell
# Install Rust WebAssembly tools
cargo install wasm-pack trunk

# Add WASM target
rustup target add wasm32-unknown-unknown
```

### 🏃‍♂️ Development (Quick Start)

```powershell
# Clone and start development environment
git clone <repository-url>
cd stellar_heads

# Start development servers (automatically builds WASM)
.\dev.ps1
```

This will:
- Build the WASM game module
- Start backend server on `http://localhost:3000`
- Start frontend development server on `http://localhost:8080`
- Serve game assets from backend

### 🚀 Production Build

```powershell
# Build everything for production
.\build.ps1
```

This creates optimized builds of:
- WASM game module with size optimizations
- Frontend bundle with compression
- Backend with static asset serving

## 🎮 Game Controls

- **A/D** - Move left/right
- **Space** - Jump
- **X** - Kick ball

## 🌐 Web Deployment

The application is designed for easy web deployment:

1. **Frontend**: Served as static files from `yew-frontend/dist/`
2. **Backend**: Rust binary serving API and static assets
3. **Game**: WASM module loaded dynamically in browser

### Environment Variables

Create `.env` files in the backend directory:

```env
DATABASE_URL=postgres://user:password@localhost:5432/stellar_heads
JOIN_CONTRACT_ADDRESS=<stellar_contract_address>
LEADERBOARD_CONTRACT_ADDRESS=<stellar_contract_address>
```

## 🔧 Development

### Project Structure

```
stellar_heads/
├── game/                 # Bevy game engine (WASM + Native)
│   ├── assets/          # Game assets (sprites, sounds)
│   ├── src/             # Game logic and systems
│   └── pkg/             # Generated WASM output
├── backend/             # Axum web server
│   ├── src/             # Server logic and handlers
│   ├── static/          # Static assets served to web
│   └── migrations/      # Database migrations
├── yew-frontend/        # Yew web frontend
│   ├── src/             # Frontend components
│   ├── dist/            # Built frontend assets
│   └── Trunk.toml       # Trunk configuration
├── shared/              # Common types and DTOs
├── build.ps1           # Production build script
└── dev.ps1             # Development script
```

### Building Individual Components

```powershell
# Build WASM game module only
cd game
wasm-pack build --target web --release

# Build frontend only
cd yew-frontend
trunk build --release

# Build backend only
cd backend
cargo build --release
```

## 🎯 Features

- **Physics-based gameplay** with realistic ball movement
- **Web-based multiplayer** ready architecture
- **Blockchain integration** with Stellar network
- **Wallet connectivity** via Freighter extension
- **Cross-platform** (Web, Desktop)
- **Production-ready** build pipeline

## 🛠️ Troubleshooting

### Common Issues

1. **WASM build fails**: Ensure wasm-pack is installed and up to date
2. **Frontend won't start**: Check that trunk is installed
3. **Game doesn't load**: Verify WASM files are copied to static directory
4. **Database connection fails**: Check PostgreSQL is running and .env is correct

### Performance

- WASM module is optimized for size (~200KB gzipped)
- Frontend uses code splitting for faster loading
- Backend serves compressed static assets

## 📝 License

[Add your license information here]

## 🤝 Contributing

[Add contribution guidelines here]