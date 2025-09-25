use serde::{Serialize, Deserialize};
use tokio::process::Command as AsyncCommand;
use tracing::info;

// Network constants
const TESTNET_RPC_URL: &str = "https://soroban-testnet.stellar.org:443";
const TESTNET_PASSPHRASE: &str = "Test SDF Network ; September 2015";
const MAINNET_PASSPHRASE: &str = "Public Global Stellar Network ; September 2015";
// Removed DEFAULT_SOURCE_ACCOUNT - was unused

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub transaction_xdr: String,
    pub network_passphrase: String,
}

pub struct SorobanCliClient {
    contract_address: String,
    network: String,
    rpc_url: String,
    network_passphrase: String,
}

impl SorobanCliClient {
    pub fn new(contract_address: String, use_testnet: bool) -> Self {
        let (network, rpc_url, network_passphrase) = if use_testnet {
            ("testnet", TESTNET_RPC_URL, TESTNET_PASSPHRASE)
        } else {
            ("mainnet", "https://horizon.stellar.org", MAINNET_PASSPHRASE)
        };

        Self {
            contract_address,
            network: network.to_string(),
            rpc_url: rpc_url.to_string(),
            network_passphrase: network_passphrase.to_string(),
        }
    }

    pub async fn has_joined(&self, player_address: &str) -> Result<bool, Box<dyn std::error::Error>> {
        info!("Checking if {} has joined via CLI", player_address);

        let output = AsyncCommand::new("stellar")
            .args([
                "contract", "invoke",
                "--id", &self.contract_address,
                "--source-account", player_address,
                "--network", &self.network,
                "--rpc-url", &self.rpc_url,
                "--network-passphrase", &self.network_passphrase,
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

        let result = stdout.trim();
        match result {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => {
                println!("⚠️ Unexpected CLI response: {}", result);
                Ok(false)
            }
        }
    }

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

        let result = stdout.trim();
        match result.parse::<u32>() {
            Ok(wins) => Ok(wins),
            Err(_) => {
                println!("⚠️ Could not parse wins: {}", result);
                Ok(0)
            }
        }
    }

    pub async fn get_wins(&self, player_address: &str) -> Result<u32, Box<dyn std::error::Error>> {
        println!("📊 Getting wins for {} via CLI", player_address);

        let output = AsyncCommand::new("stellar")
            .args([
                "contract", "invoke",
                "--id", &self.contract_address,
                "--source-account", "GDMQ5IHBNYVXDRZNGZ2UVSFHB2E47BXK2P4QFACU7DO6MFDOWD6C7677",
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

        let result = stdout.trim();
        match result.parse::<u32>() {
            Ok(wins) => Ok(wins),
            Err(_) => {
                println!("⚠️ Could not parse wins: {}", result);
                Ok(0)
            }
        }
    }

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
            // Parse ledger sequence from output
            if let Some(ledger_line) = stdout.lines().find(|line| line.contains("ledger") || line.contains("sequence")) {
                if let Some(ledger_str) = ledger_line.split_whitespace().find(|s| s.chars().all(|c| c.is_numeric())) {
                    return Ok(ledger_str.parse().unwrap_or(0));
                }
            }
            Ok(0) // Default if we can't parse ledger
        } else {
            Err(format!("Transaction submission failed: {}", stderr).into())
        }
    }

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

        let xdr = if let Some(xdr_line) = stdout.lines().find(|line| line.contains("XDR") || line.starts_with("AAAA")) {
            xdr_line.trim().to_string()
        } else {
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