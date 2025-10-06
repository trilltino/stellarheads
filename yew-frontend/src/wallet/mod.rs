pub mod freighter;

pub use freighter::*;

/// Connected wallet information
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectedWallet {
    pub address: String,
    pub wallet_type: String,
}

impl ConnectedWallet {
    pub fn new(address: String, wallet_type: String) -> Self {
        Self {
            address,
            wallet_type,
        }
    }

    pub fn freighter(address: String) -> Self {
        Self::new(address, "freighter".to_string())
    }
}