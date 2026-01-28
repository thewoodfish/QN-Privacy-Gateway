# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-01-28

### Added

- Initial release of QN Privacy Gateway
- Privacy-preserving JSON-RPC gateway for Solana via QuickNode
- Three privacy modes: Strict, Balanced, and Dev
- Deterministic request hashing with SHA-256
- Request normalization to reduce fingerprinting
- TTL-based caching for safe read methods
- WebSocket proxy support for real-time subscriptions
- Retro CRT-style monitoring dashboard with live logs
- Server-Sent Events (SSE) for real-time log streaming
- Metrics endpoint with cache hit/miss tracking
- Configurable retry logic with exponential backoff
- Docker support with multi-stage build
- Comprehensive documentation
- Unit tests for normalization logic
- CI/CD pipeline with GitHub Actions
- MIT License

### Features

#### Privacy Features
- Request normalization removes client-specific variance
- Deterministic hashing produces same hash for identical requests
- ID stripping prevents request ID tracking
- Key sorting for consistent JSON representation

#### Caching
- In-memory cache with configurable TTL
- Mode-specific caching policies
- Automatic cache eviction on expiration
- Cache hit/miss metrics

#### Monitoring
- Real-time dashboard with SSE log streaming
- Metrics tracking (requests, cache hits/misses, unique hashes)
- Latency measurements (last + P95)
- Structured logging with tracing
- Log filtering by level, method, and hash

#### Configuration
- Environment variable based configuration
- Support for .env files
- Configurable privacy modes
- Adjustable cache TTL and request timeouts
- Retry attempt configuration

### Technical Details
- Built with Rust and Axum framework
- Async runtime using Tokio
- JSON-RPC 2.0 compliant
- WebSocket support via tokio-tungstenite
- Connection pooling with reqwest

### Documentation
- Comprehensive README with examples
- Architecture documentation
- Contributing guidelines
- Demo script for testing
- API documentation in code

[0.1.0]: https://github.com/yourusername/QN-Privacy-Gateway/releases/tag/v0.1.0
