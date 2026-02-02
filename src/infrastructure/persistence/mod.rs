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
//!
//! ## Implementations
//!
//! - `in_memory`: In-memory implementations for testing
//! - `postgres`: PostgreSQL implementations (TODO)
//! - `event_store`: Event sourcing support (TODO)

pub mod event_store;
pub mod in_memory;
pub mod postgres;
pub mod traits;

pub use traits::{
    CounterpartyRepository, RepositoryError, RepositoryResult, RfqRepository, TradeRepository,
    VenueRepository,
};
