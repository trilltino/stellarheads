// Soroban smart contract infrastructure modules

pub mod cache;
pub mod circuit_breaker;
pub mod client;
pub mod manager;
pub mod pool;
pub mod queue;
pub mod registry;

// Re-export commonly used types for easier imports
pub use cache::ContractCache;
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerStats};
pub use client::{generate_leaderboard_xdr, submit_signed_transaction, ContractConfig};
pub use manager::{ScalableContractManager, ContractMetrics, ContractInfo, HealthStatus};
pub use pool::{StellarRpcPool, PoolConfig, PoolStats, PooledRpcConnection};
pub use queue::{ContractQueue, ContractOperation, OperationPriority, QueueResult};
pub use registry::{
    ContractRegistry, ContractMetadata, ContractHandle, NetworkType,
    create_default_registry, RegistryStats,
};
