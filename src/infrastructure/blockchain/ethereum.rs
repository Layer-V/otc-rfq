//! # Ethereum Client
//!
//! Ethereum and L2 client implementation using ethers-rs.
//!
//! Provides blockchain interactions for Ethereum mainnet and L2 networks
//! including Polygon, Arbitrum, Optimism, and Base.

use super::client::{
    BlockchainClient, BlockchainError, BlockchainResult, ChainId, TxHash, TxPriority, TxReceipt,
};
use super::gas::{FeeHistory, GasEstimator, GasPrice};
use async_trait::async_trait;
use ethers::prelude::*;
use std::sync::Arc;

/// Ethereum client implementation using ethers-rs.
///
/// Supports Ethereum mainnet and L2 networks with RPC failover.
#[derive(Debug)]
pub struct EthereumClient {
    /// The chain this client is connected to.
    chain_id: ChainId,
    /// Primary RPC provider.
    provider: Arc<Provider<Http>>,
    /// Backup RPC URLs for failover.
    backup_urls: Vec<String>,
    /// Gas estimator with buffer.
    gas_estimator: GasEstimator,
}

impl EthereumClient {
    /// Creates a new Ethereum client.
    ///
    /// # Arguments
    ///
    /// * `chain_id` - The chain to connect to
    /// * `rpc_url` - Primary RPC endpoint URL
    /// * `backup_urls` - Backup RPC URLs for failover
    ///
    /// # Errors
    ///
    /// Returns an error if the provider cannot be created.
    pub fn new(
        chain_id: ChainId,
        rpc_url: &str,
        backup_urls: Vec<String>,
    ) -> BlockchainResult<Self> {
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| BlockchainError::connection(e.to_string()))?;

        Ok(Self {
            chain_id,
            provider: Arc::new(provider),
            backup_urls,
            gas_estimator: GasEstimator::default(),
        })
    }

    /// Creates a client with a custom gas buffer.
    ///
    /// # Arguments
    ///
    /// * `chain_id` - The chain to connect to
    /// * `rpc_url` - Primary RPC endpoint URL
    /// * `backup_urls` - Backup RPC URLs for failover
    /// * `gas_buffer_percent` - Gas buffer percentage
    ///
    /// # Errors
    ///
    /// Returns an error if the provider cannot be created.
    pub fn with_gas_buffer(
        chain_id: ChainId,
        rpc_url: &str,
        backup_urls: Vec<String>,
        gas_buffer_percent: u64,
    ) -> BlockchainResult<Self> {
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| BlockchainError::connection(e.to_string()))?;

        Ok(Self {
            chain_id,
            provider: Arc::new(provider),
            backup_urls,
            gas_estimator: GasEstimator::new(gas_buffer_percent),
        })
    }

    /// Returns the gas estimator.
    #[must_use]
    pub fn gas_estimator(&self) -> &GasEstimator {
        &self.gas_estimator
    }

    /// Returns the backup RPC URLs.
    #[must_use]
    pub fn backup_urls(&self) -> &[String] {
        &self.backup_urls
    }

    /// Attempts to switch to a backup provider.
    ///
    /// # Errors
    ///
    /// Returns an error if no backup providers are available or all fail.
    pub async fn try_failover(&mut self) -> BlockchainResult<()> {
        for url in &self.backup_urls {
            match Provider::<Http>::try_from(url.as_str()) {
                Ok(provider) => {
                    // Test the connection
                    if provider.get_block_number().await.is_ok() {
                        self.provider = Arc::new(provider);
                        return Ok(());
                    }
                }
                Err(_) => continue,
            }
        }

        Err(BlockchainError::connection(
            "all backup providers failed".to_string(),
        ))
    }

    /// Fetches fee history for EIP-1559 gas pricing.
    ///
    /// # Arguments
    ///
    /// * `block_count` - Number of blocks to fetch history for
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn get_fee_history(&self, block_count: u64) -> BlockchainResult<FeeHistory> {
        let history = self
            .provider
            .fee_history(block_count, BlockNumber::Latest, &[25.0, 50.0, 75.0])
            .await
            .map_err(|e| BlockchainError::connection(e.to_string()))?;

        let base_fees: Vec<u64> = history
            .base_fee_per_gas
            .iter()
            .map(|f| f.as_u64())
            .collect();

        let priority_fees: Vec<Vec<u64>> = history
            .reward
            .iter()
            .map(|block_rewards| block_rewards.iter().map(|r| r.as_u64()).collect())
            .collect();

        Ok(FeeHistory::new(base_fees, priority_fees))
    }

    /// Calculates EIP-1559 gas price based on priority.
    async fn calculate_eip1559_price(&self, priority: TxPriority) -> BlockchainResult<GasPrice> {
        let fee_history = self.get_fee_history(10).await?;

        let max_fee = fee_history.recommended_max_fee();
        let priority_index = match priority {
            TxPriority::Low => 0,
            TxPriority::Medium => 1,
            TxPriority::High => 2,
        };
        let priority_fee = fee_history.recommended_priority_fee(priority_index);

        Ok(GasPrice::eip1559(max_fee, priority_fee))
    }

    /// Calculates legacy gas price based on priority.
    async fn calculate_legacy_price(&self, priority: TxPriority) -> BlockchainResult<GasPrice> {
        let gas_price = self
            .provider
            .get_gas_price()
            .await
            .map_err(|e| BlockchainError::connection(e.to_string()))?;

        let multiplier = match priority {
            TxPriority::Low => 90,
            TxPriority::Medium => 100,
            TxPriority::High => 120,
        };

        let adjusted = gas_price.as_u64() * multiplier / 100;
        Ok(GasPrice::legacy(adjusted))
    }
}

