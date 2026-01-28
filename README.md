# 🔐 QN Privacy Gateway

> A privacy-preserving Solana JSON-RPC gateway for QuickNode

[![Rust](https://img.shields.io/badge/rust-1.76%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)
[![Docker](https://img.shields.io/badge/docker-ready-brightgreen.svg)](./Dockerfile)

**QN Privacy Gateway** is a privacy-preserving Solana JSON-RPC gateway that sits between clients and QuickNode, reducing metadata leakage and fingerprinting while remaining fully compatible with existing Solana wallets, dApps, and indexers.

This gateway is designed to run on top of QuickNode RPC infrastructure.

## ✨ Features

- 🛡️ **Privacy-First**: Deterministic request hashing reduces client fingerprinting
- 🔄 **Request Normalization**: Eliminates client-specific variance in RPC calls
- ⚡ **Smart Caching**: Configurable TTL caching for safe read methods
- 🔌 **WebSocket Support**: Full WebSocket proxy for real-time subscriptions
- 🎯 **Multiple Privacy Modes**: Choose between `strict`, `balanced`, or `dev` modes
- 📊 **Live Dashboard**: Beautiful retro CRT-style monitoring dashboard
- 🐳 **Docker Ready**: Single-command deployment with Docker
- 🔧 **Fully Configurable**: Environment-based configuration

## 🏗️ Architecture

```
┌─────────────────────────┐
│  Client (Wallet/dApp)   │
└───────────┬─────────────┘
            │
            │ HTTP/WS
            ▼
┌─────────────────────────┐
│  QN Privacy Gateway     │
│  ┌──────────────────┐   │
│  │ Normalization    │   │
│  │ Hashing          │   │
│  │ Cache Layer      │   │
│  └──────────────────┘   │
└───────────┬─────────────┘
            │
            │ Proxied Request
            ▼
┌─────────────────────────┐
│  QuickNode Solana RPC   │
│  (HTTP + WebSocket)     │
└─────────────────────────┘
```

## 🔒 Privacy Guarantees

### How It Works

1. **Deterministic Hashing**: Semantically identical requests produce the same hash, regardless of client-specific variations
2. **Request Normalization**: Removes client-specific metadata that could be used for fingerprinting
3. **Smart Caching**: Safe read methods are cached to reduce upstream visibility of repeated queries
4. **Zero Response Modification**: No response data is modified or redacted - full compatibility guaranteed

### Privacy Modes

| Mode | Normalization | Caching | Use Case |
|------|--------------|---------|----------|
| **Strict** | Full | All safe methods | Maximum privacy |
| **Balanced** | Full | Common methods only | Good privacy + performance |
| **Dev** | Minimal | Disabled | Development & debugging |

## 🚀 Quick Start

### Prerequisites

- [QuickNode](https://www.quicknode.com/) Solana RPC endpoint (mainnet or devnet)
- Docker (for containerized deployment) OR Rust 1.76+ (for local build)

### 1. QuickNode Setup

1. Create or select a Solana endpoint in your QuickNode dashboard
2. Copy the HTTPS RPC URL (and optionally the WebSocket URL)
3. Set the URL in your environment or `.env` file

### 2. Run with Docker

```bash
docker build -t qn-privacy-gateway .

docker run --rm -p 8080:8080 \
  -e QUICKNODE_RPC_URL="https://YOUR-QUICKNODE-URL" \
  -e PRIVACY_MODE=balanced \
  qn-privacy-gateway
```

### 3. Local Build & Run

```bash
# Clone the repository
git clone https://github.com/yourusername/QN-Privacy-Gateway.git
cd QN-Privacy-Gateway

# Copy and configure environment
cp .env.example .env
# Edit .env with your QuickNode URL

# Build and run
cargo build --release
cargo run --release
```

The gateway will start on `http://localhost:8080`

## 📡 Usage

### Example Request

```bash
curl -s http://localhost:8080/ \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getSlot","params":[]}'
```

### WebSocket Support

Connect to the gateway WebSocket endpoint for real-time subscriptions:

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');
ws.onopen = () => {
  ws.send(JSON.stringify({
    jsonrpc: '2.0',
    id: 1,
    method: 'accountSubscribe',
    params: ['YOUR_ACCOUNT_ADDRESS', { encoding: 'jsonParsed' }]
  }));
};
```

## 📊 Metrics & Monitoring

### Metrics Endpoint

`GET /metrics` returns:

```json
{
  "requests_total": 0,
  "cache_hits": 0,
  "cache_misses": 0,
  "unique_request_hashes": 0
}
```

### Live Dashboard

Open the retro CRT-style dashboard at `http://localhost:8080/dashboard` to view:

- 📈 Real-time request metrics and cache statistics
- 📝 Live streaming logs with event filtering
- ⏱️ Latency tracking (last request + P95)
- 🔍 Request hash and method filtering
- 🎨 Beautiful retro terminal aesthetic

![Dashboard Preview](assets/dashboard-preview.png)
*Retro CRT-style monitoring dashboard with live logs and metrics*

## ⚙️ Configuration

All configuration is done via environment variables (`.env` file supported):

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `QUICKNODE_RPC_URL` | ✅ Yes | - | QuickNode Solana RPC endpoint URL |
| `QUICKNODE_WS_URL` | ❌ Optional | - | QuickNode WebSocket URL for `/ws` proxying |
| `PRIVACY_MODE` | ❌ Optional | `balanced` | Privacy mode: `strict` \| `balanced` \| `dev` |
| `CACHE_TTL_SECONDS` | ❌ Optional | `5` | Cache TTL for safe read methods |
| `REQUEST_TIMEOUT_MS` | ❌ Optional | `8000` | Upstream request timeout (milliseconds) |
| `RETRY_ATTEMPTS` | ❌ Optional | `3` | Number of retry attempts for upstream errors |
| `BIND_ADDR` | ❌ Optional | `0.0.0.0:8080` | Gateway listen address |
| `RUST_LOG` | ❌ Optional | - | Logging level (e.g., `info`, `debug`) |

## 🗺️ Roadmap

- [ ] Deterministic batch caching with per-request fan-in/out
- [ ] Optional jitter for cache TTL to reduce timing fingerprints
- [ ] Pluggable allow/deny list for custom RPC methods
- [ ] Rate limiting and request throttling
- [ ] Prometheus metrics export
- [ ] Multi-backend load balancing
- [ ] Request signing and authentication

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built for the QuickNode hackathon
- Powered by [Rust](https://www.rust-lang.org/) and [Axum](https://github.com/tokio-rs/axum)
- Designed for the [Solana](https://solana.com/) ecosystem

---

**Built with ❤️ for privacy and performance**
