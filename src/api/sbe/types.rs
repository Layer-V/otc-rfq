//! # SBE API Types
//!
//! Request and response types for SBE client API.
//!
//! These types represent the wire format for SBE messages and provide
//! encoding/decoding implementations using the infrastructure SBE helpers.

use crate::domain::value_objects::{
    Instrument, OrderSide, Price, Quantity, RfqId, QuoteId, TradeId, VenueId, RfqState,
};
use crate::domain::value_objects::enums::{AssetClass, VenueType};
use crate::domain::value_objects::timestamp::Timestamp;
use crate::infrastructure::sbe::{SbeEncode, SbeDecode, SbeError};
use crate::infrastructure::sbe::error::SbeResult;
use crate::infrastructure::sbe::types::{SbeUuid, SbeDecimal, encode_var_string, decode_var_string};
use uuid::Uuid;

/// Message header size in bytes.
pub const MESSAGE_HEADER_SIZE: usize = 8;

/// Schema ID for OTC RFQ SBE messages.
pub const SCHEMA_ID: u16 = 1;

/// Schema version.
pub const SCHEMA_VERSION: u16 = 1;

// ============================================================================
// Template ID Constants
// ============================================================================

/// Template ID for CreateRfqRequest.
pub const CREATE_RFQ_REQUEST_TEMPLATE_ID: u16 = 20;

/// Template ID for CreateRfqResponse.
pub const CREATE_RFQ_RESPONSE_TEMPLATE_ID: u16 = 21;

/// Template ID for GetRfqRequest.
pub const GET_RFQ_REQUEST_TEMPLATE_ID: u16 = 22;

/// Template ID for GetRfqResponse.
pub const GET_RFQ_RESPONSE_TEMPLATE_ID: u16 = 23;

/// Template ID for CancelRfqRequest.
pub const CANCEL_RFQ_REQUEST_TEMPLATE_ID: u16 = 24;

/// Template ID for CancelRfqResponse.
pub const CANCEL_RFQ_RESPONSE_TEMPLATE_ID: u16 = 25;

/// Template ID for ExecuteTradeRequest.
pub const EXECUTE_TRADE_REQUEST_TEMPLATE_ID: u16 = 26;

/// Template ID for ExecuteTradeResponse.
pub const EXECUTE_TRADE_RESPONSE_TEMPLATE_ID: u16 = 27;

/// Template ID for SubscribeQuotesRequest.
pub const SUBSCRIBE_QUOTES_REQUEST_TEMPLATE_ID: u16 = 30;

/// Template ID for QuoteUpdate.
pub const QUOTE_UPDATE_TEMPLATE_ID: u16 = 31;

/// Template ID for SubscribeRfqStatusRequest.
pub const SUBSCRIBE_RFQ_STATUS_REQUEST_TEMPLATE_ID: u16 = 32;

/// Template ID for RfqStatusUpdate.
pub const RFQ_STATUS_UPDATE_TEMPLATE_ID: u16 = 33;

/// Template ID for UnsubscribeRequest.
pub const UNSUBSCRIBE_REQUEST_TEMPLATE_ID: u16 = 40;

/// Template ID for ErrorResponse.
pub const ERROR_RESPONSE_TEMPLATE_ID: u16 = 50;

// ============================================================================
// Helper Functions
// ============================================================================

/// Encodes the SBE message header.
#[inline]
fn encode_header(buffer: &mut [u8], block_length: u16, template_id: u16) -> SbeResult<()> {
    if buffer.len() < MESSAGE_HEADER_SIZE {
        return Err(SbeError::BufferTooSmall {
            needed: MESSAGE_HEADER_SIZE,
            available: buffer.len(),
        });
    }
    buffer[0..2].copy_from_slice(&block_length.to_le_bytes());
    buffer[2..4].copy_from_slice(&template_id.to_le_bytes());
    buffer[4..6].copy_from_slice(&SCHEMA_ID.to_le_bytes());
    buffer[6..8].copy_from_slice(&SCHEMA_VERSION.to_le_bytes());
    Ok(())
}

