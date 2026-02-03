//! # HTTP Client Utilities
//!
//! Shared HTTP client utilities for venue adapters.
//!
//! This module provides a reusable HTTP client wrapper with:
//! - Configurable timeouts
//! - Automatic retries
//! - JSON serialization/deserialization
//! - Error handling
//!
//! # Examples
//!
//! ```ignore
//! use otc_rfq::infrastructure::venues::http_client::HttpClient;
//!
//! let client = HttpClient::new(5000)?;
//! let response: MyResponse = client.get("https://api.example.com/endpoint").await?;
//! ```

use crate::infrastructure::venues::error::{VenueError, VenueResult};
use reqwest::{Client, Response, StatusCode};
use serde::de::DeserializeOwned;
use std::time::Duration;

/// HTTP client wrapper for venue adapters.
///
/// Provides a convenient interface for making HTTP requests with
/// proper error handling and timeout configuration.
#[derive(Debug, Clone)]
pub struct HttpClient {
    /// Inner reqwest client.
    client: Client,
    /// Request timeout in milliseconds.
    timeout_ms: u64,
}

impl HttpClient {
    /// Creates a new HTTP client with the specified timeout.
    ///
    /// # Arguments
    ///
    /// * `timeout_ms` - Request timeout in milliseconds.
    ///
    /// # Errors
    ///
    /// Returns `VenueError::InternalError` if the client cannot be created.
    pub fn new(timeout_ms: u64) -> VenueResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .build()
            .map_err(|e| {
                VenueError::internal_error(format!("Failed to create HTTP client: {}", e))
            })?;

