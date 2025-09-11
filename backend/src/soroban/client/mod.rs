use soroban_client::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaderboardTransaction {
    pub contract_address: String,
    pub player_address: String,
    pub username: String,
    pub score: u64,
    pub session_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub transaction_xdr: String,
    pub network_passphrase: String,
    pub success: bool,
    pub message: String,
}

pub struct SorobanLeaderboardClient {
    client: Client,
    contract_address: String,
    network_passphrase: String,
}

impl SorobanLeaderboardClient {
    pub fn new(contract_address: String, is_testnet: bool) -> Self {
        // Initialize the soroban-client 0.5.1
        let rpc_url = if is_testnet {
            "https://soroban-testnet.stellar.org"
        } else {
            "https://soroban-mainnet.stellar.org"
        };
        
        let network_passphrase = if is_testnet {
            "Test SDF Network ; September 2015"
        } else {
            "Public Global Stellar Network ; September 2015"
        };
        
        let client = Client::new(rpc_url)
            .expect("Failed to create Soroban client");
        
        Self {
            client,
            contract_address,
            network_passphrase: network_passphrase.to_string(),
        }
    }

    /// Create a transaction to submit a score to the leaderboard
    pub async fn create_submit_score_transaction(
        &self,
        player_public_key: &str,
        username: String,
        score: u64,
    ) -> Result<TransactionResponse, Box<dyn std::error::Error>> {
        
        // Use soroban-client 0.5.1 API
        // Your contract: submit_score(env: Env, player: Address, username: String, score: u64) -> bool
        
        let transaction_xdr = self.client
            .create_invoke_transaction(
                player_public_key, // source account
                &self.contract_address, // contract address
                "submit_score", // function name
                &[
                    serde_json::json!({"address": player_public_key}), // Address type
                    serde_json::json!(username), // String
                    serde_json::json!(score), // u64
                ],
                &self.network_passphrase,
            )
            .await?;
        
        Ok(TransactionResponse {
            transaction_xdr,
            network_passphrase: self.network_passphrase.clone(),
            success: true,
            message: "Transaction created successfully - ready for Freighter signing".to_string(),
        })
    }

    /// Create a transaction to start a game session
    pub async fn create_start_session_transaction(
        &self,
        player_public_key: &str,
        session_id: String,
    ) -> Result<TransactionResponse, Box<dyn std::error::Error>> {
        
        // Your contract: start_game_session(env: Env, player: Address, session_id: String) -> GameSession
        
        let transaction_xdr = self.client
            .create_invoke_transaction(
                player_public_key,
                &self.contract_address,
                "start_game_session",
                &[
                    serde_json::json!({"address": player_public_key}), // Address
                    serde_json::json!(session_id), // String
                ],
                &self.network_passphrase,
            )
            .await?;
        
        Ok(TransactionResponse {
            transaction_xdr,
            network_passphrase: self.network_passphrase.clone(),
            success: true,
            message: "Game session transaction created successfully - sign with Freighter".to_string(),
        })
    }

    /// Get leaderboard data (read-only, no transaction needed)
    pub async fn get_leaderboard(
        &self,
        limit: u32,
    ) -> Result<Vec<LeaderboardEntry>, Box<dyn std::error::Error>> {
        // This would use the client to read contract state
        // Implementation depends on the specific soroban-client version
        todo!("Implement leaderboard reading")
    }

    /// Get a player's best score (read-only)
    pub async fn get_player_best_score(
        &self,
        player_address: &str,
    ) -> Result<Option<PlayerScore>, Box<dyn std::error::Error>> {
        // Call the contract's get_player_best function
        // get_player_best(env: Env, player: Address) -> Option<PlayerScore>
        
        match self.client.call_contract(
            &self.contract_address,
            "get_player_best",
            &[serde_json::json!({"address": player_address})], // Address
        ).await {
            Ok(result) => {
                // Parse the result - this would depend on the soroban-client response format
                // For now, return None as a placeholder
                // TODO: Implement proper parsing of contract response
                println!("Contract call result: {:?}", result);
                Ok(None)
            },
            Err(e) => {
                println!("Failed to call contract: {}", e);
                Ok(None) // Return None instead of error for better UX
            }
        }
    }

    /// Initialize the contract (admin only)
    pub async fn create_initialize_transaction(
        &self,
        admin_address: &str,
    ) -> Result<TransactionResponse, Box<dyn std::error::Error>> {
        // Your contract: initialize(env: Env, admin: Address)
        
        let transaction_xdr = self.client
            .create_invoke_transaction(
                admin_address,
                &self.contract_address,
                "initialize",
                &[serde_json::json!({"address": admin_address})], // Address
                &self.network_passphrase,
            )
            .await?;
        
        Ok(TransactionResponse {
            transaction_xdr,
            network_passphrase: self.network_passphrase.clone(),
            success: true,
            message: "Contract initialization transaction created - sign to deploy".to_string(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub player_address: String,
    pub username: String,
    pub score: u64,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerScore {
    pub player_address: String,
    pub username: String,
    pub score: u64,
    pub timestamp: u64,
}