/// Decodes the SBE message header.
#[inline]
fn decode_header(buffer: &[u8]) -> SbeResult<(u16, u16, u16, u16)> {
    if buffer.len() < MESSAGE_HEADER_SIZE {
        return Err(SbeError::BufferTooSmall {
            needed: MESSAGE_HEADER_SIZE,
            available: buffer.len(),
        });
    }
    let block_length = u16::from_le_bytes([buffer[0], buffer[1]]);
    let template_id = u16::from_le_bytes([buffer[2], buffer[3]]);
    let schema_id = u16::from_le_bytes([buffer[4], buffer[5]]);
    let version = u16::from_le_bytes([buffer[6], buffer[7]]);
    Ok((block_length, template_id, schema_id, version))
}

/// Encodes OrderSide to SBE enum value.
#[inline]
#[must_use]
fn encode_order_side(side: OrderSide) -> u8 {
    match side {
        OrderSide::Buy => 0,
        OrderSide::Sell => 1,
    }
}

/// Decodes OrderSide from SBE enum value.
#[inline]
fn decode_order_side(value: u8) -> SbeResult<OrderSide> {
    match value {
        0 => Ok(OrderSide::Buy),
        1 => Ok(OrderSide::Sell),
        _ => Err(SbeError::InvalidEnumValue(value)),
    }
}

/// Encodes AssetClass to SBE enum value.
#[inline]
#[must_use]
fn encode_asset_class(class: AssetClass) -> u8 {
    match class {
        AssetClass::CryptoSpot => 0,
        AssetClass::CryptoDerivs => 1,
        AssetClass::Stock => 2,
        AssetClass::Forex => 3,
        AssetClass::Commodity => 4,
    }
}

/// Decodes AssetClass from SBE enum value.
#[inline]
fn decode_asset_class(value: u8) -> SbeResult<AssetClass> {
    match value {
        0 => Ok(AssetClass::CryptoSpot),
        1 => Ok(AssetClass::CryptoDerivs),
        2 => Ok(AssetClass::Stock),
        3 => Ok(AssetClass::Forex),
        4 => Ok(AssetClass::Commodity),
        _ => Err(SbeError::InvalidEnumValue(value)),
    }
}

/// Encodes RfqState to SBE enum value.
#[inline]
#[must_use]
fn encode_rfq_state(state: RfqState) -> u8 {
    match state {
        RfqState::Created => 0,
        RfqState::QuoteRequesting => 1,
        RfqState::QuotesReceived => 2,
        RfqState::ClientSelecting => 3,
        RfqState::Executing => 4,
        RfqState::Executed => 5,
        RfqState::Failed => 6,
        RfqState::Cancelled => 7,
        RfqState::Expired => 8,
        RfqState::Negotiating => 9,
    }
}

/// Decodes RfqState from SBE enum value.
#[inline]
fn decode_rfq_state(value: u8) -> SbeResult<RfqState> {
    match value {
        0 => Ok(RfqState::Created),
        1 => Ok(RfqState::QuoteRequesting),
        2 => Ok(RfqState::QuotesReceived),
        3 => Ok(RfqState::ClientSelecting),
        4 => Ok(RfqState::Executing),
        5 => Ok(RfqState::Executed),
        6 => Ok(RfqState::Failed),
        7 => Ok(RfqState::Cancelled),
        8 => Ok(RfqState::Expired),
        9 => Ok(RfqState::Negotiating),
        _ => Err(SbeError::InvalidEnumValue(value)),
    }
}

/// Encodes VenueType to SBE enum value.
#[inline]
#[must_use]
fn encode_venue_type(vtype: VenueType) -> u8 {
    match vtype {
        VenueType::InternalMM => 0,
        VenueType::ExternalMM => 1,
        VenueType::DexAggregator => 2,
        VenueType::Protocol => 3,
        VenueType::RfqProtocol => 4,
    }
}

/// Decodes VenueType from SBE enum value.
#[inline]
fn decode_venue_type(value: u8) -> SbeResult<VenueType> {
    match value {
        0 => Ok(VenueType::InternalMM),
        1 => Ok(VenueType::ExternalMM),
        2 => Ok(VenueType::DexAggregator),
        3 => Ok(VenueType::Protocol),
        4 => Ok(VenueType::RfqProtocol),
        _ => Err(SbeError::InvalidEnumValue(value)),
    }
}

/// Calculates the size of a variable-length string field.
#[inline]
#[must_use]
fn var_string_size(s: &str) -> usize {
    4 + s.len() // 4-byte length prefix + UTF-8 data
}

// ============================================================================
// Request Types
// ============================================================================