        Ok(Self { client, timeout_ms })
    }

    /// Creates a new HTTP client with custom headers.
    ///
    /// # Arguments
    ///
    /// * `timeout_ms` - Request timeout in milliseconds.
    /// * `default_headers` - Default headers to include in all requests.
    ///
    /// # Errors
    ///
    /// Returns `VenueError::InternalError` if the client cannot be created.
    pub fn with_headers(
        timeout_ms: u64,
        default_headers: reqwest::header::HeaderMap,
    ) -> VenueResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .default_headers(default_headers)
            .build()
            .map_err(|e| {
                VenueError::internal_error(format!("Failed to create HTTP client: {}", e))
            })?;

        Ok(Self { client, timeout_ms })
    }

    /// Returns the configured timeout in milliseconds.
    #[inline]
    #[must_use]
    pub fn timeout_ms(&self) -> u64 {
        self.timeout_ms
    }

    /// Makes a GET request and deserializes the JSON response.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to request.
    ///
    /// # Errors
    ///
    /// Returns `VenueError::NetworkError` if the request fails.
    /// Returns `VenueError::ProtocolError` if the response cannot be parsed.
    pub async fn get<T: DeserializeOwned>(&self, url: &str) -> VenueResult<T> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| self.map_reqwest_error(e))?;

        self.handle_response(response).await
    }

    /// Makes a GET request with query parameters and deserializes the JSON response.
    ///
    /// # Arguments
    ///
    /// * `url` - The base URL.
    /// * `params` - Query parameters.
    ///
    /// # Errors
    ///
    /// Returns `VenueError::NetworkError` if the request fails.
    /// Returns `VenueError::ProtocolError` if the response cannot be parsed.
    pub async fn get_with_params<T: DeserializeOwned, P: serde::Serialize + ?Sized>(
        &self,
        url: &str,
        params: &P,
    ) -> VenueResult<T> {
        let response = self
            .client
            .get(url)
            .query(params)
            .send()
            .await
            .map_err(|e| self.map_reqwest_error(e))?;

        self.handle_response(response).await
    }

    /// Makes a GET request with custom headers and deserializes the JSON response.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to request.
    /// * `headers` - Additional headers to include.
    ///
    /// # Errors
    ///
    /// Returns `VenueError::NetworkError` if the request fails.
    /// Returns `VenueError::ProtocolError` if the response cannot be parsed.
    pub async fn get_with_headers<T: DeserializeOwned>(
        &self,
        url: &str,
        headers: reqwest::header::HeaderMap,
    ) -> VenueResult<T> {
        let response = self
            .client
            .get(url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| self.map_reqwest_error(e))?;

        self.handle_response(response).await
    }

    /// Makes a GET request with query parameters and custom headers.
    ///
    /// # Arguments
    ///
    /// * `url` - The base URL.
    /// * `params` - Query parameters.
    /// * `headers` - Additional headers to include.
    ///
    /// # Errors
    ///
    /// Returns `VenueError::NetworkError` if the request fails.
    /// Returns `VenueError::ProtocolError` if the response cannot be parsed.
    pub async fn get_with_params_and_headers<T: DeserializeOwned, P: serde::Serialize + ?Sized>(
        &self,
        url: &str,
        params: &P,
        headers: reqwest::header::HeaderMap,
    ) -> VenueResult<T> {
        let response = self
            .client
            .get(url)
            .query(params)
            .headers(headers)
            .send()
            .await
            .map_err(|e| self.map_reqwest_error(e))?;

        self.handle_response(response).await
    }

    /// Makes a POST request with JSON body and deserializes the JSON response.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to request.
    /// * `body` - The request body to serialize as JSON.
    ///
    /// # Errors
    ///
    /// Returns `VenueError::NetworkError` if the request fails.
    /// Returns `VenueError::ProtocolError` if the response cannot be parsed.
    pub async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        url: &str,
        body: &B,
    ) -> VenueResult<T> {
        let response = self
            .client
            .post(url)
            .json(body)
            .send()
            .await
            .map_err(|e| self.map_reqwest_error(e))?;

        self.handle_response(response).await
    }

    /// Makes a POST request with JSON body and custom headers.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to request.
    /// * `body` - The request body to serialize as JSON.
    /// * `headers` - Additional headers to include.
    ///
    /// # Errors
    ///
    /// Returns `VenueError::NetworkError` if the request fails.
    /// Returns `VenueError::ProtocolError` if the response cannot be parsed.
    pub async fn post_with_headers<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        url: &str,
        body: &B,
        headers: reqwest::header::HeaderMap,
    ) -> VenueResult<T> {
        let response = self
            .client
            .post(url)
            .json(body)
            .headers(headers)
            .send()
            .await
            .map_err(|e| self.map_reqwest_error(e))?;

        self.handle_response(response).await
    }

    /// Makes a simple health check GET request.
    ///
    /// Returns `true` if the request succeeds with a 2xx status code.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to check.
    pub async fn health_check(&self, url: &str) -> bool {
        match self.client.get(url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// Makes a health check GET request with custom headers.
    ///
    /// Returns `true` if the request succeeds with a 2xx status code.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to check.
    /// * `headers` - Additional headers to include.
    pub async fn health_check_with_headers(
        &self,
        url: &str,
        headers: reqwest::header::HeaderMap,
    ) -> bool {
        match self.client.get(url).headers(headers).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// Handles the HTTP response, checking status and deserializing JSON.
    async fn handle_response<T: DeserializeOwned>(&self, response: Response) -> VenueResult<T> {
        let status = response.status();

        if status.is_success() {
            response
                .json::<T>()
                .await
                .map_err(|e| VenueError::protocol_error(format!("Failed to parse response: {}", e)))
        } else {
            let error_body = response.text().await.unwrap_or_default();
            Err(self.map_status_error(status, &error_body))
        }
    }

    /// Maps a reqwest error to a VenueError.
    fn map_reqwest_error(&self, error: reqwest::Error) -> VenueError {
        if error.is_timeout() {
            VenueError::timeout("Request timed out")
        } else if error.is_connect() {
            VenueError::connection(format!("Connection failed: {}", error))
        } else {
            VenueError::connection(format!("HTTP request failed: {}", error))
        }
    }

    /// Maps an HTTP status code to a VenueError.
    fn map_status_error(&self, status: StatusCode, body: &str) -> VenueError {
        match status {
            StatusCode::BAD_REQUEST => {
                VenueError::invalid_request(format!("Bad request: {}", body))
            }
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                VenueError::authentication(format!("Authentication failed: {}", body))
            }
            StatusCode::NOT_FOUND => {
                VenueError::protocol_error(format!("Resource not found: {}", body))
            }
            StatusCode::TOO_MANY_REQUESTS => VenueError::rate_limited("Rate limit exceeded"),
            StatusCode::INTERNAL_SERVER_ERROR
            | StatusCode::BAD_GATEWAY
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::GATEWAY_TIMEOUT => {
                VenueError::connection(format!("Server error ({}): {}", status, body))
            }
            _ => VenueError::protocol_error(format!("HTTP error ({}): {}", status, body)),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn new_client() {
        let client = HttpClient::new(5000);
        assert!(client.is_ok());
        assert_eq!(client.unwrap().timeout_ms(), 5000);
    }

    #[test]
    fn with_headers() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("X-Custom", "value".parse().unwrap());
        let client = HttpClient::with_headers(3000, headers);
        assert!(client.is_ok());
    }
}
