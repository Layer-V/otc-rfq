//! # REST API
//!
//! REST endpoints using axum for management operations.
//!
//! This module provides a complete REST API for the OTC RFQ system,
//! including endpoints for RFQ management, venue configuration, trade queries,
//! and market maker performance metrics.
//!
//! # Endpoints
//!
//! ## RFQs
//! - `GET /api/v1/rfqs` - List RFQs with filtering and pagination
//! - `GET /api/v1/rfqs/{id}` - Get RFQ by ID
//! - `POST /api/v1/rfqs` - Create new RFQ
//! - `DELETE /api/v1/rfqs/{id}` - Cancel RFQ
//!
//! ## Venues
//! - `GET /api/v1/venues` - List all venues
//! - `PUT /api/v1/venues/{id}` - Update venue configuration
//!
//! ## Trades
//! - `GET /api/v1/trades` - List trades with filtering and pagination
//! - `GET /api/v1/trades/{id}` - Get trade by ID
//!
//! ## MM Performance (admin)
//! - `GET /api/v1/mm-performance` - List all MM performance metrics
//! - `GET /api/v1/mm-performance/{mm_id}` - Get specific MM performance
//!
//! ## Health
//! - `GET /api/v1/health` - Health check endpoint
//!
//! # Usage
//!
//! ```ignore
//! use otc_rfq::api::rest::{create_router, AppState};
//! use std::sync::Arc;
//!
//! let state = Arc::new(AppState {
//!     rfq_repository: /* ... */,
//!     venue_repository: /* ... */,
//!     trade_repository: /* ... */,
//!     mm_performance_tracker: None,
//! });
//!
//! let router = create_router(state);
//!
//! let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
//! axum::serve(listener, router).await?;
//! ```

pub mod handlers;
pub mod routes;

pub use handlers::{
    AppState, CreateRfqRequest, ErrorResponse, HealthResponse, MmPerformanceFilter,
    MmPerformanceResponse, PaginatedResponse, PaginationMeta, PaginationParams, RfqFilter,
    RfqResponse, TradeFilter, TradeRepository, TradeResponse, UpdateVenueRequest, VenueRepository,
    VenueResponse,
};
pub use routes::create_router;
