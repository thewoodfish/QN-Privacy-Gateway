# Architecture Documentation

## System Overview

QN Privacy Gateway acts as a privacy-preserving middleware between Solana clients and QuickNode infrastructure.

## Component Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│                           CLIENT LAYER                                  │
│                                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                 │
│  │   Wallets    │  │    dApps     │  │   Indexers   │                 │
│  │  (Phantom,   │  │   (React,    │  │   (Custom    │                 │
│  │   Solflare)  │  │    Vue, etc) │  │   Services)  │                 │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘                 │
│         │                  │                  │                         │
│         └──────────────────┼──────────────────┘                         │
│                            │                                            │
│                            │ JSON-RPC / WebSocket                       │
│                            ▼                                            │
└─────────────────────────────────────────────────────────────────────────┘
                             │
┌────────────────────────────┼────────────────────────────────────────────┐
│                            │                                            │
│                   QN PRIVACY GATEWAY                                    │
│                            │                                            │
│  ┌─────────────────────────▼────────────────────────────┐              │
│  │                    HTTP/WS Server                     │              │
│  │                  (Axum Router)                        │              │
│  │                                                       │              │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐    │              │
│  │  │  POST /    │  │  GET /ws   │  │ GET /dash  │    │              │
│  │  │  (RPC)     │  │ (WebSocket)│  │ (Dashboard)│    │              │
│  │  └─────┬──────┘  └─────┬──────┘  └────────────┘    │              │
│  └────────┼───────────────┼────────────────────────────┘              │
│           │               │                                            │
│  ┌────────▼───────────────┼────────────────────────────┐              │
│  │   Request Handler      │   WebSocket Proxy          │              │
│  │                        │                             │              │
│  │  1. Parse JSON-RPC     │   1. Establish connection  │              │
│  │  2. Extract method     │   2. Bidirectional relay   │              │
│  │  3. Normalize request  │   3. Message mapping       │              │
│  │  4. Generate hash      │                             │              │
│  │  5. Check cache        │                             │              │
│  │  6. Forward to QN      │                             │              │
│  │  7. Cache response     │                             │              │
│  └────────┬───────────────┴────────────────────────────┘              │
│           │                                                            │
│  ┌────────▼──────────────────────────────────────────┐                │
│  │           Privacy Features                        │                │
│  │                                                   │                │
│  │  ┌──────────────────┐  ┌──────────────────┐     │                │
│  │  │  Normalization   │  │  Deterministic   │     │                │
│  │  │  Engine          │  │  Hashing         │     │                │
│  │  │                  │  │  (SHA-256)       │     │                │
│  │  │  • Sort keys     │  │                  │     │                │
│  │  │  • Remove IDs    │  │  • Generate      │     │                │
│  │  │  • Canonicalize  │  │    cache keys    │     │                │
│  │  └──────────────────┘  └──────────────────┘     │                │
│  └───────────────────────────────────────────────────┘                │
│                                                                        │
│  ┌─────────────────────────────────────────────────────┐              │
│  │           Cache Layer (In-Memory)                   │              │
│  │                                                     │              │
│  │  • TTL-based expiration                            │              │
│  │  • Hash-keyed storage                              │              │
│  │  • Safe read methods only                          │              │
│  │  • RwLock for concurrent access                    │              │
│  └─────────────────────────────────────────────────────┘              │
│                                                                        │
│  ┌─────────────────────────────────────────────────────┐              │
│  │           Metrics & Logging                         │              │
│  │                                                     │              │
│  │  • Request counting                                │              │
│  │  • Cache hit/miss ratio                            │              │
│  │  • Unique hash tracking                            │              │
│  │  • Latency measurements                            │              │
│  │  • Structured event logging                        │              │
│  │  • SSE broadcast for dashboard                     │              │
│  └─────────────────────────────────────────────────────┘              │
│                            │                                           │
└────────────────────────────┼───────────────────────────────────────────┘
                             │
                             │ Proxied Request
                             │ (Normalized & Hashed)
                             │
