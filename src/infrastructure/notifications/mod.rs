//! # Notification Infrastructure
//!
//! Channel adapters for multi-channel trade confirmation delivery.

pub mod websocket_confirmation;
pub mod email_confirmation;
pub mod api_callback_confirmation;
pub mod grpc_confirmation;

pub use websocket_confirmation::WebSocketConfirmationAdapter;
pub use email_confirmation::EmailConfirmationAdapter;
pub use api_callback_confirmation::ApiCallbackConfirmationAdapter;
pub use grpc_confirmation::GrpcConfirmationAdapter;
