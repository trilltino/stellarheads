# 🌟 Stellar Heads Game Architecture Analysis

## Overview
Stellar Heads is a single-player 2D physics-based soccer game built with Bevy game engine that automatically submits game results to an Axum backend server. The architecture was simplified from a multiplayer networked game to a streamlined single-player experience.

## 🏗️ System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Stellar Heads Ecosystem                      │
├─────────────────────────────────┬───────────────────────────────┤
│           Bevy Game             │        Axum Backend           │
│         (Single Player)         │      (HTTP REST API)         │
├─────────────────────────────────┼───────────────────────────────┤
│  ┌─────────────────────────┐   │  ┌─────────────────────────┐  │
│  │    Game Logic           │   │  │   Game Results API      │  │
│  │  • Physics (Avian2D)    │   │  │  • POST /api/game/result│  │
│  │  • Scoring System       │   │  │  • GET /api/game/stats  │  │
│  │  • Player Movement      │   │  │                         │  │
│  │  • Ball Physics         │   HTTP │  User Management API   │  │
│  │  • Goals & Collision    ├───┼──┤  • POST /api/guest      │  │
│  └─────────────────────────┘   │  │  • User validation      │  │
│                                │  │                         │  │
│  ┌─────────────────────────┐   │  │  Database Integration   │  │
│  │  Result Submission      │   │  │  • PostgreSQL           │  │
│  │  • Match finish detect  │   │  │  • User storage         │  │
│  │  • HTTP POST to backend │   │  │  • Game results logging │  │
│  │  • Async tokio/reqwest  │   │  │                         │  │
│  └─────────────────────────┘   │  └─────────────────────────┘  │
└─────────────────────────────────┴───────────────────────────────┘
```

## 🎮 Game Components (Bevy)

### Core Systems

#### 1. **Gameplay Systems** (`game/src/shared/gameplay/`)
- **Ball Physics** (`ball.rs`): Physics-based ball with avian2d integration
- **Player Controller** (`player.rs`): Single-player character with movement, jumping, ground detection
- **Goals** (`goals.rs`): Goal detection and scoring triggers
- **Ground** (`ground.rs`): Field boundaries, walls, and collision surfaces
- **Collision** (`collision.rs`): Ball-player-goal collision detection

#### 2. **Scoring System** (`game/src/shared/scoring/scoring.rs`)
```rust
// Key Components
PlayerInfo {
    username: String,
    wallet_address: String,
}

Score {
    left_team: u32,
    right_team: u32,
    high_score: u32,
}

GameTimer {
    remaining_time: f32,
    match_duration: f32,
    is_finished: bool,
}
```

**Critical System**: `send_game_result_system` - Triggered on `MatchFinished` events:
1. Detects match completion (time up or score limit reached)
2. Creates `GameResult` with player info, final score, match duration
3. Spawns async HTTP request to backend using `reqwest`
4. Sends POST to `http://127.0.0.1:3000/api/game/result`

#### 3. **UI & State Management** (`game/src/shared/ui/`)
- **AppState**: LaunchMenu → InGame → GameOver flow
- **Game HUD**: Score display, timer, notifications
- **Inspector**: Enhanced debug UI with live game state editing

### Game Flow
```
LaunchMenu → InGame → [Match Plays] → MatchFinished → GameResult HTTP POST → Continue
```

## 🌐 Backend Integration (Axum)

### HTTP API Endpoints (`backend/src/main.rs`)
```rust
Router::new()
    .route("/api/guest", post(register_guest))           // User creation
    .route("/api/game/result", post(submit_game_result)) // Game result submission
    .route("/api/game/stats", get(get_player_stats))     // Player statistics
    .route("/join", post(create_join_transaction))       // Soroban integration
    .route("/check-joined", get(check_player_joined))    // Status check
    .route("/submit-signed-transaction", post(submit_signed_transaction)) // Blockchain
```

### Game Results Handler (`backend/src/handlers/game.rs`)
```rust
pub async fn submit_game_result(
    State(pool): State<PgPool>,
    Json(game_result): Json<GameResult>,
) -> Result<Json<GameResultResponse>, StatusCode>
```

**Process Flow:**
1. Receives `GameResult` from game client
2. Validates user exists in PostgreSQL database  
3. Logs comprehensive game information
4. Returns success/failure response
5. **Future**: Store results in game_results table

### Data Flow: Game → Backend