┌────────────────────────────▼───────────────────────────────────────────┐
│                                                                         │
│                      QUICKNODE INFRASTRUCTURE                           │
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────┐    │
│  │                                                               │    │
│  │           QuickNode Solana RPC/WS Endpoint                    │    │
│  │                                                               │    │
│  │  • High-performance Solana RPC                               │    │
│  │  • WebSocket subscriptions                                   │    │
│  │  • Enterprise-grade infrastructure                           │    │
│  │  • Global availability                                       │    │
│  │                                                               │    │
│  └───────────────────────────────────────────────────────────────┘    │
│                            │                                           │
└────────────────────────────┼───────────────────────────────────────────┘
                             │
                             ▼
                    Solana Blockchain
```

## Data Flow

### 1. RPC Request Flow

```
Client → Gateway → Normalize → Hash → Cache Check
                                           │
                                           ├─ Hit → Return Cached
                                           │
                                           └─ Miss → Forward to QuickNode
                                                         │
                                                         ▼
                                                    Store in Cache
                                                         │
                                                         ▼
                                                   Return to Client
```

### 2. WebSocket Flow

```
Client ←──→ Gateway ←──→ QuickNode WS
            (Proxy)

• Bidirectional message relay
• No modification to messages
• Connection lifecycle management
```

## Privacy Mechanisms

### Request Normalization

1. **Key Sorting**: All JSON object keys are sorted lexicographically
2. **ID Stripping**: Client-specific request IDs are removed for hashing
3. **Version Canonicalization**: JSON-RPC version is normalized to "2.0"
4. **Recursive Processing**: Normalization applies to nested objects and arrays

### Deterministic Hashing

- SHA-256 hash of normalized request
- Same semantic request always produces same hash
- Hash used as cache key and for tracking unique requests
- Reduces ability to track individual clients by request variance

### Caching Strategy

**Strict Mode**:
- Caches: getAccountInfo, getBalance, getLatestBlockhash, getSlot, getBlock

**Balanced Mode**:
- Caches: getLatestBlockhash, getSlot, getBalance

**Dev Mode**:
- No caching (pass-through)

## Configuration

Environment variables control all behavior:

- **QUICKNODE_RPC_URL**: Upstream HTTP endpoint
- **QUICKNODE_WS_URL**: Upstream WebSocket endpoint
- **PRIVACY_MODE**: strict | balanced | dev
- **CACHE_TTL_SECONDS**: Time-to-live for cached responses
- **REQUEST_TIMEOUT_MS**: Timeout for upstream requests
- **RETRY_ATTEMPTS**: Number of retries on failure
- **BIND_ADDR**: Local binding address

## Monitoring

### Metrics Collected

- **requests_total**: Total number of requests processed
- **cache_hits**: Number of cache hits
- **cache_misses**: Number of cache misses
- **unique_request_hashes**: Count of unique request patterns

### Dashboard

- Real-time SSE stream of structured log events
- Live metrics display with auto-refresh
- Filtering by log level, method, and hash
- Latency tracking (last + P95)
- Retro CRT visual theme

## Security Considerations

1. **No Request Logging**: Individual request payloads are not logged
2. **Hash-Based Tracking**: Only hashes are stored, not full requests
3. **No Response Modification**: Responses are passed through unchanged
4. **Configurable Privacy**: Users can choose appropriate privacy/performance trade-off
5. **TLS Support**: HTTPS supported for secure upstream connections

## Performance

- **Async Runtime**: Tokio for efficient concurrent request handling
- **Connection Pooling**: Reqwest HTTP client with connection reuse
- **In-Memory Cache**: Lock-free reads with RwLock
- **Lazy Eviction**: Cache entries evicted on access, not proactively
- **Backoff Strategy**: Linear backoff for upstream retries

## Deployment Options

1. **Docker Container**: Single-command deployment
2. **Binary Deployment**: Static binary with minimal dependencies
3. **Source Build**: Cargo build for custom configurations
4. **Cloud Platforms**: Deployable to any container platform (K8s, ECS, Cloud Run, etc.)
