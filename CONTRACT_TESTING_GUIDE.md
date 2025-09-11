# 🧪 Contract Testing Guide

## ✅ Your Soroban Contract Integration is Ready!

### 🎯 **What You Can Test Now:**

1. **🏗️ Initialize Contract** 
   - Makes you the admin of the contract
   - Required before any other operations
   - Function: `initialize(env: Env, admin: Address)`

2. **🌟 Join Leaderboard** 
   - Submits initial score of 0 to join
   - Function: `submit_score(env: Env, player: Address, username: String, score: u64) -> bool`

3. **🎯 Test Score (1000 pts)**
   - Submits a test score of 1000 points
   - Function: `submit_score(env: Env, player: Address, username: String, score: u64) -> bool`

### 🔧 **Setup Steps:**

1. **Deploy Your Contract:**
   ```bash
   # Build the contract
   soroban contract build
   
   # Deploy to testnet
   soroban contract deploy \
     --wasm target/wasm32-unknown-unknown/release/your_contract.wasm \
     --source YOUR_KEYPAIR \
     --network testnet
   ```

2. **Update Contract ID:**
   - Edit `.env` file line 15: `LEADERBOARD_CONTRACT_ADDRESS=YOUR_CONTRACT_ID`
   - Restart backend: `cd backend && cargo run`

3. **Test the Flow:**
   - Login to frontend → Game page
   - Use "Contract Signing Test" panel
   - Try all 3 buttons to test different functions

### 🔄 **Complete Test Flow:**

```
Frontend → Backend → Soroban Contract → Freighter → Stellar Network
   │           │            │              │            │
   │           │     Creates XDR      Signs with      Submits to
   │      API Call      for your      your wallet      blockchain
   │                   contract
   │
User clicks test button
```

### 🎮 **Contract Functions Mapped:**

| Your Contract Function | API Endpoint | Test Button | Description |
|----------------------|--------------|-------------|-------------|
| `initialize(admin)` | `/api/soroban/test-signing` | "Initialize Contract" | Set contract admin |
| `submit_score(player, username, 0)` | `/api/soroban/test-signing` | "Join Leaderboard" | Join with score 0 |
| `submit_score(player, username, 1000)` | `/api/soroban/test-signing` | "Test Score (1000)" | Submit test score |
| `start_game_session(player, session_id)` | `/api/soroban/start-game` | (Game integration) | Start game session |
| `get_player_best(player)` | `/api/soroban/player-score` | (Read-only) | Get best score |

### 🛡️ **Transaction Signing Process:**

1. **Backend creates transaction XDR** using your contract address
2. **Frontend receives XDR** with network details  
3. **Freighter opens** showing YOUR contract address
4. **User confirms** - can see exactly which contract they're interacting with
5. **Transaction signed** and ready for submission
6. **Blockchain execution** - score stored in YOUR contract

### 🌟 **Key Features:**

- ✅ **Real contract integration** - Uses your actual deployed contract
- ✅ **Freighter wallet signing** - Secure transaction signing
- ✅ **Multiple test scenarios** - Initialize, join, submit scores
- ✅ **Error handling** - Comprehensive error messages
- ✅ **Transaction display** - Shows signed XDR for verification
- ✅ **Network configuration** - Testnet/Mainnet support

### 🚀 **Ready for Production:**

When you're ready to go live:
1. Deploy contract to **mainnet** 
2. Update `.env` with mainnet contract ID
3. Change `SOROBAN_NETWORK=mainnet` in `.env`
4. Test with small amounts first
5. Launch your game with blockchain leaderboards! 🎮

Your Stellar Heads game now has **full Soroban smart contract integration**! 🌟