//! # Blockchain Client Trait
//!
//! Port definition for blockchain interactions.
//!
//! This module defines the [`BlockchainClient`] trait that abstracts
//! blockchain operations for Ethereum and L2 networks.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

use super::gas::GasPrice;

/// Supported blockchain networks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChainId {
    /// Ethereum mainnet (chain ID 1).
    Ethereum,
    /// Polygon mainnet (chain ID 137).
    Polygon,
    /// Arbitrum One (chain ID 42161).
    Arbitrum,
    /// Optimism mainnet (chain ID 10).
    Optimism,
    /// Base mainnet (chain ID 8453).
    Base,
}

impl ChainId {
    /// Returns the numeric chain ID.
    #[must_use]
    pub const fn as_u64(&self) -> u64 {
        match self {
            Self::Ethereum => 1,
            Self::Polygon => 137,
            Self::Arbitrum => 42161,
            Self::Optimism => 10,
            Self::Base => 8453,
        }
    }

    /// Creates a ChainId from a numeric chain ID.
    ///
    /// # Returns
    ///
    /// `Some(ChainId)` if the chain ID is supported, `None` otherwise.
    #[must_use]
    pub const fn from_u64(chain_id: u64) -> Option<Self> {
        match chain_id {
            1 => Some(Self::Ethereum),
            137 => Some(Self::Polygon),
            42161 => Some(Self::Arbitrum),
            10 => Some(Self::Optimism),
            8453 => Some(Self::Base),
            _ => None,
        }
    }

    /// Returns the chain name as a string.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Ethereum => "ethereum",
            Self::Polygon => "polygon",
            Self::Arbitrum => "arbitrum",
            Self::Optimism => "optimism",
            Self::Base => "base",
        }
    }

    /// Returns the average block time in milliseconds.
    #[must_use]
    pub const fn block_time_ms(&self) -> u64 {
        match self {
            Self::Ethereum => 12000,
            Self::Polygon => 2000,
            Self::Arbitrum => 250,
            Self::Optimism => 2000,
            Self::Base => 2000,
        }
    }

    /// Returns whether EIP-1559 is supported.
    #[must_use]
    pub const fn supports_eip1559(&self) -> bool {
        match self {
            Self::Ethereum | Self::Polygon | Self::Optimism | Self::Base => true,
            Self::Arbitrum => false,
        }
    }
}

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Transaction priority for gas pricing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TxPriority {
    /// Low priority - slower confirmation.
    Low,
    /// Medium priority - standard confirmation.
    #[default]
    Medium,
    /// High priority - faster confirmation.
    High,
}

impl fmt::Display for TxPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
        }
    }
}

/// Transaction hash type (32 bytes).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TxHash(pub String);

impl TxHash {
    /// Creates a new transaction hash.
    #[must_use]
    pub fn new(hash: impl Into<String>) -> Self {
        Self(hash.into())
    }

    /// Returns the hash as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TxHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for TxHash {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Transaction receipt with confirmation details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxReceipt {
    /// Transaction hash.
    pub tx_hash: TxHash,
    /// Block number where the transaction was included.
    pub block_number: u64,
    /// Gas used by the transaction.
    pub gas_used: u64,
    /// Effective gas price paid.
    pub effective_gas_price: u64,
    /// Whether the transaction succeeded.
    pub success: bool,
}

/// Error type for blockchain operations.
#[derive(Debug, Error)]
pub enum BlockchainError {
    /// RPC connection error.
    #[error("connection error: {0}")]
    Connection(String),

    /// Transaction submission error.
    #[error("transaction error: {0}")]
    Transaction(String),

    /// Transaction reverted.
    #[error("transaction reverted: {0}")]
    Reverted(String),

    /// Gas estimation error.
    #[error("gas estimation error: {0}")]
    GasEstimation(String),

    /// Nonce error.
    #[error("nonce error: {0}")]
    Nonce(String),

    /// Timeout waiting for confirmation.
    #[error("timeout: {0}")]
    Timeout(String),

    /// Chain not supported.
    #[error("unsupported chain: {0}")]
    UnsupportedChain(String),

    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),
}

impl BlockchainError {
    /// Creates a connection error.
    #[must_use]
    pub fn connection(msg: impl Into<String>) -> Self {
        Self::Connection(msg.into())
    }

    /// Creates a transaction error.
    #[must_use]
    pub fn transaction(msg: impl Into<String>) -> Self {
        Self::Transaction(msg.into())
    }

    /// Creates a reverted error.
    #[must_use]
    pub fn reverted(msg: impl Into<String>) -> Self {
        Self::Reverted(msg.into())
    }

    /// Creates a gas estimation error.
    #[must_use]
    pub fn gas_estimation(msg: impl Into<String>) -> Self {
        Self::GasEstimation(msg.into())
    }

    /// Creates a nonce error.
    #[must_use]
    pub fn nonce(msg: impl Into<String>) -> Self {
        Self::Nonce(msg.into())
    }

