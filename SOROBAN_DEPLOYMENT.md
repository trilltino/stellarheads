# 🌟 Stellar Heads Soroban Contract Deployment Guide

## Prerequisites
1. Install Soroban CLI: `cargo install soroban-cli`
2. Have Testnet XLM in your wallet
3. Freighter wallet extension installed

## Step 1: Create Contract Project
```bash
# Create a new Soroban contract project
mkdir stellar_heads_contract
cd stellar_heads_contract
soroban contract init .
```

## Step 2: Replace Contract Code
Replace `src/lib.rs` with the leaderboard contract from `backend/src/soroban/contracts/leaderboard.rs`

## Step 3: Build the Contract
```bash
soroban contract build
```

## Step 4: Deploy to Testnet
```bash
# Configure Testnet
soroban network add testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"

# Generate identity (or use existing)
soroban keys generate alice

# Fund the account
soroban keys fund alice --network testnet

# Deploy the contract
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/stellar_heads_contract.wasm \
  --source alice \
  --network testnet

# This returns your CONTRACT_ID like: CBQHNAXSI55GX2GN6D67GK7BHVPSLJUGX5KPJXFYNO7PSRT4LANXICKF
```

## Step 5: Initialize the Contract
```bash
# Initialize with admin address
soroban contract invoke \
  --id YOUR_CONTRACT_ID \
  --source alice \
  --network testnet \
  -- initialize \
  --admin ALICE_PUBLIC_KEY
```

## Step 6: Update Environment Variables
Add to your `.env` file:
```env
LEADERBOARD_CONTRACT_ADDRESS=YOUR_CONTRACT_ID_HERE
SOROBAN_NETWORK=testnet
SOROBAN_RPC_URL=https://soroban-testnet.stellar.org:443
```

## Step 7: Test Contract Function
```bash
# Test submitting a score
soroban contract invoke \
  --id YOUR_CONTRACT_ID \
  --source alice \
  --network testnet \
  -- submit_score \
  --player ALICE_PUBLIC_KEY \
  --username "TestPlayer" \
  --score 1000
```

## Frontend Integration Flow:

1. **User plays game** → Achieves high score
2. **Game calls backend** → `/api/soroban/submit-score`
3. **Backend creates transaction** → Uses your deployed CONTRACT_ID
4. **Frontend signs with Freighter** → User approves transaction
5. **Backend submits to network** → Score stored on blockchain
6. **Leaderboard updates** → Reads from contract state

## Contract Functions Available:
- `submit_score(player, username, score)` - Submit game score
- `get_leaderboard(limit)` - Get top players
- `get_player_best(player)` - Get player's best score
- `start_game_session(player, session_id)` - Track game sessions

## Production Deployment (Mainnet):
```bash
# Same steps but with mainnet:
soroban network add mainnet \
  --rpc-url https://horizon-rpc.stellar.org:443 \
  --network-passphrase "Public Global Stellar Network ; September 2015"
```

**Note**: Mainnet deployments cost real XLM for fees!