/// CreateRfqRequest - Create a new RFQ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateRfqRequest {
    /// Request correlation ID.
    pub request_id: Uuid,
    /// Client identifier.
    pub client_id: String,
    /// Trading symbol.
    pub symbol: String,
    /// Base asset.
    pub base_asset: String,
    /// Quote asset.
    pub quote_asset: String,
    /// Order side.
    pub side: OrderSide,
    /// Requested quantity.
    pub quantity: Quantity,
    /// RFQ timeout in seconds.
    pub timeout_seconds: i64,
    /// Asset class.
    pub asset_class: AssetClass,
}

impl CreateRfqRequest {
    /// Block length for CreateRfqRequest (fixed fields only).
    pub const BLOCK_LENGTH: u16 = 35;
}

impl SbeEncode for CreateRfqRequest {
    fn encoded_size(&self) -> usize {
        MESSAGE_HEADER_SIZE
            + Self::BLOCK_LENGTH as usize
            + var_string_size(&self.client_id)
            + var_string_size(&self.symbol)
            + var_string_size(&self.base_asset)
            + var_string_size(&self.quote_asset)
    }

    fn encode(&self, buffer: &mut [u8]) -> SbeResult<usize> {
        let size = self.encoded_size();
        if buffer.len() < size {
            return Err(SbeError::BufferTooSmall {
                needed: size,
                available: buffer.len(),
            });
        }

        let mut offset: usize = 0;

        // Header
        encode_header(buffer, Self::BLOCK_LENGTH, CREATE_RFQ_REQUEST_TEMPLATE_ID)?;
        offset = offset.checked_add(MESSAGE_HEADER_SIZE)
            .ok_or_else(|| SbeError::BufferTooSmall { needed: size, available: buffer.len() })?;

        // requestId
        let request_uuid = SbeUuid::from_uuid(self.request_id);
        request_uuid.encode(&mut buffer[offset..])?;
        offset = offset.checked_add(SbeUuid::SIZE)
            .ok_or_else(|| SbeError::BufferTooSmall { needed: size, available: buffer.len() })?;

        // side
        buffer[offset] = encode_order_side(self.side);
        offset = offset.checked_add(1)
            .ok_or_else(|| SbeError::BufferTooSmall { needed: size, available: buffer.len() })?;

        // quantity
        let qty_decimal = SbeDecimal::from_decimal(self.quantity.get());
        qty_decimal.encode(&mut buffer[offset..])?;
        offset = offset.checked_add(SbeDecimal::SIZE)
            .ok_or_else(|| SbeError::BufferTooSmall { needed: size, available: buffer.len() })?;

        // timeoutSeconds
        buffer[offset..offset.checked_add(8).ok_or_else(|| SbeError::Overflow)?]
            .copy_from_slice(&self.timeout_seconds.to_le_bytes());
        offset = offset.checked_add(8)
            .ok_or_else(|| SbeError::BufferTooSmall { needed: size, available: buffer.len() })?;

        // assetClass
        buffer[offset] = encode_asset_class(self.asset_class);
        offset = offset.checked_add(1)
            .ok_or_else(|| SbeError::BufferTooSmall { needed: size, available: buffer.len() })?;

        // Variable fields
        let client_size = encode_var_string(&self.client_id, &mut buffer[offset..])?;
        offset = offset.checked_add(client_size)
            .ok_or_else(|| SbeError::BufferTooSmall { needed: size, available: buffer.len() })?;

        let symbol_size = encode_var_string(&self.symbol, &mut buffer[offset..])?;
        offset = offset.checked_add(symbol_size)
            .ok_or_else(|| SbeError::BufferTooSmall { needed: size, available: buffer.len() })?;

        let base_size = encode_var_string(&self.base_asset, &mut buffer[offset..])?;
        offset = offset.checked_add(base_size)
            .ok_or_else(|| SbeError::BufferTooSmall { needed: size, available: buffer.len() })?;

        let quote_size = encode_var_string(&self.quote_asset, &mut buffer[offset..])?;
        offset = offset.checked_add(quote_size)
            .ok_or_else(|| SbeError::BufferTooSmall { needed: size, available: buffer.len() })?;

        Ok(offset)
    }
}

