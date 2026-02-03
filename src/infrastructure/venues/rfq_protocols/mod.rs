//! # RFQ Protocol Adapters
//!
//! Adapters for RFQ-native protocols like Hashflow, Bebop, and Airswap.
//!
//! ## Available Adapters
//!
//! - [`HashflowAdapter`]: Hashflow RFQ protocol with gasless trading
//! - [`BebopAdapter`]: Bebop RFQ protocol with batch swap support
//! - [`AirswapAdapter`]: Airswap P2P RFQ protocol with EIP-712 orders
//!
//! ## Features
//!
//! - Gasless trading with MEV protection
//! - Signed quotes with EIP-712 signatures
//! - Quote expiry tracking
//! - Multi-chain support
//! - Batch swap support (Bebop)
//! - Peer-to-peer trading (Airswap)

pub mod airswap;
pub mod bebop;
pub mod hashflow;

pub use airswap::{
    AirswapAdapter, AirswapChain, AirswapConfig, AirswapOrder, AirswapRfqRequest,
    AirswapRfqResponse, SignedAirswapOrder,
};
pub use bebop::{
    BebopAdapter, BebopBatchQuoteRequest, BebopChain, BebopConfig, BebopQuoteData,
    BebopQuoteRequest, BebopQuoteResponse,
};
pub use hashflow::{
    HashflowAdapter, HashflowChain, HashflowConfig, HashflowQuoteData, HashflowRfqRequest,
    HashflowRfqResponse,
};
