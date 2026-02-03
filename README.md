# OTC RFQ Engine

[![Dual License](https://img.shields.io/badge/license-MIT-blue)](./LICENSE)
[![Crates.io](https://img.shields.io/crates/v/otc-rfq.svg)](https://crates.io/crates/otc-rfq)
[![Downloads](https://img.shields.io/crates/d/otc-rfq.svg)](https://crates.io/crates/otc-rfq)
[![Stars](https://img.shields.io/github/stars/joaquinbejar/otc-rfq.svg)](https://github.com/joaquinbejar/otc-rfq/stargazers)
[![Issues](https://img.shields.io/github/issues/joaquinbejar/otc-rfq.svg)](https://github.com/joaquinbejar/otc-rfq/issues)
[![PRs](https://img.shields.io/github/issues-pr/joaquinbejar/otc-rfq.svg)](https://github.com/joaquinbejar/otc-rfq/pulls)

[![Build Status](https://img.shields.io/github/actions/workflow/status/joaquinbejar/otc-rfq/ci.yml?branch=main)](https://github.com/joaquinbejar/otc-rfq/actions)
[![Coverage](https://img.shields.io/codecov/c/github/joaquinbejar/otc-rfq)](https://codecov.io/gh/joaquinbejar/otc-rfq)
[![Dependencies](https://img.shields.io/librariesio/github/joaquinbejar/otc-rfq)](https://libraries.io/github/joaquinbejar/otc-rfq)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/otc-rfq)

**High-performance OTC Request-for-Quote engine supporting DeFi protocols (0x, 1inch, Uniswap, Hashflow) and TradFi venues via FIX 4.4. Built in Rust with sub-millisecond latency.**

---

## Table of Contents

- [Overview](#overview)
- [Key Features](#key-features)
- [Architecture](#architecture)
- [Domain Model](#domain-model)
- [Project Structure](#project-structure)
- [Technology Stack](#technology-stack)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [API Reference](#api-reference)
- [Testing](#testing)
- [Performance](#performance)
- [Documentation](#documentation)
- [Roadmap](#roadmap)
- [Contribution and Contact](#contribution-and-contact)

---

## Overview

OTC RFQ Engine is an enterprise-grade liquidity aggregation system that enables institutional-quality Over-The-Counter (OTC) trading across multiple venues. The system implements a **reverse auction pattern** where quote requests are broadcast to multiple liquidity providers simultaneously, and the best quotes are presented to clients for execution.

### Problem Statement

Institutional traders face several challenges when executing large OTC trades:

- **Fragmented Liquidity**: Liquidity is spread across multiple venues (banks, market makers, DEXs)
- **Manual Processes**: Traditional RFQ workflows involve phone calls and chat messages
- **Limited Transparency**: Difficult to compare prices across venues in real-time
- **Operational Risk**: Manual execution increases error rates
- **DeFi Integration Gap**: Traditional systems don't connect to on-chain liquidity

### Solution

The OTC RFQ System addresses these challenges by:

- **Aggregating liquidity** from TradFi and DeFi sources
- **Automating the RFQ workflow** from request to settlement
- **Providing real-time price discovery** across all venues
- **Ensuring compliance** with regulatory requirements
- **Supporting multiple asset classes** (crypto, stocks, forex, commodities)

---

## Key Features

### Core Capabilities

| Feature | Description |
|---------|-------------|
| **Multi-Venue Aggregation** | Simultaneously query DeFi protocols and TradFi market makers |
| **Sub-Millisecond Latency** | Optimized for high-frequency trading workflows |
| **FIX 4.4 Protocol** | Native support for traditional finance connectivity via [IronFix](https://github.com/joaquinbejar/IronFix) |
| **DeFi Integration** | 0x, 1inch, Paraswap, Hashflow, Bebop, Uniswap V3, Curve |
| **Event Sourcing** | Complete audit trail with domain event persistence |
| **Multi-Protocol API** | gRPC (trading), REST (management), WebSocket (streaming) |
| **Binary Encoding** | High-performance SBE encoding via [IronSBE](https://github.com/joaquinbejar/IronSBE) |

### Supported Venue Types

```
┌─────────────────────────────────────────────────────────────────┐
│                      VENUE ECOSYSTEM                             │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  TradFi MMs  │  │     DEX      │  │  RFQ Proto   │          │
│  │              │  │  Aggregators │  │              │          │
│  │ • FIX 4.4    │  │              │  │ • Hashflow   │          │
│  │ • Internal   │  │ • 0x         │  │ • Bebop      │          │
│  │   Market     │  │ • 1inch      │  │ • Airswap    │          │
│  │   Makers     │  │ • Paraswap   │  │              │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐                             │
│  │   On-Chain   │  │  Settlement  │                             │
│  │   Protocols  │  │   Options    │                             │
│  │              │  │              │                             │
│  │ • Uniswap V3 │  │ • On-chain   │                             │
│  │ • Curve      │  │ • Off-chain  │                             │
│  │ • Balancer   │  │ • Hybrid     │                             │
│  └──────────────┘  └──────────────┘                             │
└─────────────────────────────────────────────────────────────────┘
```

### Supported Asset Classes

- **Crypto Spot**: BTC, ETH, and major tokens
- **Crypto Derivatives**: Perpetuals, options (future)
- **Forex**: Major currency pairs
- **Equities**: Stocks and ETFs
- **Commodities**: Precious metals, energy

---

## Architecture

The system follows a **layered architecture** based on Domain-Driven Design (DDD) principles with clear separation between business logic and infrastructure concerns.

### High-Level Architecture

```mermaid
flowchart TB
    subgraph CLIENTS["CLIENTS"]
        direction LR
        C1["Trading Desks"]
        C2["Algo Traders"]
        C3["DeFi Apps"]
    end

    subgraph API["API LAYER"]
        direction TB
        GRPC["gRPC<br/>(Trading)"]
        REST["REST<br/>(Management)"]
        WS["WebSocket<br/>(Streaming)"]
    end

    subgraph APPLICATION["APPLICATION LAYER"]
        direction TB
        RFQ_SVC["RFQ Service"]
        QUOTE_AGG["Quote Aggregation<br/>Engine"]
        EXEC_SVC["Execution Service"]
        COMPLIANCE["Compliance<br/>Service"]
    end

    subgraph DOMAIN["DOMAIN LAYER"]
        direction TB
        RFQ["RFQ Aggregate"]
        QUOTE["Quote Entity"]
        TRADE["Trade Aggregate"]
    end

    subgraph INFRASTRUCTURE["INFRASTRUCTURE LAYER"]
        direction TB
        
        subgraph VENUES["Venue Adapters"]
            FIX["FIX MM<br/>(IronFix)"]
            DEX["DEX Aggregators<br/>(0x, 1inch)"]
            RFQ_PROTO["RFQ Protocols<br/>(Hashflow)"]
        end
        
        subgraph PERSISTENCE["Persistence"]
            PG["PostgreSQL"]
            REDIS["Redis Cache"]
            KAFKA["Kafka Events"]
        end
    end

    CLIENTS --> API
    API --> APPLICATION
    APPLICATION --> DOMAIN
    APPLICATION --> INFRASTRUCTURE
    DOMAIN --> INFRASTRUCTURE
```

### Architecture Principles

| Principle | Description |
|-----------|-------------|
| **Layered Architecture** | Clear separation between domain, application, and infrastructure |
| **Domain-Driven Design** | Business logic encapsulated in domain layer |
| **Hexagonal Architecture** | Ports and adapters for external integrations |
| **Event Sourcing** | Immutable event log for audit and replay |
| **CQRS** | Separate read and write models where beneficial |
| **Microservices Ready** | Monolith that can be split into services |

### Layer Responsibilities

```
┌─────────────────────────────────────────────────────────────────┐
│                        API LAYER                                 │
│  Handles external communication (gRPC, REST, WebSocket)         │
│  Request/response serialization, authentication, rate limiting  │
├─────────────────────────────────────────────────────────────────┤
│                     APPLICATION LAYER                            │
│  Use cases, orchestration, transaction management               │
│  No business logic, only coordination                           │
├─────────────────────────────────────────────────────────────────┤
│                       DOMAIN LAYER                               │
│  Core business logic, entities, value objects, domain events    │
│  Pure Rust, no I/O, no external dependencies                    │
├─────────────────────────────────────────────────────────────────┤
│                   INFRASTRUCTURE LAYER                           │
│  Adapters for external systems (venues, databases, messaging)   │
│  Implements domain interfaces                                   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Domain Model

### RFQ Workflow (Reverse Auction)

```
┌─────────────────────────────────────────────────────────────────────┐
│                     REVERSE AUCTION FLOW                             │
│                                                                      │
│   1. CLIENT REQUESTS          2. VENUES RESPOND         3. CLIENT   │
│      QUOTES                      WITH PRICES               SELECTS  │
│                                                                      │
│   ┌─────────┐              ┌─────────┐                 ┌─────────┐  │
│   │ Client  │──RFQ──────▶ │ Venue 1 │ ────Quote────▶ │         │  │
│   │ (Taker) │              └─────────┘                 │ Best    │  │
│   │         │              ┌─────────┐                 │ Quote   │  │
│   │         │──RFQ──────▶ │ Venue 2 │ ────Quote────▶ │ Wins    │  │
│   │         │              └─────────┘                 │         │  │
│   │         │              ┌─────────┐                 │         │  │
│   │         │──RFQ──────▶ │ Venue 3 │ ────Quote────▶ │         │  │
│   └─────────┘              └─────────┘                 └─────────┘  │
│                                                                      │
│   4. EXECUTION                 5. SETTLEMENT                        │
│                                                                      │
│   ┌─────────┐              ┌─────────┐                              │
│   │ Trade   │──Execute──▶ │ Settle  │ ──▶ Assets Exchanged         │
│   │ Created │              │         │                              │
│   └─────────┘              └─────────┘                              │
└─────────────────────────────────────────────────────────────────────┘
```

### RFQ State Machine

```
                                ┌──────────────┐
                                │   CREATED    │
                                └──────┬───────┘
                                       │
                    start_quote_collection()
                                       │
                                       ▼
                                ┌──────────────┐
                     ┌─────────│   QUOTE      │─────────┐
                     │         │  REQUESTING  │         │
                     │         └──────┬───────┘         │
                     │                │                 │
                timeout/          receive_quote()   cancel()
                no quotes             │                 │
                     │                ▼                 │
                     │         ┌──────────────┐         │
                     │         │   QUOTES     │         │
                     ├────────▶│  RECEIVED    │◀────────┤
                     │         └──────┬───────┘         │
                     │                │                 │
                     │          select_quote()          │
                     │                │                 │
                     │                ▼                 │
                     │         ┌──────────────┐         │
                     │         │  EXECUTING   │─────────┤
                     │         └──────┬───────┘         │
                     │                │         │       │
                     │    mark_executed()  mark_failed()│
                     │                │         │       │
                     ▼                ▼         ▼       ▼
              ┌──────────┐     ┌──────────┐ ┌──────────┐┌──────────┐
              │ EXPIRED  │     │ EXECUTED │ │  FAILED  ││CANCELLED │
              └──────────┘     └──────────┘ └──────────┘└──────────┘
```

### Core Aggregates

| Aggregate | Description |
|-----------|-------------|
| **RFQ** | Central aggregate managing the complete trading workflow |
| **Quote** | Price quote from a venue with validity period |
| **Trade** | Executed trade with settlement tracking |
| **Venue** | Liquidity source configuration and health |
| **Counterparty** | Client or market maker with compliance status |

### Domain Events

```
RFQ Lifecycle Events:
├── RFQCreated
├── QuoteCollectionStarted
├── QuoteReceived
├── QuoteSelected
├── TradeExecuted
├── RFQCancelled
└── RFQExpired

Settlement Events:
├── SettlementInitiated
├── SettlementConfirmed
└── SettlementFailed

Compliance Events:
├── ComplianceCheckPassed
└── ComplianceCheckFailed
```

---

## Project Structure

```
otc-rfq/
├── src/
│   ├── domain/                  # Core business logic (DDD)
│   │   ├── entities/            # Aggregates: RFQ, Quote, Trade, Venue
│   │   │   ├── rfq.rs           # RFQ aggregate with state machine
│   │   │   ├── quote.rs         # Quote entity
│   │   │   ├── trade.rs         # Trade aggregate
│   │   │   └── venue.rs         # Venue aggregate
│   │   ├── value_objects/       # Immutable value types
│   │   │   ├── ids.rs           # RFQId, QuoteId, TradeId, VenueId
│   │   │   ├── price.rs         # Price with precision
│   │   │   ├── quantity.rs      # Quantity with validation
│   │   │   ├── timestamp.rs     # Timestamp wrapper
│   │   │   └── enums.rs         # OrderSide, AssetClass, etc.
│   │   ├── events/              # Domain events for event sourcing
│   │   │   ├── rfq_events.rs    # RFQ lifecycle events
│   │   │   └── trade_events.rs  # Trade execution events
│   │   └── errors/              # Domain-specific errors
│   │       └── domain_error.rs
│   │
│   ├── application/             # Use cases and services
│   │   ├── use_cases/           # Business workflows
│   │   │   ├── create_rfq.rs    # CreateRFQUseCase
│   │   │   ├── collect_quotes.rs # CollectQuotesUseCase
│   │   │   └── execute_trade.rs # ExecuteTradeUseCase
│   │   ├── services/            # Application services
│   │   │   ├── quote_aggregation.rs  # Quote ranking and selection
│   │   │   └── circuit_breaker.rs    # Fault tolerance
│   │   ├── dto/                 # Data transfer objects
│   │   └── error.rs             # Application errors
│   │
│   ├── infrastructure/          # External adapters
│   │   ├── venues/              # Venue adapters (ports & adapters)
│   │   │   ├── traits.rs        # VenueAdapter trait
│   │   │   ├── internal_mm.rs   # Internal market maker
│   │   │   ├── fix_adapter.rs   # FIX protocol adapter
│   │   │   ├── dex_aggregator.rs # DEX aggregator adapter
│   │   │   └── registry.rs      # Venue registry
│   │   ├── persistence/         # Data persistence
│   │   │   ├── postgres/        # PostgreSQL repositories
│   │   │   ├── redis/           # Redis cache
│   │   │   └── event_store.rs   # Event sourcing store
│   │   ├── blockchain/          # Blockchain clients
│   │   │   └── ethereum.rs      # Ethereum/L2 client
│   │   └── sbe/                 # SBE binary encoding
│   │
│   ├── api/                     # External interfaces
│   │   ├── grpc/                # gRPC services (tonic)
│   │   │   ├── service.rs       # RFQ service implementation
│   │   │   └── proto/           # Protocol buffer definitions
│   │   ├── rest/                # REST endpoints (axum)
│   │   │   ├── handlers.rs      # Request handlers
│   │   │   └── routes.rs        # Route configuration
│   │   ├── websocket/           # Real-time streaming
│   │   │   └── handlers.rs      # WebSocket handlers
│   │   └── middleware/          # Cross-cutting concerns
│   │       ├── auth.rs          # JWT authentication
│   │       └── rate_limit.rs    # Rate limiting
│   │
│   ├── config.rs                # Configuration management
│   ├── lib.rs                   # Library entry point
│   └── main.rs                  # Application entry point
│
├── proto/                       # Protocol Buffer definitions
│   └── rfq.proto                # gRPC service definitions
├── schemas/sbe/                 # SBE message schemas
├── migrations/                  # Database migrations
├── benches/                     # Performance benchmarks
└── tests/                       # Integration tests
```

---

## Technology Stack

| Layer | Technology | Rationale |
|-------|------------|-----------|
| **Language** | Rust | Performance, safety, async |
| **Async Runtime** | Tokio | Industry standard for Rust async |
| **Web Framework** | Axum | Modern, tower-based |
| **gRPC** | Tonic | High-performance Rust gRPC |
| **Database** | PostgreSQL | Reliability, JSON support |
| **Cache** | Redis | Speed, pub/sub |
| **Message Queue** | Kafka | Throughput, durability |
| **Blockchain** | ethers-rs | Ethereum interaction |
| **Serialization** | serde, prost | Rust standard |
| **Binary Encoding** | IronSBE | High-performance SBE |
| **FIX Protocol** | IronFix | FIX 4.4 support |
| **Observability** | tracing | Structured logging |

### Dependencies Highlights

```toml
[dependencies]
tokio = { version = "1.49", features = ["full"] }
axum = { version = "0.8", features = ["ws", "macros"] }
tonic = { version = "0.14", features = ["transport", "tls"] }
sqlx = { version = "0.8", features = ["postgres", "uuid", "chrono"] }
redis = { version = "1.0", features = ["tokio-comp"] }
ethers = { version = "2.0", features = ["rustls"] }
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1.20", features = ["v4", "serde"] }
rust_decimal = { version = "1.40", features = ["serde"] }
```

---

## Prerequisites

- **Rust**: 1.75+ (stable)
- **PostgreSQL**: 14+
- **Redis**: 7+
- **Kafka**: 3+ (optional, for event streaming)
- **Docker**: 20+ (optional, for containerized development)

---

## Installation

### From Source

```bash
# Clone repository
git clone https://github.com/joaquinbejar/otc-rfq.git
cd otc-rfq

# Build release
cargo build --release

# Run tests
cargo test --all-features

# Run lints
cargo clippy --all-targets --all-features -- -D warnings
```

### From Crates.io

```bash
cargo add otc-rfq
```

### Docker

```bash
# Build image
docker build -t otc-rfq:latest .

# Run container
docker run -p 50051:50051 -p 8080:8080 otc-rfq:latest
```

---

## Configuration

### Environment Variables

Create a `.env` file or set environment variables:

```bash
# =============================================================================
# Database Configuration
# =============================================================================
DATABASE_URL=postgres://user:pass@localhost:5432/otc_rfq
DATABASE_MAX_CONNECTIONS=20
DATABASE_MIN_CONNECTIONS=5

# =============================================================================
# Redis Configuration
# =============================================================================
REDIS_URL=redis://localhost:6379
REDIS_MAX_CONNECTIONS=10

# =============================================================================
# API Configuration
# =============================================================================
GRPC_PORT=50051
REST_PORT=8080
WEBSOCKET_PORT=8081

# =============================================================================
# Authentication
# =============================================================================
JWT_SECRET=your-secret-key-here
JWT_EXPIRATION_HOURS=24
API_KEY_HEADER=X-API-Key

# =============================================================================
# FIX Protocol (TradFi)
# =============================================================================
FIX_ENABLED=true
FIX_SENDER_COMP_ID=OTC_RFQ
FIX_TARGET_COMP_ID=MARKET_MAKER
FIX_HOST=fix.marketmaker.com
FIX_PORT=9878

# =============================================================================
# DeFi Venues
# =============================================================================
ZERO_X_API_KEY=your_api_key
ZERO_X_BASE_URL=https://api.0x.org
ONE_INCH_API_KEY=your_api_key
ONE_INCH_BASE_URL=https://api.1inch.io
HASHFLOW_API_KEY=your_api_key

# =============================================================================
# Blockchain
# =============================================================================
ETHEREUM_RPC_URL=https://mainnet.infura.io/v3/your-key
POLYGON_RPC_URL=https://polygon-rpc.com
ARBITRUM_RPC_URL=https://arb1.arbitrum.io/rpc

# =============================================================================
# Observability
# =============================================================================
LOG_LEVEL=info
TRACING_ENABLED=true
METRICS_PORT=9090
```

### Configuration File (TOML)

```toml
[server]
grpc_port = 50051
rest_port = 8080
websocket_port = 8081

[database]
url = "postgres://user:pass@localhost:5432/otc_rfq"
max_connections = 20
min_connections = 5

[redis]
url = "redis://localhost:6379"
max_connections = 10

[venues]
quote_timeout_ms = 500
min_quotes_required = 1
max_concurrent_requests = 100

[compliance]
kyc_required = true
aml_check_enabled = true
max_notional_usd = 10000000
```

---

## Usage

### Starting the Server

```bash
# Development
cargo run

# Production
cargo run --release

# With custom config
CONFIG_PATH=./config.toml cargo run --release
```

### gRPC Client Example

```rust
use otc_rfq::api::grpc::RfqServiceClient;
use otc_rfq::api::grpc::{CreateRfqRequest, Instrument, OrderSide, AssetClass};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to gRPC server
    let mut client = RfqServiceClient::connect("http://localhost:50051").await?;
    
    // Create RFQ
    let response = client.create_rfq(CreateRfqRequest {
        client_id: "DESK_001".to_string(),
        instrument: Some(Instrument {
            symbol: "BTC/USD".to_string(),
            asset_class: AssetClass::CryptoSpot as i32,
            ..Default::default()
        }),
        side: OrderSide::Buy as i32,
        quantity: Some(Decimal { value: "10.0".to_string() }),
        timeout_seconds: 30,
    }).await?;
    
    let rfq = response.into_inner().rfq.unwrap();
    println!("RFQ created: {}", rfq.id);
    
    // Stream quotes
    let mut stream = client.stream_quotes(StreamQuotesRequest {
        rfq_id: rfq.id.clone(),
    }).await?.into_inner();
    
    while let Some(quote) = stream.message().await? {
        println!("Quote received: {} @ {}", quote.venue_id, quote.price);
    }
    
    Ok(())
}
```

### REST API Examples

```bash
# Create RFQ
curl -X POST http://localhost:8080/api/v1/rfqs \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "client_id": "DESK_001",
    "symbol": "BTC/USD",
    "side": "BUY",
    "quantity": "10.0",
    "timeout_seconds": 30
  }'

# Get RFQ by ID
curl http://localhost:8080/api/v1/rfqs/{rfq_id} \
  -H "Authorization: Bearer $TOKEN"

# List RFQs with pagination
curl "http://localhost:8080/api/v1/rfqs?page=1&page_size=20&status=QUOTES_RECEIVED" \
  -H "Authorization: Bearer $TOKEN"

# Execute trade
curl -X POST http://localhost:8080/api/v1/rfqs/{rfq_id}/execute \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "quote_id": "quote-uuid-here"
  }'

# List venues
curl http://localhost:8080/api/v1/venues \
  -H "Authorization: Bearer $TOKEN"

# Get venue health
curl http://localhost:8080/api/v1/venues/{venue_id}/health \
  -H "Authorization: Bearer $TOKEN"
```

### WebSocket Streaming

```javascript
// Connect to WebSocket
const ws = new WebSocket('ws://localhost:8081/ws');

// Subscribe to quote updates
ws.send(JSON.stringify({
  type: 'subscribe',
  channel: 'quotes',
  rfq_id: 'rfq-uuid-here'
}));

// Handle messages
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Quote update:', data);
};
```

---

## API Reference

### gRPC Services

| Service | Method | Description |
|---------|--------|-------------|
| `RfqService` | `CreateRfq` | Create a new RFQ |
| `RfqService` | `GetRfq` | Get RFQ by ID |
| `RfqService` | `ListRfqs` | List RFQs with filters |
| `RfqService` | `CancelRfq` | Cancel an active RFQ |
| `RfqService` | `StreamQuotes` | Stream quotes for an RFQ |
| `RfqService` | `ExecuteTrade` | Execute trade on selected quote |
| `VenueService` | `ListVenues` | List available venues |
| `VenueService` | `GetVenueHealth` | Get venue health status |

### REST Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/v1/rfqs` | Create RFQ |
| `GET` | `/api/v1/rfqs/{id}` | Get RFQ |
| `GET` | `/api/v1/rfqs` | List RFQs |
| `DELETE` | `/api/v1/rfqs/{id}` | Cancel RFQ |
| `POST` | `/api/v1/rfqs/{id}/execute` | Execute trade |
| `GET` | `/api/v1/venues` | List venues |
| `GET` | `/api/v1/venues/{id}/health` | Venue health |
| `GET` | `/api/v1/trades` | List trades |
| `GET` | `/api/v1/trades/{id}` | Get trade |

### WebSocket Channels

| Channel | Description |
|---------|-------------|
| `quotes` | Real-time quote updates |
| `rfq_status` | RFQ state changes |
| `trades` | Trade execution updates |
| `venue_health` | Venue health changes |

---

## Testing

### Running Tests

```bash
# Run all tests
cargo test --all-features

# Run specific test module
cargo test domain::entities::rfq

# Run with output
cargo test -- --nocapture

# Run integration tests (requires database)
TEST_DATABASE_URL=postgres://... cargo test --test integration

# Run benchmarks
cargo bench
```

### Test Categories

| Category | Description | Count |
|----------|-------------|-------|
| **Unit Tests** | Domain logic, value objects | 800+ |
| **Integration Tests** | Repository, API handlers | 200+ |
| **E2E Tests** | Complete RFQ workflow | 12 |
| **Property Tests** | Proptest-based invariants | 50+ |
| **Benchmarks** | Performance measurements | 10+ |

### Pre-Push Checks

```bash
# Run all checks
make pre-push

# Individual checks
make fmt       # Format code
make lint      # Run clippy
make test      # Run tests
make doc       # Generate docs
```

---

## Performance

### Targets

| Metric | Target | Achieved |
|--------|--------|----------|
| Quote collection latency | < 100ms (p99) | ✅ |
| Trade execution latency | < 50ms (p99) | ✅ |
| Concurrent RFQs | 10,000+ | ✅ |
| Quotes per second | 100,000+ | ✅ |
| API response time | < 10ms (p99) | ✅ |

### Benchmarks

```bash
# Run benchmarks
cargo bench

# Quote aggregation benchmark
cargo bench --bench quote_aggregation
```

### Optimization Features

- **Zero-copy deserialization** with SBE encoding
- **Connection pooling** for database and Redis
- **Concurrent venue requests** with tokio
- **Circuit breakers** for fault tolerance
- **Response caching** with Redis

---

## Documentation

### Internal Documentation

| Document | Description |
|----------|-------------|
| [System Overview](.internalDoc/architecture/01-SYSTEM-OVERVIEW.md) | Business context and goals |
| [Architecture](.internalDoc/architecture/02-ARCHITECTURE.md) | Layered architecture design |
| [Domain Model](.internalDoc/architecture/03-DOMAIN-MODEL.md) | Entities, aggregates, events |
| [Data Model](.internalDoc/architecture/04-DATA-MODEL.md) | Database schema and caching |
| [API Overview](.internalDoc/api/01-API-OVERVIEW.md) | Multi-protocol API design |
| [FIX Protocol](.internalDoc/protocols/02-FIX-PROTOCOL.md) | FIX 4.4 integration |
| [DeFi Protocols](.internalDoc/protocols/03-DEFI-PROTOCOLS.md) | DEX and RFQ protocol adapters |
| [SBE Encoding](.internalDoc/protocols/04-SBE-ENCODING.md) | Binary message encoding |
| [Rust Guidelines](.internalDoc/09-rust-guidelines.md) | Coding standards |

### API Documentation

```bash
# Generate and open docs
cargo doc --open --no-deps
```

---

## Roadmap

### Completed Milestones

- [x] **M0**: Project Setup - Repository, CI/CD, documentation structure
- [x] **M1**: Domain Layer - Entities, value objects, domain events
- [x] **M2**: Infrastructure Layer - Venues, persistence, blockchain
- [x] **M3**: Application Layer - Use cases, services, DTOs
- [x] **M4**: API Layer - gRPC, REST, WebSocket
- [x] **M5**: Testing & Integration - Unit, integration, E2E tests

### Future Milestones

- [ ] **M6**: Production Hardening - Monitoring, alerting, deployment
- [ ] **M7**: Advanced Features - Algorithmic execution, portfolio management
- [ ] **M8**: Multi-Region - Geographic distribution, failover

See [GitHub Issues](https://github.com/joaquinbejar/otc-rfq/issues) for detailed progress.

---

## Contribution and Contact

We welcome contributions to this project! If you would like to contribute, please follow these steps:

1. Fork the repository.
2. Create a new branch for your feature or bug fix.
3. Make your changes and ensure that the project still builds and all tests pass.
4. Commit your changes and push your branch to your forked repository.
5. Submit a pull request to the main repository.

### Development Workflow

```bash
# Create feature branch
git checkout -b M[N]/issue-[NUM]-description

# Make changes and test
cargo test --all-features
make pre-push

# Commit and push
git commit -m "feat: description"
git push origin your-branch

# Create PR
gh pr create
```

If you have any questions, issues, or would like to provide feedback, please feel free to contact the project maintainer:

### **Contact Information**

- **Author**: Joaquín Béjar García
- **Email**: jb@taunais.com
- **Telegram**: [@joaquin_bejar](https://t.me/joaquin_bejar)
- **Repository**: <https://github.com/joaquinbejar/otc-rfq>
- **Documentation**: <https://docs.rs/otc-rfq>

We appreciate your interest and look forward to your contributions!

---

**License**: MIT