impl SbeDecode for CreateRfqRequest {
    fn decode(buffer: &[u8]) -> SbeResult<Self> {
        let (_block_length, template_id, _schema_id, _version) = decode_header(buffer)?;

        if template_id != CREATE_RFQ_REQUEST_TEMPLATE_ID {
            return Err(SbeError::UnknownTemplateId(template_id));
        }

        let mut offset: usize = MESSAGE_HEADER_SIZE;

        // requestId
        let request_uuid = SbeUuid::decode(&buffer[offset..])?;
        offset = offset.checked_add(SbeUuid::SIZE)
            .ok_or_else(|| SbeError::Overflow)?;

        // side
        let side = decode_order_side(buffer[offset])?;
        offset = offset.checked_add(1)
            .ok_or_else(|| SbeError::Overflow)?;

        // quantity
        let qty_sbe = SbeDecimal::decode(&buffer[offset..])?;
        let qty_decimal = qty_sbe.to_decimal()?;
        let quantity = Quantity::from_decimal(qty_decimal)
            .map_err(|e| SbeError::InvalidFieldValue(format!("invalid quantity: {}", e)))?;
        offset = offset.checked_add(SbeDecimal::SIZE)
            .ok_or_else(|| SbeError::Overflow)?;

        // timeoutSeconds
        let timeout_bytes = buffer.get(offset..offset.checked_add(8).ok_or_else(|| SbeError::Overflow)?)
            .ok_or_else(|| SbeError::BufferTooSmall { needed: offset + 8, available: buffer.len() })?;
        let timeout_seconds = i64::from_le_bytes(timeout_bytes.try_into()
            .map_err(|_| SbeError::InvalidFieldValue("invalid timeout".to_string()))?);
        offset = offset.checked_add(8)
            .ok_or_else(|| SbeError::Overflow)?;

        // assetClass
        let asset_class = decode_asset_class(buffer[offset])?;
        offset = offset.checked_add(1)
            .ok_or_else(|| SbeError::Overflow)?;

        // Variable fields
        let (client_id, client_size) = decode_var_string(&buffer[offset..])?;
        offset = offset.checked_add(client_size)
            .ok_or_else(|| SbeError::Overflow)?;

        let (symbol, symbol_size) = decode_var_string(&buffer[offset..])?;
        offset = offset.checked_add(symbol_size)
            .ok_or_else(|| SbeError::Overflow)?;

        let (base_asset, base_size) = decode_var_string(&buffer[offset..])?;
        offset = offset.checked_add(base_size)
            .ok_or_else(|| SbeError::Overflow)?;

        let (quote_asset, _quote_size) = decode_var_string(&buffer[offset..])?;

        Ok(Self {
            request_id: request_uuid.to_uuid(),
            client_id,
            symbol,
            base_asset,
            quote_asset,
            side,
            quantity,
            timeout_seconds,
            asset_class,
        })
    }
}

/// GetRfqRequest - Retrieve an RFQ by ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetRfqRequest {
    /// Request correlation ID.
    pub request_id: Uuid,
    /// RFQ identifier to retrieve.
    pub rfq_id: Uuid,
}

impl GetRfqRequest {
    /// Block length for GetRfqRequest (fixed fields only).
    pub const BLOCK_LENGTH: u16 = 32;

    /// Creates a new GetRfqRequest.
    #[must_use]
    pub fn new(request_id: Uuid, rfq_id: Uuid) -> Self {
        Self { request_id, rfq_id }
    }
}

impl SbeEncode for GetRfqRequest {
    fn encoded_size(&self) -> usize {
        MESSAGE_HEADER_SIZE + Self::BLOCK_LENGTH as usize
    }

    fn encode(&self, buffer: &mut [u8]) -> SbeResult<usize> {
        let size = self.encoded_size();
        if buffer.len() < size {
            return Err(SbeError::BufferTooSmall {
                needed: size,
                available: buffer.len(),
            });
        }

        let mut offset: usize = 0;

        // Header
        encode_header(buffer, Self::BLOCK_LENGTH, GET_RFQ_REQUEST_TEMPLATE_ID)?;
        offset = offset.checked_add(MESSAGE_HEADER_SIZE)
            .ok_or_else(|| SbeError::BufferTooSmall { needed: MESSAGE_HEADER_SIZE, available: 0 })?;

        // requestId
        let request_uuid = SbeUuid::from_uuid(self.request_id);
        request_uuid.encode(&mut buffer[offset..])?;
        offset = offset.checked_add(SbeUuid::SIZE)
            .ok_or_else(|| SbeError::BufferTooSmall { needed: size, available: buffer.len() })?;

        // rfqId
        let rfq_uuid = SbeUuid::from_uuid(self.rfq_id);
        rfq_uuid.encode(&mut buffer[offset..])?;
        offset = offset.checked_add(SbeUuid::SIZE)
            .ok_or_else(|| SbeError::BufferTooSmall { needed: size, available: buffer.len() })?;

        Ok(offset)
    }
}

