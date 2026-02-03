//! # RFQ Protocol Adapters
//!
//! Adapters for RFQ-native protocols like Hashflow and Bebop.
//!
//! ## Available Adapters
//!
//! - [`HashflowAdapter`]: Hashflow RFQ protocol with gasless trading
//! - [`BebopAdapter`]: Bebop RFQ protocol with batch swap support
//!
//! ## Features
//!
//! - Gasless trading with MEV protection
//! - Signed quotes with EIP-712 signatures
//! - Quote expiry tracking
//! - Multi-chain support
//! - Batch swap support (Bebop)

pub mod bebop;
pub mod hashflow;

pub use bebop::{
    BebopAdapter, BebopBatchQuoteRequest, BebopChain, BebopConfig, BebopQuoteData,
    BebopQuoteRequest, BebopQuoteResponse,
};
pub use hashflow::{
    HashflowAdapter, HashflowChain, HashflowConfig, HashflowQuoteData, HashflowRfqRequest,
    HashflowRfqResponse,
};
