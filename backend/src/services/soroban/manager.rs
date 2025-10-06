use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn, error};
use serde::{Deserialize, Serialize};

use super::{
    registry::{ContractRegistry, ContractMetadata, create_default_registry},
    queue::{ContractQueue, ContractOperation, OperationPriority, QueueResult},
    pool::PoolConfig,
    circuit_breaker::CircuitBreakerConfig,
};
use crate::error::{AppError, Result};
use shared::dto::contract::LeaderboardFunction;

/// High-level contract manager that orchestrates all scalability components
pub struct ScalableContractManager {
    registry: Arc<ContractRegistry>,
    queue: Arc<ContractQueue>,
    metrics: Arc<tokio::sync::RwLock<ContractMetrics>>,
}

impl ScalableContractManager {
    /// Create a new scalable contract manager with all features enabled
    pub async fn new() -> Result<Self> {
        info!("🚀 Initializing Scalable Contract Manager");

        // Create contract registry with default contracts
        let registry = create_default_registry()
            .await
            .map_err(|e| AppError::Config(format!("Failed to create registry: {}", e)))?;

        // Create async queue for operations
        let queue = Arc::new(ContractQueue::new());

        // Initialize metrics
        let metrics = Arc::new(tokio::sync::RwLock::new(ContractMetrics::default()));

        // Start background tasks
        Self::start_background_tasks(queue.clone(), metrics.clone());

        info!("✅ Scalable Contract Manager initialized successfully");

        Ok(Self {
            registry,
            queue,
            metrics,
        })
    }

    /// Generate XDR for contract function with all scalability features
    pub async fn generate_xdr(
        &self,
        contract_id: &str,
        source_account: &str,
        function: &LeaderboardFunction,
    ) -> Result<String> {
        // Get contract handle from registry
        let handle = self
            .registry
            .get(contract_id)
            .await
            .ok_or_else(|| AppError::Config(format!("Contract not found: {}", contract_id)))?;

        // Check cache first
        let cache_key = format!("xdr:{}:{}:{}", contract_id, source_account, function.name());
        if let Some(cached_xdr) = handle.cache.get(&cache_key).await {
            info!("✅ XDR retrieved from cache");
            self.record_cache_hit().await;
            return String::from_utf8(cached_xdr)
                .map_err(|e| AppError::XdrEncoding(format!("Invalid cached XDR: {}", e)));
        }

        self.record_cache_miss().await;

        // Get RPC connection from pool (validates connection is available)
        let _connection = handle
            .get_rpc_connection()
            .await
            .map_err(|e| AppError::StellarRpc(e))?;

        // Execute with circuit breaker protection
        let xdr_result = handle
            .call_with_protection(async {
                // Call the actual XDR generation
                super::client::generate_leaderboard_xdr(
                    &super::client::ContractConfig {
                        contract_id: handle.metadata.contract_id.clone(),
                        network_passphrase: handle.metadata.network_passphrase.clone(),
                        rpc_url: handle.metadata.rpc_url.clone(),
                    },
                    source_account,
                    function,
                )
                .await
            })
            .await
            .map_err(|e| AppError::StellarRpc(e))?;

        // Cache the result (1 minute TTL for XDR)
        handle
            .cache
            .set(cache_key, xdr_result.clone().into_bytes(), Some(Duration::from_secs(60)))
            .await;

        self.record_xdr_generated().await;
        Ok(xdr_result)
    }

    /// Submit signed transaction via async queue with retry logic
    pub async fn submit_transaction(
        &self,
        contract_id: &str,
        source_account: String,
        function: LeaderboardFunction,
        signed_xdr: String,
        priority: Option<OperationPriority>,
    ) -> Result<String> {
        // Create operation
        let operation = ContractOperation::new(
            contract_id.to_string(),
            function.name().to_string(),
            source_account,
            Some(signed_xdr),
        )
        .with_priority(priority.unwrap_or(OperationPriority::Normal))
        .with_max_retries(3);

        // Submit to queue
        let operation_id = self
            .queue
            .submit(operation)
            .await
            .map_err(|e| AppError::Transaction(e))?;

        self.record_transaction_submitted().await;
        Ok(operation_id)
    }

    /// Get operation result from queue
    pub async fn get_operation_result(&self) -> Option<QueueResult> {
        self.queue.next_result().await
    }

