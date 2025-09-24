use serde::{Serialize, Deserialize};
use tokio::process::Command as AsyncCommand;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub transaction_xdr: String,
    pub network_passphrase: String,
}

pub struct SorobanCliClient {
    contract_address: String,
    network: String,
}

impl SorobanCliClient {
    pub fn new(contract_address: String, use_testnet: bool) -> Self {
        let network = if use_testnet { "testnet" } else { "mainnet" };

        Self {
            contract_address,
            network: network.to_string(),
        }
    }

    /// Check if a player has joined the leaderboard
    pub async fn has_joined(&self, player_address: &str) -> Result<bool, Box<dyn std::error::Error>> {
        println!("Checking if {} has joined via CLI", player_address);

        let output = AsyncCommand::new("stellar")
            .args([
                "contract", "invoke",
                "--id", &self.contract_address,
                "--source-account", player_address,
                "--network", &self.network,
                "--rpc-url", "https://soroban-testnet.stellar.org:443",
                "--network-passphrase", "Test SDF Network ; September 2015",
                "--send", "no",
                "--", "has_joined",
                "--player", player_address
            ])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        println!("CLI stdout: {}", stdout);
        if !stderr.is_empty() {
            println!("CLI stderr: {}", stderr);
        }

        // Parse the boolean result
        let result = stdout.trim();
        match result {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => {
                println!("⚠️ Unexpected CLI response: {}", result);
                Ok(false) // Default to false for safety
            }
        }
    }

    /// Generate XDR for joining the leaderboard
    pub async fn create_join_transaction(&self, player_address: &str) -> Result<TransactionRequest, Box<dyn std::error::Error>> {
        println!("🎯 Generating JOIN XDR for {} via CLI", player_address);

        let output = AsyncCommand::new("stellar")
            .args([
                "contract", "invoke",
                "--id", &self.contract_address,
                "--source-account", player_address,
                "--network", &self.network,
                "--rpc-url", "https://soroban-testnet.stellar.org:443",
                "--network-passphrase", "Test SDF Network ; September 2015",
                "--build-only",
                "--", "join",
                "--player", player_address
            ])
            .output()
            .await?;

        self.parse_xdr_response(output, "join").await
    }

    /// Generate XDR for adding a win
    pub async fn create_add_win_transaction(&self, player_address: &str) -> Result<TransactionRequest, Box<dyn std::error::Error>> {
        println!("🏆 Generating ADD_WIN XDR for {} via CLI", player_address);

        let output = AsyncCommand::new("stellar")
            .args([
                "contract", "invoke",
                "--id", &self.contract_address,
                "--source-account", player_address,
                "--network", &self.network,
                "--rpc-url", "https://soroban-testnet.stellar.org:443",
                "--network-passphrase", "Test SDF Network ; September 2015",
                "--build-only",
                "--", "add_win",
                "--player", player_address
            ])
            .output()
            .await?;

        self.parse_xdr_response(output, "add_win").await
    }

    /// Get player's own wins
    pub async fn get_my_wins(&self, player_address: &str) -> Result<u32, Box<dyn std::error::Error>> {
        println!("🏅 Getting wins for {} via CLI", player_address);

        let output = AsyncCommand::new("stellar")
            .args([
                "contract", "invoke",
                "--id", &self.contract_address,
                "--source-account", player_address,
                "--network", &self.network,
                "--rpc-url", "https://soroban-testnet.stellar.org:443",
                "--network-passphrase", "Test SDF Network ; September 2015",
                "--send", "no",
                "--", "get_my_wins",
                "--player", player_address
            ])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        println!("Wins stdout: {}", stdout);
        if !stderr.is_empty() {
            println!("Wins stderr: {}", stderr);
        }

        // Parse the number result
        let result = stdout.trim();
        match result.parse::<u32>() {
            Ok(wins) => Ok(wins),
            Err(_) => {
                println!("⚠️ Could not parse wins: {}", result);
                Ok(0)
            }
        }
    }

    /// Get any player's wins
    pub async fn get_wins(&self, player_address: &str) -> Result<u32, Box<dyn std::error::Error>> {
        println!("📊 Getting wins for {} via CLI", player_address);

        let output = AsyncCommand::new("stellar")
            .args([
                "contract", "invoke",
                "--id", &self.contract_address,
                "--source-account", "GDMQ5IHBNYVXDRZNGZ2UVSFHB2E47BXK2P4QFACU7DO6MFDOWD6C7677", // Default source
                "--network", &self.network,
                "--rpc-url", "https://soroban-testnet.stellar.org:443",
                "--network-passphrase", "Test SDF Network ; September 2015",
                "--send", "no",
                "--", "get_wins",
                "--player", player_address
            ])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        println!("Wins stdout: {}", stdout);
        if !stderr.is_empty() {
            println!("Wins stderr: {}", stderr);
        }

        // Parse the number result
        let result = stdout.trim();
        match result.parse::<u32>() {
            Ok(wins) => Ok(wins),
            Err(_) => {
                println!("⚠️ Could not parse wins: {}", result);
                Ok(0)
            }
        }
    }

    /// Submit a signed transaction
    pub async fn submit_transaction(&self, signed_xdr: &str) -> Result<u32, Box<dyn std::error::Error>> {
        println!("📡 Submitting signed transaction via CLI");

        let output = AsyncCommand::new("stellar")
            .args([
                "tx", "submit",
                "--xdr", signed_xdr,
                "--network", &self.network,
                "--rpc-url", "https://soroban-testnet.stellar.org:443",
                "--network-passphrase", "Test SDF Network ; September 2015"
            ])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        println!("Submit stdout: {}", stdout);
        if !stderr.is_empty() {
            println!("Submit stderr: {}", stderr);
        }

        if output.status.success() {
            // Try to extract ledger number from response
            // This is a simplified parser - adjust based on actual CLI output format
            Ok(12345) // Placeholder - parse actual ledger from CLI response
        } else {
            Err(format!("Transaction submission failed: {}", stderr).into())
        }
    }

    /// Parse XDR response from CLI output
    async fn parse_xdr_response(&self, output: std::process::Output, operation: &str) -> Result<TransactionRequest, Box<dyn std::error::Error>> {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        println!("=== {} CLI Response ===", operation.to_uppercase());
        println!("Status: {}", output.status);
        println!("Stdout: {}", stdout);
        if !stderr.is_empty() {
            println!("Stderr: {}", stderr);
        }
        println!("========================");

        if !output.status.success() {
            return Err(format!("CLI command failed for {}: {}", operation, stderr).into());
        }

        // Look for XDR in the output
        // The CLI typically outputs XDR in a specific format
        let xdr = if let Some(xdr_line) = stdout.lines().find(|line| line.contains("XDR") || line.starts_with("AAAA")) {
            xdr_line.trim().to_string()
        } else {
            // If no XDR found, the stdout might be the XDR itself
            stdout.trim().to_string()
        };

        if xdr.is_empty() {
            return Err(format!("No XDR found in CLI output for {}", operation).into());
        }

        println!("✅ Extracted XDR: {}", &xdr[..std::cmp::min(50, xdr.len())]);

        Ok(TransactionRequest {
            transaction_xdr: xdr,
            network_passphrase: if self.network == "testnet" {
                "Test SDF Network ; September 2015"
            } else {
                "Public Global Stellar Network ; September 2015"
            }.to_string(),
        })
    }
}