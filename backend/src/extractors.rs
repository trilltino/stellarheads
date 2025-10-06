use axum::{
    extract::Query,
    http::StatusCode,
};
use serde::Deserialize;
use shared::WalletAddress;

/// Validated wallet address query parameter
#[derive(Debug, Clone, Deserialize)]
pub struct WalletAddressQuery {
    pub wallet_address: String,
}

impl WalletAddressQuery {
    /// Validate and convert to WalletAddress
    pub fn validate(self) -> Result<WalletAddress, (StatusCode, String)> {
        WalletAddress::new(self.wallet_address).map_err(|e| (StatusCode::BAD_REQUEST, e))
    }
}

/// Pagination parameters with sensible defaults
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

impl PaginationQuery {
    /// Validate pagination parameters
    pub fn validate(&self) -> Result<(), (StatusCode, String)> {
        if self.limit < 1 || self.limit > 100 {
            return Err((
                StatusCode::BAD_REQUEST,
                "Limit must be between 1 and 100".to_string(),
            ));
        }

        if self.offset < 0 {
            return Err((
                StatusCode::BAD_REQUEST,
                "Offset must be non-negative".to_string(),
            ));
        }

        Ok(())
    }

    pub fn limit(&self) -> i64 {
        self.limit.min(100).max(1)
    }

    pub fn offset(&self) -> i64 {
        self.offset.max(0)
    }
}

impl Default for PaginationQuery {
    fn default() -> Self {
        Self {
            limit: default_limit(),
            offset: 0,
        }
    }
}

/// Helper to extract and validate wallet address from query
pub type ValidatedWallet = Query<WalletAddressQuery>;

/// Helper to extract pagination parameters
pub type Pagination = Query<PaginationQuery>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_pagination() {
        let pagination = PaginationQuery::default();
        assert_eq!(pagination.limit, 20);
        assert_eq!(pagination.offset, 0);
    }

    #[test]
    fn test_pagination_validation() {
        let valid = PaginationQuery {
            limit: 50,
            offset: 10,
        };
        assert!(valid.validate().is_ok());

        let invalid_limit = PaginationQuery {
            limit: 150,
            offset: 0,
        };
        assert!(invalid_limit.validate().is_err());

        let invalid_offset = PaginationQuery {
            limit: 20,
            offset: -5,
        };
        assert!(invalid_offset.validate().is_err());
    }

    #[test]
    fn test_wallet_validation() {
        let valid = WalletAddressQuery {
            wallet_address: "GCRBGOBUEZCZF5GIMWSE7MUTY22MGYUMFZY7NZNVPUEDB6MSJIPAJMBZ".to_string(),
        };
        assert!(valid.validate().is_ok());

        let invalid = WalletAddressQuery {
            wallet_address: "invalid".to_string(),
        };
        assert!(invalid.validate().is_err());
    }
}