impl SbeDecode for GetRfqRequest {
    fn decode(buffer: &[u8]) -> SbeResult<Self> {
        let (_block_length, template_id, _schema_id, _version) = decode_header(buffer)?;

        if template_id != GET_RFQ_REQUEST_TEMPLATE_ID {
            return Err(SbeError::UnknownTemplateId(template_id));
        }

        let mut offset: usize = MESSAGE_HEADER_SIZE;

        // requestId
        let request_uuid = SbeUuid::decode(&buffer[offset..])?;
        offset = offset.checked_add(SbeUuid::SIZE)
            .ok_or_else(|| SbeError::Overflow)?;

        // rfqId
        let rfq_uuid = SbeUuid::decode(&buffer[offset..])?;
        let _ = offset.checked_add(SbeUuid::SIZE)
            .ok_or_else(|| SbeError::Overflow)?;

        Ok(Self {
            request_id: request_uuid.to_uuid(),
            rfq_id: rfq_uuid.to_uuid(),
        })
    }
}

/// ExecuteTradeRequest - Execute a trade with a selected quote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecuteTradeRequest {
    /// RFQ identifier.
    pub rfq_id: Uuid,
    /// Selected quote identifier.
    pub quote_id: Uuid,
}

impl ExecuteTradeRequest {
    /// Block length for ExecuteTradeRequest.
    pub const BLOCK_LENGTH: u16 = 32;

    /// Creates a new ExecuteTradeRequest.
    #[must_use]
    pub fn new(rfq_id: Uuid, quote_id: Uuid) -> Self {
        Self { rfq_id, quote_id }
    }
}

impl SbeEncode for ExecuteTradeRequest {
    fn encoded_size(&self) -> usize {
        MESSAGE_HEADER_SIZE + Self::BLOCK_LENGTH as usize
    }

    fn encode(&self, buffer: &mut [u8]) -> SbeResult<usize> {
        let size = self.encoded_size();
        if buffer.len() < size {
            return Err(SbeError::BufferTooSmall {
                needed: size,
                available: buffer.len(),
            });
        }

        let mut offset: usize = 0;

        // Header
        encode_header(buffer, Self::BLOCK_LENGTH, EXECUTE_TRADE_REQUEST_TEMPLATE_ID)?;
        offset = offset.checked_add(MESSAGE_HEADER_SIZE)
            .ok_or_else(|| SbeError::BufferTooSmall { needed: size, available: buffer.len() })?;

        // rfqId
        let rfq_uuid = SbeUuid::from_uuid(self.rfq_id);
        rfq_uuid.encode(&mut buffer[offset..])?;
        offset = offset.checked_add(SbeUuid::SIZE)
            .ok_or_else(|| SbeError::BufferTooSmall { needed: size, available: buffer.len() })?;

        // quoteId
        let quote_uuid = SbeUuid::from_uuid(self.quote_id);
        quote_uuid.encode(&mut buffer[offset..])?;
        offset = offset.checked_add(SbeUuid::SIZE)
            .ok_or_else(|| SbeError::BufferTooSmall { needed: size, available: buffer.len() })?;

        Ok(offset)
    }
}

impl SbeDecode for ExecuteTradeRequest {
    fn decode(buffer: &[u8]) -> SbeResult<Self> {
        let (_block_length, template_id, _schema_id, _version) = decode_header(buffer)?;

        if template_id != EXECUTE_TRADE_REQUEST_TEMPLATE_ID {
            return Err(SbeError::UnknownTemplateId(template_id));
        }

        let mut offset: usize = MESSAGE_HEADER_SIZE;

        // rfqId
        let rfq_uuid = SbeUuid::decode(&buffer[offset..])?;
        offset = offset.checked_add(SbeUuid::SIZE)
            .ok_or_else(|| SbeError::Overflow)?;

        // quoteId
        let quote_uuid = SbeUuid::decode(&buffer[offset..])?;
        let _ = offset.checked_add(SbeUuid::SIZE)
            .ok_or_else(|| SbeError::Overflow)?;

        Ok(Self {
            rfq_id: rfq_uuid.to_uuid(),
            quote_id: quote_uuid.to_uuid(),
        })
    }
}