    /// Creates a timeout error.
    #[must_use]
    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::Timeout(msg.into())
    }

    /// Creates an unsupported chain error.
    #[must_use]
    pub fn unsupported_chain(msg: impl Into<String>) -> Self {
        Self::UnsupportedChain(msg.into())
    }

    /// Creates an internal error.
    #[must_use]
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

/// Result type for blockchain operations.
pub type BlockchainResult<T> = Result<T, BlockchainError>;

/// Trait for blockchain client operations.
///
/// Provides an abstraction over blockchain interactions for Ethereum
/// and L2 networks.
#[async_trait]
pub trait BlockchainClient: Send + Sync + fmt::Debug {
    /// Returns the chain ID this client is connected to.
    fn chain_id(&self) -> ChainId;

    /// Returns the current block number.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    async fn get_block_number(&self) -> BlockchainResult<u64>;

    /// Returns the balance of an address in wei.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to query
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    async fn get_balance(&self, address: &str) -> BlockchainResult<u128>;

    /// Estimates gas for a transaction.
    ///
    /// # Arguments
    ///
    /// * `to` - Destination address
    /// * `data` - Transaction calldata
    /// * `value` - Value in wei
    ///
    /// # Errors
    ///
    /// Returns an error if gas estimation fails.
    async fn estimate_gas(&self, to: &str, data: &[u8], value: u128) -> BlockchainResult<u64>;

    /// Returns the current gas price.
    ///
    /// # Arguments
    ///
    /// * `priority` - Transaction priority level
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    async fn get_gas_price(&self, priority: TxPriority) -> BlockchainResult<GasPrice>;

    /// Submits a transaction to the network.
    ///
    /// # Arguments
    ///
    /// * `to` - Destination address
    /// * `data` - Transaction calldata
    /// * `value` - Value in wei
    /// * `gas_limit` - Gas limit
    /// * `gas_price` - Gas price configuration
    ///
    /// # Errors
    ///
    /// Returns an error if transaction submission fails.
    async fn send_transaction(
        &self,
        to: &str,
        data: &[u8],
        value: u128,
        gas_limit: u64,
        gas_price: GasPrice,
    ) -> BlockchainResult<TxHash>;

    /// Waits for a transaction to be confirmed.
    ///
    /// # Arguments
    ///
    /// * `tx_hash` - Transaction hash to wait for
    /// * `confirmations` - Number of confirmations to wait for
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails or times out.
    async fn wait_for_confirmation(
        &self,
        tx_hash: &TxHash,
        confirmations: u64,
    ) -> BlockchainResult<TxReceipt>;

    /// Returns the nonce for an address.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to query
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    async fn get_nonce(&self, address: &str) -> BlockchainResult<u64>;

    /// Checks if the client is connected and healthy.
    ///
    /// # Errors
    ///
    /// Returns an error if the health check fails.
    async fn health_check(&self) -> BlockchainResult<()>;
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn chain_id_as_u64() {
        assert_eq!(ChainId::Ethereum.as_u64(), 1);
        assert_eq!(ChainId::Polygon.as_u64(), 137);
        assert_eq!(ChainId::Arbitrum.as_u64(), 42161);
        assert_eq!(ChainId::Optimism.as_u64(), 10);
        assert_eq!(ChainId::Base.as_u64(), 8453);
    }

    #[test]
    fn chain_id_from_u64() {
        assert_eq!(ChainId::from_u64(1), Some(ChainId::Ethereum));
        assert_eq!(ChainId::from_u64(137), Some(ChainId::Polygon));
        assert_eq!(ChainId::from_u64(42161), Some(ChainId::Arbitrum));
        assert_eq!(ChainId::from_u64(10), Some(ChainId::Optimism));
        assert_eq!(ChainId::from_u64(8453), Some(ChainId::Base));
        assert_eq!(ChainId::from_u64(999), None);
    }

    #[test]
    fn chain_id_display() {
        assert_eq!(ChainId::Ethereum.to_string(), "ethereum");
        assert_eq!(ChainId::Polygon.to_string(), "polygon");
    }

    #[test]
    fn chain_id_supports_eip1559() {
        assert!(ChainId::Ethereum.supports_eip1559());
        assert!(ChainId::Polygon.supports_eip1559());
        assert!(!ChainId::Arbitrum.supports_eip1559());
    }

    #[test]
    fn tx_hash_display() {
        let hash = TxHash::new("0x1234");
        assert_eq!(hash.to_string(), "0x1234");
        assert_eq!(hash.as_str(), "0x1234");
    }

    #[test]
    fn tx_priority_default() {
        assert_eq!(TxPriority::default(), TxPriority::Medium);
    }

    #[test]
    fn blockchain_error_display() {
        let err = BlockchainError::connection("test");
        assert_eq!(err.to_string(), "connection error: test");

        let err = BlockchainError::reverted("out of gas");
        assert_eq!(err.to_string(), "transaction reverted: out of gas");
    }

    #[test]
    fn chain_id_serde_roundtrip() {
        let chain = ChainId::Ethereum;
        let json = serde_json::to_string(&chain).unwrap();
        assert_eq!(json, "\"ethereum\"");
        let deserialized: ChainId = serde_json::from_str(&json).unwrap();
        assert_eq!(chain, deserialized);
    }
}
