//! # NATS Publisher Worker
//!
//! Background task responsible for reliably publishing events
//! to NATS JetStream.

use async_nats::jetstream;
use async_nats::jetstream::stream::Config;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::infrastructure::messaging::dispatcher::PublishPayload;

/// Background worker for publishing events to NATS JetStream.
pub struct NatsPublisherWorker {
    client: async_nats::Client,
    jetstream: jetstream::Context,
    receiver: mpsc::Receiver<PublishPayload>,
    stream_name: String,
    subject_prefix: String,
}

impl NatsPublisherWorker {
    /// Connects to NATS and initializes the JetStream context and stream.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection to NATS fails, or if JetStream stream creation fails.
    pub async fn connect(
        nats_url: &str,
        stream_name: &str,
        subject_prefix: &str,
        receiver: mpsc::Receiver<PublishPayload>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let client = async_nats::connect(nats_url).await?;
        let jetstream = jetstream::new(client.clone());

        // Ensure the stream exists
        let subjects = vec![format!("{}.>", subject_prefix)];

        match jetstream.get_stream(stream_name).await {
            Ok(_stream) => {
                info!("Found existing JetStream stream: {}", stream_name);
                // Optionally update the stream config to ensure subjects cover our prefix
                // For simplicity, we assume it's created properly or we just use it.
            }
            Err(_) => {
                info!(
                    "Creating JetStream stream: {} for subjects {:?}",
                    stream_name, subjects
                );
                jetstream
                    .create_stream(Config {
                        name: stream_name.to_string(),
                        subjects,
                        ..Default::default()
                    })
                    .await?;
            }
        }

        Ok(Self {
            client,
            jetstream,
            receiver,
            stream_name: stream_name.to_string(),
            subject_prefix: subject_prefix.to_string(),
        })
    }

    /// Health check for the NATS connection.
    ///
    /// # Errors
    ///
    /// Returns an error if the NATS client is disconnected.
    pub fn health_check(&self) -> Result<(), String> {
        use async_nats::connection::State;
        if self.client.connection_state() == State::Connected {
            Ok(())
        } else {
            Err("NATS client is disconnected".to_string())
        }
    }

    /// Starts the worker loop to consume events and publish to NATS.
    pub async fn run(mut self) {
        info!(
            "Starting NatsPublisherWorker for stream {} (prefix: {})",
            self.stream_name, self.subject_prefix
        );

        while let Some((subject, payload)) = self.receiver.recv().await {
            self.publish_with_retry(&subject, &payload).await;
        }

        info!("NatsPublisherWorker shutting down (channel closed)");
    }

    /// Publishes an event to JetStream with exponential backoff retry.
    async fn publish_with_retry(&self, subject: &str, payload: &str) {
        let max_retries = 5;
        let mut retry_count = 0;
        let mut delay = Duration::from_millis(100);
        let payload_bytes = bytes::Bytes::from(payload.to_string());

        loop {
            match self
                .jetstream
                .publish(subject.to_string(), payload_bytes.clone())
                .await
            {
                Ok(ack) => match ack.await {
                    Ok(publish_ack) => {
                        debug!(
                            "Successfully published event to {} (seq: {})",
                            subject, publish_ack.sequence
                        );
                        return;
                    }
                    Err(e) => {
                        warn!("JetStream ack error for {}: {}", subject, e);
                    }
                },
                Err(e) => {
                    warn!("Failed to publish to {}: {}", subject, e);
                }
            }

            retry_count += 1;
            if retry_count > max_retries {
                error!(
                    "Exceeded max retries ({}) publishing event to {}. Dropping event.",
                    max_retries, subject
                );
                // In a production system, here we would push the payload to a local DLQ file
                return;
            }

            warn!(
                "Retrying publish to {} in {}ms (attempt {}/{})",
                subject,
                delay.as_millis(),
                retry_count,
                max_retries
            );
            tokio::time::sleep(delay).await;
            delay *= 2; // Exponential backoff
        }
    }
}