/// SubscribeQuotesRequest - Subscribe to quote updates for an RFQ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscribeQuotesRequest {
    /// RFQ identifier to subscribe to.
    pub rfq_id: Uuid,
}

impl SubscribeQuotesRequest {
    /// Block length for SubscribeQuotesRequest.
    pub const BLOCK_LENGTH: u16 = 16;

    /// Creates a new SubscribeQuotesRequest.
    #[must_use]
    pub fn new(rfq_id: Uuid) -> Self {
        Self { rfq_id }
    }
}

impl SbeEncode for SubscribeQuotesRequest {
    fn encoded_size(&self) -> usize {
        MESSAGE_HEADER_SIZE + Self::BLOCK_LENGTH as usize
    }

    fn encode(&self, buffer: &mut [u8]) -> SbeResult<usize> {
        let size = self.encoded_size();
        if buffer.len() < size {
            return Err(SbeError::BufferTooSmall {
                needed: size,
                available: buffer.len(),
            });
        }

        let mut offset: usize = 0;

        // Header
        encode_header(buffer, Self::BLOCK_LENGTH, SUBSCRIBE_QUOTES_REQUEST_TEMPLATE_ID)?;
        offset = offset.checked_add(MESSAGE_HEADER_SIZE)
            .ok_or_else(|| SbeError::BufferTooSmall { needed: size, available: buffer.len() })?;

        // rfqId
        let rfq_uuid = SbeUuid::from_uuid(self.rfq_id);
        rfq_uuid.encode(&mut buffer[offset..])?;
        offset = offset.checked_add(SbeUuid::SIZE)
            .ok_or_else(|| SbeError::BufferTooSmall { needed: size, available: buffer.len() })?;

        Ok(offset)
    }
}

impl SbeDecode for SubscribeQuotesRequest {
    fn decode(buffer: &[u8]) -> SbeResult<Self> {
        let (_block_length, template_id, _schema_id, _version) = decode_header(buffer)?;

        if template_id != SUBSCRIBE_QUOTES_REQUEST_TEMPLATE_ID {
            return Err(SbeError::UnknownTemplateId(template_id));
        }

        let mut offset = MESSAGE_HEADER_SIZE;

        // rfqId
        let rfq_uuid = SbeUuid::decode(&buffer[offset..])?;

        Ok(Self {
            rfq_id: rfq_uuid.to_uuid(),
        })
    }
}

/// SubscribeRfqStatusRequest - Subscribe to RFQ status updates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscribeRfqStatusRequest {
    /// RFQ identifier to subscribe to.
    pub rfq_id: Uuid,
}

impl SubscribeRfqStatusRequest {
    /// Block length for SubscribeRfqStatusRequest.
    pub const BLOCK_LENGTH: u16 = 16;

    /// Creates a new SubscribeRfqStatusRequest.
    #[must_use]
    pub fn new(rfq_id: Uuid) -> Self {
        Self { rfq_id }
    }
}

impl SbeEncode for SubscribeRfqStatusRequest {
    fn encoded_size(&self) -> usize {
        MESSAGE_HEADER_SIZE + Self::BLOCK_LENGTH as usize
    }

    fn encode(&self, buffer: &mut [u8]) -> SbeResult<usize> {
        let size = self.encoded_size();
        if buffer.len() < size {
            return Err(SbeError::BufferTooSmall {
                needed: size,
                available: buffer.len(),
            });
        }

        let mut offset: usize = 0;

        // Header
        encode_header(buffer, Self::BLOCK_LENGTH, SUBSCRIBE_RFQ_STATUS_REQUEST_TEMPLATE_ID)?;
        offset = offset.checked_add(MESSAGE_HEADER_SIZE)
            .ok_or_else(|| SbeError::BufferTooSmall { needed: size, available: buffer.len() })?;

        // rfqId
        let rfq_uuid = SbeUuid::from_uuid(self.rfq_id);
        rfq_uuid.encode(&mut buffer[offset..])?;
        offset = offset.checked_add(SbeUuid::SIZE)
            .ok_or_else(|| SbeError::BufferTooSmall { needed: size, available: buffer.len() })?;

        Ok(offset)
    }
}

