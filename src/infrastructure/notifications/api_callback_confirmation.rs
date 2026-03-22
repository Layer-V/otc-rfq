//! # API Callback Confirmation Adapter
//!
//! Delivers trade confirmations via HTTP POST to webhook URLs.

use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::services::confirmation_service::ConfirmationChannelAdapter;
use crate::domain::value_objects::confirmation::{ConfirmationChannel, TradeConfirmation};
use async_trait::async_trait;
use reqwest;
use serde_json;
use std::time::Duration;

/// API callback confirmation adapter.
#[derive(Debug)]
pub struct ApiCallbackConfirmationAdapter {
    webhook_url: String,
    client: reqwest::Client,
}

impl ApiCallbackConfirmationAdapter {
    /// Creates a new API callback confirmation adapter.
    ///
    /// # Arguments
    ///
    /// * `webhook_url` - The webhook URL to POST confirmations to
    /// * `timeout` - HTTP request timeout
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new(webhook_url: String, timeout: Duration) -> DomainResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| DomainError::ConfirmationFailed {
                channel: "API_CALLBACK".to_string(),
                reason: format!("Failed to create HTTP client: {}", e),
            })?;

        Ok(Self {
            webhook_url,
            client,
        })
    }

    /// Creates a new adapter with default timeout (10 seconds).
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn with_default_timeout(webhook_url: String) -> DomainResult<Self> {
        Self::new(webhook_url, Duration::from_secs(10))
    }
}

#[async_trait]
impl ConfirmationChannelAdapter for ApiCallbackConfirmationAdapter {
    async fn send(&self, confirmation: &TradeConfirmation) -> DomainResult<()> {
        // Serialize confirmation to JSON
        let json_body =
            serde_json::to_string(confirmation).map_err(|e| DomainError::ConfirmationFailed {
                channel: "API_CALLBACK".to_string(),
                reason: format!("JSON serialization failed: {}", e),
            })?;

        // Send POST request
        let response = self
            .client
            .post(&self.webhook_url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "OTC-RFQ-Platform/1.0")
            .body(json_body)
            .send()
            .await
            .map_err(|e| DomainError::ConfirmationFailed {
                channel: "API_CALLBACK".to_string(),
                reason: format!("HTTP request failed: {}", e),
            })?;

        // Check response status
        let status = response.status();
        if status.is_success() {
            tracing::info!(
                trade_id = %confirmation.trade_id(),
                webhook_url = %self.webhook_url,
                status = %status,
                "API callback confirmation sent successfully"
            );
            Ok(())
        } else if status.is_server_error() {
            // 5xx errors should be retried by the confirmation service
            let reason = match status.canonical_reason() {
                Some(reason) => format!("Server error {}: {}", status.as_u16(), reason),
                None => format!("Server error {}", status.as_u16()),
            };
            Err(DomainError::ConfirmationFailed {
                channel: "API_CALLBACK".to_string(),
                reason,
            })
        } else {
            // 4xx errors should not be retried (client errors are permanent)
            let reason = match status.canonical_reason() {
                Some(reason) => format!("Client error {}: {}", status.as_u16(), reason),
                None => format!("Client error {}", status.as_u16()),
            };
            Err(DomainError::ConfirmationFailed {
                channel: "API_CALLBACK".to_string(),
                reason,
            })
        }
    }

    fn channel(&self) -> ConfirmationChannel {
        ConfirmationChannel::ApiCallback
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{
        Blockchain, CounterpartyId, Price, Quantity, RfqId, SettlementMethod, TradeId,
    };
    use rust_decimal::Decimal;

    fn create_test_confirmation() -> TradeConfirmation {
        TradeConfirmation::new(
            TradeId::new_v4(),
            RfqId::new_v4(),
            Price::new(50000.0).unwrap(),
            Quantity::new(1.0).unwrap(),
            Decimal::new(10, 0),
            Decimal::new(5, 0),
            Decimal::new(15, 0),
            SettlementMethod::OnChain(Blockchain::Ethereum),
            CounterpartyId::new("buyer-1"),
            CounterpartyId::new("seller-1"),
        )
    }

    #[test]
    fn new_creates_adapter() {
        let result = ApiCallbackConfirmationAdapter::new(
            "https://example.com/webhook".to_string(),
            Duration::from_secs(5),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn with_default_timeout_creates_adapter() {
        let result = ApiCallbackConfirmationAdapter::with_default_timeout(
            "https://example.com/webhook".to_string(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn channel_returns_api_callback() {
        let adapter = ApiCallbackConfirmationAdapter::with_default_timeout(
            "https://example.com/webhook".to_string(),
        )
        .unwrap();
        assert_eq!(adapter.channel(), ConfirmationChannel::ApiCallback);
    }

    #[tokio::test]
    async fn send_invalid_url_fails() {
        let adapter =
            ApiCallbackConfirmationAdapter::with_default_timeout("not-a-valid-url".to_string())
                .unwrap();
        let confirmation = create_test_confirmation();

        let result = adapter.send(&confirmation).await;
        assert!(result.is_err());
    }

    // Note: Integration tests with actual HTTP server would go in tests/ directory
}
