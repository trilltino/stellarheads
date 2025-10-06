# Stellar Heads

**Multiplayer 2D football game with Soroban smart contract leaderboard and WebAssembly gameplay**

- Bevy game engine with physics simulation: collision detection, ball dynamics, goal scoring, player controls, and audio system
- Soroban smart contract for persistent on-chain leaderboard with score submission and global rankings
- WASM-compiled game binary (60MB) serving browser-based multiplayer with local and networked gameplay modes
- Full-stack architecture: Axum backend for API/static serving, PostgreSQL for game results, Yew frontend for UI/wallet integration
- Comprehensive testing suite: scoring logic unit tests, contract tests, repository tests, and auth middleware validation