    /// Register a new contract dynamically
    pub async fn register_contract(&self, metadata: ContractMetadata) -> Result<()> {
        self.registry
            .register(metadata)
            .await
            .map_err(|e| AppError::Config(e))
    }

    /// List all registered contracts
    pub async fn list_contracts(&self) -> Vec<ContractMetadata> {
        self.registry.list_all().await
    }

    /// Get comprehensive system metrics
    pub async fn get_metrics(&self) -> ContractMetrics {
        self.metrics.read().await.clone()
    }

    /// Get detailed contract information
    pub async fn get_contract_info(&self, contract_id: &str) -> Result<ContractInfo> {
        let handle = self
            .registry
            .get(contract_id)
            .await
            .ok_or_else(|| AppError::Config(format!("Contract not found: {}", contract_id)))?;

        Ok(ContractInfo {
            metadata: handle.metadata.clone(),
            pool_stats: handle.rpc_pool.stats().await,
            circuit_breaker_stats: handle.circuit_breaker.stats().await,
            cache_stats: handle.cache.stats().await,
        })
    }

    /// Health check for the contract manager
    pub async fn health_check(&self) -> HealthStatus {
        let metrics = self.metrics.read().await.clone();
        let registry_stats = self.registry.stats().await;

        HealthStatus {
            healthy: registry_stats.enabled_contracts > 0,
            total_contracts: registry_stats.total_contracts,
            enabled_contracts: registry_stats.enabled_contracts,
            total_operations: metrics.total_operations,
            failed_operations: metrics.failed_operations,
            cache_hit_rate: metrics.cache_hit_rate(),
        }
    }

    // Internal metric recording methods
    async fn record_cache_hit(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.cache_hits += 1;
    }

    async fn record_cache_miss(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.cache_misses += 1;
    }

    async fn record_xdr_generated(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.xdr_generated += 1;
        metrics.total_operations += 1;
    }

    async fn record_transaction_submitted(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.transactions_submitted += 1;
        metrics.total_operations += 1;
    }

    /// Start background tasks for queue processing, cache cleanup, etc.
    fn start_background_tasks(
        queue: Arc<ContractQueue>,
        metrics: Arc<tokio::sync::RwLock<ContractMetrics>>,
    ) {
        // Queue result processor
        tokio::spawn(async move {
            info!("🔄 Starting queue result processor");

            while let Some(result) = queue.next_result().await {
                match result {
                    QueueResult::Success { operation_id, result } => {
                        info!("✅ Operation {} succeeded: {}", operation_id, result);
                        let mut m = metrics.write().await;
                        m.successful_operations += 1;
                    }
                    QueueResult::Retry { operation_id, attempt } => {
                        warn!("🔄 Operation {} retry attempt {}", operation_id, attempt);
                        let mut m = metrics.write().await;
                        m.retried_operations += 1;
                    }
                    QueueResult::Failed { operation_id, error } => {
                        error!("❌ Operation {} failed: {}", operation_id, error);
                        let mut m = metrics.write().await;
                        m.failed_operations += 1;
                    }
                }
            }
        });
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContractMetrics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub retried_operations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub xdr_generated: u64,
    pub transactions_submitted: u64,
}

impl ContractMetrics {
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            return 0.0;
        }
        (self.cache_hits as f64 / total as f64) * 100.0
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_operations == 0 {
            return 0.0;
        }
        (self.successful_operations as f64 / self.total_operations as f64) * 100.0
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ContractInfo {
    pub metadata: ContractMetadata,
    pub pool_stats: super::pool::PoolStats,
    pub circuit_breaker_stats: super::circuit_breaker::CircuitBreakerStats,
    pub cache_stats: super::cache::CacheStats,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub total_contracts: usize,
    pub enabled_contracts: usize,
    pub total_operations: u64,
    pub failed_operations: u64,
    pub cache_hit_rate: f64,
}

/// Configuration for the scalable contract manager
#[derive(Debug, Clone)]
pub struct ContractManagerConfig {
    pub pool_config: PoolConfig,
    pub circuit_breaker_config: CircuitBreakerConfig,
    pub cache_ttl: Duration,
}

impl Default for ContractManagerConfig {
    fn default() -> Self {
        Self {
            pool_config: PoolConfig::default(),
            circuit_breaker_config: CircuitBreakerConfig::default(),
            cache_ttl: Duration::from_secs(300),
        }
    }
}