#### Shared Data Structures (`shared/src/dto/game.rs`)
```rust
GameResult {
    player_username: String,
    player_wallet_address: String,
    match_result: MatchResult,  // Win/Loss/Draw
    final_score: GameScore,     // left_team, right_team scores
    match_duration_seconds: f32,
    timestamp: DateTime<Utc>,
}
```

#### HTTP Request Flow
1. **Game Match Ends** → `MatchFinished` event fired
2. **Scoring System** → `send_game_result_system` catches event
3. **Data Creation** → `GameResult` struct populated with match data
4. **Async HTTP** → `tokio::spawn` + `reqwest::Client::post`
5. **Backend Processing** → Axum handler validates & logs
6. **Response** → Success/failure returned to game

## 🗄️ Database Integration

### User Management
- **PostgreSQL** via SQLx
- **User Table**: id, username, wallet_address, created_at
- **Validation**: Ensures game results tied to valid users

### Game Results (Future Enhancement)
```sql
-- Planned table structure
CREATE TABLE game_results (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    match_result VARCHAR(10), -- 'Win', 'Loss', 'Draw'
    left_score INTEGER,
    right_score INTEGER,
    duration_seconds REAL,
    created_at TIMESTAMP DEFAULT NOW()
);
```

## 🚀 Key Features

### 1. **Automatic Result Submission**
- No manual intervention required
- Triggers on natural game end conditions
- Robust error handling and logging

### 2. **User Association**
- Game results linked to specific username + wallet address
- Backend validation ensures data integrity
- Preparation for blockchain/Stellar integration

### 3. **Enhanced Inspector UI**
- Live game state editing (scores, timer, player info)
- Real-time notifications display
- World inspector for debugging
- Player information management

### 4. **Clean Architecture**
- Single-player focus (removed networking complexity)
- Shared data structures between game and backend
- Async HTTP communication
- Event-driven game result detection

## 🔧 Technical Stack

### Game (Bevy)
- **Engine**: Bevy 0.16.1
- **Physics**: Avian2D 0.3.1
- **UI**: bevy_egui + bevy-inspector-egui
- **HTTP**: reqwest 0.12 + tokio
- **Audio**: Bevy built-in audio systems

### Backend (Axum)
- **Server**: Axum 0.8.4
- **Database**: SQLx + PostgreSQL
- **Serialization**: Serde
- **CORS**: tower-http
- **Blockchain**: Soroban SDK (Stellar)

### Shared
- **Data**: Serde + Chrono for serialization
- **Types**: Shared DTOs between game and backend

## 🎯 Game Result Submission Details

### Trigger Conditions
1. **Score Limit**: First team reaches 5 goals
2. **Time Limit**: 120 seconds match duration expires
3. **Manual Reset**: Press 'R' key (resets but doesn't submit)

### Result Determination
- **Win**: Player's team (assumed left) has higher score
- **Loss**: AI team (right) has higher score  
- **Draw**: Equal scores at time expiration

### HTTP Request Example
```json
POST http://127.0.0.1:3000/api/game/result
{
    "player_username": "test_player",
    "player_wallet_address": "GCKFBEIYTKP33TO3QLCCKMXOMVK7X4PYC7_TEST_ADDRESS",
    "match_result": "Win",
    "final_score": {
        "left_team": 5,
        "right_team": 3
    },
    "match_duration_seconds": 87.5,
    "timestamp": "2025-09-13T09:59:00Z"
}
```

### Backend Response
```json
{
    "success": true,
    "message": "Game result recorded for test_player"
}
```

## 🎮 Enhanced Inspector Features

The inspector now provides:

### Game State Management
- **Live Score Editing**: Drag values to modify scores during gameplay
- **Timer Control**: Adjust remaining match time
- **Quick Reset**: Single button to reset match state

### Player Management  
- **Username Editing**: Change player username for result submission
- **Wallet Address**: Modify wallet address for blockchain integration
- **Live Preview**: See current player info that will be submitted

### Debug Information
- **Active Notifications**: View current game notifications and timers
- **Entity Browser**: Inspect all game entities and components
- **Resource Viewer**: Access all Bevy resources and systems

## 🔄 Future Enhancements

### Database
- Implement game_results table storage
- Add player statistics and leaderboards
- Historical match data and trends

### Blockchain Integration
- Submit game results to Stellar blockchain
- Token rewards for wins/achievements
- Smart contract integration with Soroban

### Game Features  
- Multiple difficulty levels
- Achievement system
- Replay system
- Tournament modes

---

*This architecture provides a solid foundation for a single-player game with backend integration, ready for future multiplayer and blockchain enhancements.*