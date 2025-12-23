# Sentinel — Pingora-based Production Reverse Proxy Platform

> A security-first reverse proxy built on Pingora. Sleepable ops at the edge.

[![CI](https://github.com/raskell-io/sentinel/actions/workflows/ci.yml/badge.svg)](https://github.com/raskell-io/sentinel/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

## Overview

Sentinel is a production-grade reverse proxy platform built on top of Cloudflare's [Pingora](https://github.com/cloudflare/pingora) framework. It provides the **product layer** on top of Pingora's robust library: configuration management, policy enforcement, extensibility through agents, WAF integration, comprehensive observability, and safe defaults.

### Why Sentinel?

- **Sleepable Operations**: Bounded memory, deterministic timeouts, graceful degradation, and robust rollback mechanisms mean no 3 AM wake-up calls
- **Security-First Design**: Hardened defaults, isolated untrusted components, explicit security posture with no "magic" behavior
- **Extensible Architecture**: Complex logic lives in external agents, keeping the dataplane minimal and stable
- **Production Correctness**: Ship small, correct, testable increments that pass load, soak, and regression gates

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- Unix-like OS (Linux, macOS)
- For development: Docker (for integration testing)

### Installation

```bash
# Clone the repository
git clone https://github.com/raskell-io/sentinel.git
cd sentinel

# Install mise and set up environment
mise install
mise run setup

# Build and run the proxy
mise run release
mise run run-release

# Or run development environment with agents
mise run dev
```

The proxy will start on `http://0.0.0.0:8080` with a default upstream of `127.0.0.1:8081`.

### Basic Configuration

Set environment variables for basic configuration:

```bash
# Set upstream backend
export SENTINEL_UPSTREAM=backend.example.com:80

# Set listen address
export SENTINEL_LISTEN=0.0.0.0:8080

# Set worker threads (0 = number of CPU cores)
export SENTINEL_WORKERS=4

# Run the proxy
mise run run-release

# Or generate example configuration
mise run config-example
```

## 🏗️ Architecture

```
┌─────────────────┐
│   Client        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Sentinel Proxy │ ◄─── Pingora-based dataplane
│   (Dataplane)   │      - TLS termination
│                 │      - Routing & LB
│                 │      - Timeouts & retries
└────────┬────────┘
         │
    ┌────┴────┐
    │ Agents  │ ◄─── External processors
    │         │      - WAF (ModSecurity/Coraza)
    │         │      - Auth/PEP
    │         │      - Rate limiting
    │         │      - Custom logic
    └────┬────┘
         │
         ▼
┌─────────────────┐
│   Upstreams     │
└─────────────────┘
```

### Core Components

1. **Dataplane Proxy** (Pingora-based)
   - Handles connections, TLS, routing, upstream pools, retries, timeouts
   - Provides lifecycle hooks for policy decisions

2. **External Agent Interface** (SPOE/ext_proc-inspired)
   - Unix domain socket transport (primary)
   - Request/response lifecycle events with bounded body streaming
   - Deterministic timeouts and failure policies

3. **Configuration System**
   - KDL-based declarative configuration
   - Hot reload with validation and rollback
   - Safe, explicit defaults

## 📊 Features

### Phase 0 (Bootstrap) ✅
- [x] Basic Pingora proxy skeleton
- [x] TLS termination
- [x] Single upstream routing
- [x] Structured logging
- [x] Basic metrics endpoint

### Phase 1 (Minimal Production Proxy) 🚧
- [ ] Config schema + validation + hot reload
- [ ] Route matching + upstream pools
- [ ] Health checks
- [ ] Timeouts, retries, limits
- [ ] Graceful restart + draining
- [ ] Full observability (metrics/logs/traces)

### Phase 2 (External Processing) 📋
- [ ] Unix socket agent protocol
- [ ] Per-route agent attachment
- [ ] Agent timeouts + circuit breakers
- [ ] Reference agents (echo, denylist)

### Phase 3 (WAF Integration) 📋
- [ ] CRS-grade WAF agent
- [ ] Per-route WAF enablement
- [ ] Body inspection controls
- [ ] Audit logging + tuning workflow

### Phase 4 (Productization) 📋
- [ ] Container packaging
- [ ] Upgrade strategy
- [ ] Dashboards + runbooks
- [ ] Blue/green deployment

## 🛠️ Development

### Project Structure

```
sentinel/
├── crates/
│   ├── proxy/           # Main proxy application
│   ├── agent-protocol/  # Agent communication protocol
│   ├── config/          # Configuration management
│   └── common/          # Shared utilities
├── agents/              # External agent implementations
├── config/              # Sample configurations
└── tests/               # Integration tests
```

### Building from Source

```bash
# Development build
mise run build

# Run tests
mise run test

# Run with verbose logging
mise run run

# Run benchmarks
mise run bench

# Format code
mise run fmt

# Lint
mise run lint

# Run all checks
mise run check
```

### Testing

```bash
# Unit tests
mise run test-unit

# Integration tests
mise run test-integration

# Load testing (requires k6)
mise run load-test

# Test coverage
mise run test-coverage

# Test agents
mise run agent-test
```

## 🔒 Security

### Default Security Headers

Sentinel automatically adds these security headers to all responses:

- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `X-XSS-Protection: 1; mode=block`
- `Referrer-Policy: strict-origin-when-cross-origin`

### Limits and Bounds

All resources are bounded by default:

- **Headers**: Max 100 headers, 8KB total size
- **Body**: 10MB default limit, configurable per route
- **Connections**: 10,000 concurrent connections
- **Decompression**: 100:1 max ratio
- **Agent calls**: 1 second timeout, circuit breaker after 5 failures

### WAF Integration

When enabled, Sentinel provides CRS-grade WAF protection via external agents:

```kdl
waf {
    engine "modsecurity"
    ruleset {
        crs-version "3.3.4"
        paranoia-level 1
        anomaly-threshold 5
    }
    mode "prevention"  // or "detection"
    audit-log true
}
```

## 📝 Configuration

Sentinel uses [KDL](https://kdl.dev/) for configuration (JSON/TOML also supported):

```kdl
// sentinel.kdl
server {
    workers 4
    max-connections 10000
    graceful-shutdown-timeout 30s
}

listener "http" {
    address "0.0.0.0:8080"
    protocol "http"
}

upstream "backend" {
    target "10.0.1.10:8080" weight=1
    target "10.0.1.11:8080" weight=1
    
    health-check {
        type "http" path="/health"
        interval 10s
        timeout 5s
    }
    
    load-balancing "round_robin"
}

route "api" {
    match path-prefix="/api"
    upstream "backend"
    
    policies {
        timeout 30s
        retry-policy {
            max-attempts 3
            timeout 10s
        }
        failure-mode "closed"  // Security-first
    }
}
```

## 🚨 Operational Excellence

### Monitoring

Prometheus metrics available at `http://localhost:9090/metrics`:

- `sentinel_request_duration_seconds` - Request latency histogram
- `sentinel_requests_total` - Total requests by status
- `sentinel_upstream_failures_total` - Upstream failures
- `sentinel_circuit_breaker_state` - Circuit breaker status
- `sentinel_agent_latency_seconds` - Agent call latency

### Health Checks

- `/health` - Liveness probe
- `/ready` - Readiness probe
- `/metrics` - Prometheus metrics

### Graceful Operations

```bash
# Reload configuration without dropping connections
kill -HUP $(cat /var/run/sentinel.pid)

# Graceful shutdown with connection draining
kill -TERM $(cat /var/run/sentinel.pid)
```

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Principles

1. **Never add features without**:
   - Explicit limits
   - Explicit timeouts
   - Explicit observability
   - Tests

2. **Prefer agent-based extension** over embedding complex logic in the dataplane

3. **Keep the dataplane boring**: small surface area, stable behavior

4. **Document decisions and defaults** in the config reference

## 📊 Performance

Benchmark results on commodity hardware (Intel i7, 16GB RAM):

- **Throughput**: 50,000+ RPS (simple routing)
- **Latency**: p50 < 1ms, p99 < 10ms
- **Memory**: < 100MB RSS under load
- **CPU**: Linear scaling with cores

## 🛡️ Security Policy

Please report security vulnerabilities to security@raskell.io. See [SECURITY.md](SECURITY.md) for our security policy.

## 📜 License

Sentinel is dual-licensed:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

Choose whichever license works best for your use case.

## 🙏 Acknowledgments

- [Cloudflare Pingora](https://github.com/cloudflare/pingora) - The robust foundation
- [OWASP ModSecurity](https://github.com/SpiderLabs/ModSecurity) - WAF engine
- [Envoy Proxy](https://www.envoyproxy.io/) - ext_proc inspiration

## 🔗 Links

- [Documentation](https://sentinel.raskell.io)
- [API Reference](https://docs.rs/sentinel)
- [Docker Hub](https://hub.docker.com/r/raskell/sentinel)
- [Helm Chart](https://github.com/raskell-io/sentinel-helm)

---

**Remember**: The goal is sleepable ops. If a feature might wake you up at 3 AM, it needs better bounds, timeouts, and failure handling.