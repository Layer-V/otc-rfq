//! # gRPC Services
//!
//! gRPC service implementations using tonic.
//!
//! # Modules
//!
//! - [`proto`]: Generated protobuf types and gRPC service definitions
//! - [`conversions`]: Conversions between domain types and protobuf messages
//! - [`service`]: gRPC service implementation
//!
//! # Usage
//!
//! ```ignore
//! use otc_rfq::api::grpc::{RfqServiceImpl, proto::rfq_service_server::RfqServiceServer};
//! use tonic::transport::Server;
//!
//! let service = RfqServiceImpl::new(rfq_repository);
//! Server::builder()
//!     .add_service(RfqServiceServer::new(service))
//!     .serve("[::1]:50051".parse()?)
//!     .await?;
//! ```

pub mod conversions;
pub mod proto;
pub mod service;

pub use conversions::ConversionError;
pub use proto::otc_rfq_v1;
pub use service::RfqServiceImpl;
