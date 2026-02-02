//! # Persistence Layer
//!
//! Repository implementations and event store.
//!
//! ## Repository Traits (Ports)
//!
//! - [`RfqRepository`]: Persistence for RFQ entities
//! - [`TradeRepository`]: Persistence for Trade entities
//! - [`VenueRepository`]: Persistence for venue configurations
//! - [`CounterpartyRepository`]: Persistence for counterparty data
//! - [`EventStore`]: Append-only event storage
//!
//! ## Implementations
//!
//! - `in_memory`: In-memory implementations for testing
//! - `postgres`: PostgreSQL implementations with sqlx

pub mod event_store;
pub mod in_memory;
pub mod postgres;
pub mod traits;

pub use event_store::{EventStore, EventStoreError, EventStoreResult, StoredEvent};
pub use traits::{
    CounterpartyRepository, RepositoryError, RepositoryResult, RfqRepository, TradeRepository,
    VenueRepository,
};