#[async_trait]
impl BlockchainClient for EthereumClient {
    fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    async fn get_block_number(&self) -> BlockchainResult<u64> {
        self.provider
            .get_block_number()
            .await
            .map(|n| n.as_u64())
            .map_err(|e| BlockchainError::connection(e.to_string()))
    }

    async fn get_balance(&self, address: &str) -> BlockchainResult<u128> {
        let addr: Address = address
            .parse()
            .map_err(|_| BlockchainError::internal(format!("invalid address: {}", address)))?;

        self.provider
            .get_balance(addr, None)
            .await
            .map(|b| b.as_u128())
            .map_err(|e| BlockchainError::connection(e.to_string()))
    }

    async fn estimate_gas(&self, to: &str, data: &[u8], value: u128) -> BlockchainResult<u64> {
        let to_addr: Address = to
            .parse()
            .map_err(|_| BlockchainError::internal(format!("invalid address: {}", to)))?;

        let tx = TransactionRequest::new()
            .to(to_addr)
            .data(data.to_vec())
            .value(U256::from(value));

        let estimate = self
            .provider
            .estimate_gas(&tx.into(), None)
            .await
            .map_err(|e| BlockchainError::gas_estimation(e.to_string()))?;

        Ok(self.gas_estimator.apply_buffer(estimate.as_u64()))
    }

    async fn get_gas_price(&self, priority: TxPriority) -> BlockchainResult<GasPrice> {
        if self.chain_id.supports_eip1559() {
            self.calculate_eip1559_price(priority).await
        } else {
            self.calculate_legacy_price(priority).await
        }
    }

    async fn send_transaction(
        &self,
        to: &str,
        data: &[u8],
        value: u128,
        gas_limit: u64,
        gas_price: GasPrice,
    ) -> BlockchainResult<TxHash> {
        let to_addr: Address = to
            .parse()
            .map_err(|_| BlockchainError::internal(format!("invalid address: {}", to)))?;

        let _tx = TransactionRequest::new()
            .to(to_addr)
            .data(data.to_vec())
            .value(U256::from(value))
            .gas(U256::from(gas_limit))
            .gas_price(U256::from(gas_price.effective_price()));

        // Note: This is a simplified implementation.
        // A full implementation would use a wallet/signer for signing.
        // For now, we just return a placeholder error.
        Err(BlockchainError::transaction(
            "transaction signing not implemented - requires wallet configuration".to_string(),
        ))
    }

    async fn wait_for_confirmation(
        &self,
        tx_hash: &TxHash,
        confirmations: u64,
    ) -> BlockchainResult<TxReceipt> {
        let hash: H256 = tx_hash
            .as_str()
            .parse()
            .map_err(|_| BlockchainError::internal("invalid transaction hash".to_string()))?;

        // Wait for the transaction to be mined
        let receipt = self
            .provider
            .get_transaction_receipt(hash)
            .await
            .map_err(|e| BlockchainError::connection(e.to_string()))?
            .ok_or_else(|| BlockchainError::timeout("transaction not found".to_string()))?;

        // Check confirmations
        if confirmations > 0 {
            let current_block = self.get_block_number().await?;
            let tx_block = receipt
                .block_number
                .ok_or_else(|| BlockchainError::internal("no block number in receipt".to_string()))?
                .as_u64();

            if current_block < tx_block + confirmations {
                return Err(BlockchainError::timeout(format!(
                    "waiting for {} confirmations, have {}",
                    confirmations,
                    current_block.saturating_sub(tx_block)
                )));
            }
        }

        Ok(TxReceipt {
            tx_hash: tx_hash.clone(),
            block_number: receipt.block_number.map(|n| n.as_u64()).unwrap_or_default(),
            gas_used: receipt.gas_used.map(|g| g.as_u64()).unwrap_or_default(),
            effective_gas_price: receipt
                .effective_gas_price
                .map(|p| p.as_u64())
                .unwrap_or_default(),
            success: receipt.status.map(|s| s.as_u64() == 1).unwrap_or(false),
        })
    }

    async fn get_nonce(&self, address: &str) -> BlockchainResult<u64> {
        let addr: Address = address
            .parse()
            .map_err(|_| BlockchainError::internal(format!("invalid address: {}", address)))?;

        self.provider
            .get_transaction_count(addr, None)
            .await
            .map(|n| n.as_u64())
            .map_err(|e| BlockchainError::connection(e.to_string()))
    }

    async fn health_check(&self) -> BlockchainResult<()> {
        let chain_id = self
            .provider
            .get_chainid()
            .await
            .map_err(|e| BlockchainError::connection(e.to_string()))?;

        if chain_id.as_u64() != self.chain_id.as_u64() {
            return Err(BlockchainError::internal(format!(
                "chain ID mismatch: expected {}, got {}",
                self.chain_id.as_u64(),
                chain_id.as_u64()
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ethereum_client_gas_estimator() {
        // Note: This test doesn't make RPC calls
        let estimator = GasEstimator::new(25);
        assert_eq!(estimator.apply_buffer(100_000), 125_000);
    }

    #[test]
    fn chain_id_supports_eip1559() {
        assert!(ChainId::Ethereum.supports_eip1559());
        assert!(!ChainId::Arbitrum.supports_eip1559());
    }
}
