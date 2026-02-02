//! # Blockchain Clients
//!
//! Clients for on-chain execution on Ethereum and L2 networks.
//!
//! ## Available Components
//!
//! - [`BlockchainClient`]: Trait for blockchain interactions
//! - [`EthereumClient`]: Ethereum and L2 client implementation
//! - [`GasPrice`]: Gas pricing (legacy and EIP-1559)
//! - [`GasEstimator`]: Gas estimation with buffer
//! - [`ChainId`]: Supported blockchain networks
//!
//! ## Supported Chains
//!
//! - Ethereum mainnet
//! - Polygon
//! - Arbitrum
//! - Optimism
//! - Base

pub mod client;
pub mod config;
pub mod ethereum;
pub mod gas;
pub mod tokens;

pub use client::{
    BlockchainClient, BlockchainError, BlockchainResult, ChainId, TxHash, TxPriority, TxReceipt,
};
pub use ethereum::EthereumClient;
pub use gas::{FeeHistory, GasEstimator, GasPrice};