impl SbeDecode for SubscribeRfqStatusRequest {
    fn decode(buffer: &[u8]) -> SbeResult<Self> {
        let (_block_length, template_id, _schema_id, _version) = decode_header(buffer)?;

        if template_id != SUBSCRIBE_RFQ_STATUS_REQUEST_TEMPLATE_ID {
            return Err(SbeError::UnknownTemplateId(template_id));
        }

        let mut offset = MESSAGE_HEADER_SIZE;

        // rfqId
        let rfq_uuid = SbeUuid::decode(&buffer[offset..])?;

        Ok(Self {
            rfq_id: rfq_uuid.to_uuid(),
        })
    }
}

/// UnsubscribeRequest - Unsubscribe from updates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsubscribeRequest {
    /// RFQ identifier to unsubscribe from.
    pub rfq_id: Uuid,
}

impl UnsubscribeRequest {
    /// Block length for UnsubscribeRequest.
    pub const BLOCK_LENGTH: u16 = 16;

    /// Creates a new UnsubscribeRequest.
    #[must_use]
    pub fn new(rfq_id: Uuid) -> Self {
        Self { rfq_id }
    }
}

impl SbeEncode for UnsubscribeRequest {
    fn encoded_size(&self) -> usize {
        MESSAGE_HEADER_SIZE + Self::BLOCK_LENGTH as usize
    }

    fn encode(&self, buffer: &mut [u8]) -> SbeResult<usize> {
        let size = self.encoded_size();
        if buffer.len() < size {
            return Err(SbeError::BufferTooSmall {
                needed: size,
                available: buffer.len(),
            });
        }

        let mut offset: usize = 0;

        // Header
        encode_header(buffer, Self::BLOCK_LENGTH, UNSUBSCRIBE_REQUEST_TEMPLATE_ID)?;
        offset = offset.checked_add(MESSAGE_HEADER_SIZE)
            .ok_or_else(|| SbeError::BufferTooSmall { needed: size, available: buffer.len() })?;

        // rfqId
        let rfq_uuid = SbeUuid::from_uuid(self.rfq_id);
        rfq_uuid.encode(&mut buffer[offset..])?;
        offset = offset.checked_add(SbeUuid::SIZE)
            .ok_or_else(|| SbeError::BufferTooSmall { needed: size, available: buffer.len() })?;

        Ok(offset)
    }
}

impl SbeDecode for UnsubscribeRequest {
    fn decode(buffer: &[u8]) -> SbeResult<Self> {
        let (_block_length, template_id, _schema_id, _version) = decode_header(buffer)?;

        if template_id != UNSUBSCRIBE_REQUEST_TEMPLATE_ID {
            return Err(SbeError::UnknownTemplateId(template_id));
        }

        let mut offset = MESSAGE_HEADER_SIZE;

        // rfqId
        let rfq_uuid = SbeUuid::decode(&buffer[offset..])?;

        Ok(Self {
            rfq_id: rfq_uuid.to_uuid(),
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn get_rfq_request_roundtrip() {
        let request = GetRfqRequest::new(Uuid::new_v4(), Uuid::new_v4());
        
        let mut buffer = vec![0u8; request.encoded_size()];
        let encoded_size = request.encode(&mut buffer).unwrap();
        
        assert_eq!(encoded_size, request.encoded_size());
        
        let decoded = GetRfqRequest::decode(&buffer).unwrap();
        assert_eq!(request, decoded);
    }

    #[test]
    fn execute_trade_request_roundtrip() {
        let request = ExecuteTradeRequest::new(Uuid::new_v4(), Uuid::new_v4());
        
        let mut buffer = vec![0u8; request.encoded_size()];
        let encoded_size = request.encode(&mut buffer).unwrap();
        
        assert_eq!(encoded_size, request.encoded_size());
        
        let decoded = ExecuteTradeRequest::decode(&buffer).unwrap();
        assert_eq!(request, decoded);
    }

    #[test]
    fn subscribe_quotes_request_roundtrip() {
        let request = SubscribeQuotesRequest::new(Uuid::new_v4());
        
        let mut buffer = vec![0u8; request.encoded_size()];
        let encoded_size = request.encode(&mut buffer).unwrap();
        
        assert_eq!(encoded_size, request.encoded_size());
        
        let decoded = SubscribeQuotesRequest::decode(&buffer).unwrap();
        assert_eq!(request, decoded);
    }
}
