# Sentinel 0.3.0 Roadmap - DRAFT

**Status:** Planning
**Target:** 0.3.0
**Theme:** Dynamic Operations & Cloud-Native

---

## Feature Evaluation Matrix

### Impact vs Effort Analysis

| Feature | Impact | Effort | Risk | Priority | Notes |
|---------|--------|--------|------|----------|-------|
| **Protocol Modernization** |||||
| HTTP/3 / QUIC | HIGH | HIGH | MEDIUM | P2 | Pingora has experimental support |
| gRPC proxying | MEDIUM | MEDIUM | LOW | P2 | HTTP/2 already works, need proto awareness |
| WebTransport | LOW | HIGH | HIGH | P4 | Requires HTTP/3, spec evolving |
| **Control Plane** |||||
| REST Admin API | HIGH | MEDIUM | LOW | **P1** | Dynamic config, better ops |
| Dynamic upstream mgmt | HIGH | MEDIUM | LOW | **P1** | Add/remove backends live |
| Live agent registration | MEDIUM | MEDIUM | LOW | P2 | Agents connect dynamically |
| Admin UI | MEDIUM | HIGH | LOW | P3 | Nice to have, not critical |
| **Cloud-Native** |||||
| Helm chart | HIGH | LOW | LOW | **P1** | Easy win, high value |
| Kubernetes Operator | HIGH | HIGH | MEDIUM | P2 | Native K8s experience |
| Gateway API support | HIGH | HIGH | MEDIUM | P2 | K8s standard, complex spec |
| Service mesh docs | MEDIUM | LOW | LOW | **P1** | Already done in 0.2.0 |
| **Extensions** |||||
| WASM plugin support | HIGH | HIGH | MEDIUM | P2 | In-process, no IPC overhead |
| Plugin SDK (Rust) | MEDIUM | MEDIUM | LOW | P2 | Better agent DX |
| Plugin marketplace | LOW | HIGH | MEDIUM | P4 | Infrastructure needed |
| **Enterprise** |||||
| Multi-tenancy | HIGH | HIGH | MEDIUM | P3 | Architectural changes |
| RBAC for admin API | MEDIUM | MEDIUM | LOW | P2 | Depends on admin API |
| Audit log shipping | MEDIUM | MEDIUM | LOW | P3 | Kafka/S3 integration |
| Usage metering hooks | LOW | MEDIUM | LOW | P4 | Commercial only |
| **Performance** |||||
| Adaptive LB integration | HIGH | MEDIUM | LOW | **P1** | Already built, not wired |
| Per-agent queue isolation | MEDIUM | MEDIUM | LOW | **P1** | Technical debt |
| Pool config exposure | LOW | LOW | LOW | **P1** | Quick fix |
| gRPC health checks | MEDIUM | MEDIUM | LOW | P2 | Protocol compliance |
| Zero-alloc hot path | MEDIUM | HIGH | LOW | P3 | Deep optimization |

---

## Proposed 0.3.0 Scope

### Must Have (P1) - Core Release

These items define 0.3.0 and should all be completed:

#### 1. Control Plane API
**Effort:** 2-3 weeks | **Impact:** HIGH

REST API for runtime management without config file changes.

```
POST   /api/v1/upstreams              # Add upstream
DELETE /api/v1/upstreams/{id}         # Remove upstream
PUT    /api/v1/upstreams/{id}/targets # Update targets
GET    /api/v1/upstreams              # List upstreams
POST   /api/v1/upstreams/{id}/drain   # Drain upstream

GET    /api/v1/routes                 # List routes
PUT    /api/v1/routes/{id}/enabled    # Enable/disable route

GET    /api/v1/agents                 # List agents
GET    /api/v1/agents/{id}/status     # Agent health

POST   /api/v1/config/reload          # Trigger reload
GET    /api/v1/config/validate        # Validate config file
```

**Files to create/modify:**
- `crates/proxy/src/admin_api.rs` - API handlers
- `crates/proxy/src/admin_api/upstreams.rs` - Upstream management
- `crates/proxy/src/admin_api/routes.rs` - Route management
- `crates/proxy/src/admin_api/agents.rs` - Agent status
- `crates/config/src/api.rs` - API config types

**KDL Configuration:**
```kdl
admin-api {
    enabled true
    address "127.0.0.1:9901"
    auth {
        type "bearer"
        token "${ADMIN_API_TOKEN}"
    }
}
```

#### 2. Dynamic Upstream Management
**Effort:** 1-2 weeks | **Impact:** HIGH

Add/remove/update upstream targets at runtime without reload.

**Capabilities:**
- Add new upstream dynamically
- Add/remove targets from existing upstream
- Update target weights
- Drain targets gracefully before removal
- Persist changes to config file (optional)

**Files to modify:**
- `crates/proxy/src/upstream/mod.rs` - Add dynamic mutation methods
- `crates/proxy/src/upstream/pool.rs` - Thread-safe target updates

#### 3. Helm Chart
**Effort:** 3-5 days | **Impact:** HIGH

Production-ready Helm chart for Kubernetes deployment.

**Features:**
- Deployment with configurable replicas
- Service (ClusterIP, LoadBalancer, NodePort)
- ConfigMap for sentinel.kdl
- Secret for TLS certs and tokens
- HPA for autoscaling
- PodDisruptionBudget
- ServiceMonitor for Prometheus Operator
- Ingress resource (optional)

**Repository:** `raskell-io/sentinel-helm`

#### 4. Adaptive Load Balancing Integration
**Effort:** 3-5 days | **Impact:** HIGH

Wire existing adaptive LB code into request path.

**Current state:** Algorithm implemented but not used
**Task:** Integrate into upstream selection, add metrics

**Files to modify:**
- `crates/proxy/src/upstream/adaptive.rs` - Already exists
- `crates/proxy/src/upstream/mod.rs` - Wire into selection
- `crates/proxy/src/proxy/http_trait.rs` - Feed latency data

#### 5. Technical Debt Cleanup
**Effort:** 1 week | **Impact:** MEDIUM

| Item | Task |
|------|------|
| Per-agent queue isolation | Replace global semaphore with per-agent queues |
| Pool config exposure | Move hardcoded values to KDL config |
| Async pool shutdown | Add graceful task cancellation |

---

### Should Have (P2) - Stretch Goals

Include if time permits, otherwise defer to 0.4.0:

#### 6. WASM Plugin Support
**Effort:** 3-4 weeks | **Impact:** HIGH

Run user-defined logic in-process via WebAssembly.

**Benefits over agents:**
- No IPC overhead (~200μs saved per call)
- Single binary deployment
- Sandboxed execution

**Approach:**
- Use `wasmtime` runtime
- Define host functions for request/response access
- Support Rust and Go plugin compilation to WASM

**Plugin interface:**
```rust
// Plugin exports these functions
fn on_request_headers(headers: &Headers) -> Decision;
fn on_request_body(body: &[u8]) -> Decision;
fn on_response_headers(headers: &Headers) -> Decision;
```

#### 7. Kubernetes Operator
**Effort:** 4-6 weeks | **Impact:** HIGH

Native Kubernetes integration with CRDs.

**Custom Resources:**
- `SentinelProxy` - Main proxy deployment
- `SentinelRoute` - Route configuration
- `SentinelUpstream` - Upstream configuration
- `SentinelAgent` - Agent deployment

**Features:**
- Reconciliation loop
- Status conditions
- Event recording
- Leader election for HA

**Repository:** `raskell-io/sentinel-operator`

#### 8. HTTP/3 / QUIC Support
**Effort:** 3-4 weeks | **Impact:** HIGH

Modern protocol support for better mobile/lossy network performance.

**Dependencies:**
- Pingora QUIC support (experimental)
- `quinn` or `quiche` crate

**Scope:**
- QUIC listener
- HTTP/3 downstream
- HTTP/3 upstream (optional)
- 0-RTT connection resumption

#### 9. gRPC Proxying
**Effort:** 2-3 weeks | **Impact:** MEDIUM

Native gRPC support with streaming.

**Features:**
- Unary RPC
- Server streaming
- Client streaming
- Bidirectional streaming
- gRPC-Web support
- Reflection API passthrough

#### 10. Live Agent Registration
**Effort:** 1-2 weeks | **Impact:** MEDIUM

Agents can connect/disconnect without proxy restart.

**Flow:**
1. Agent connects to registration endpoint
2. Proxy validates agent identity
3. Agent added to available pool
4. Health checks begin
5. Agent available for routing

---

### Nice to Have (P3) - Future

Defer to 0.4.0 or later:

#### 11. Admin UI
Web-based dashboard for proxy management.

**Features:**
- Real-time metrics visualization
- Route/upstream management
- Agent status monitoring
- Log viewer
- Config editor

**Tech:** React/Vue + REST API

#### 12. Multi-tenancy
Isolated configurations per tenant.

**Features:**
- Tenant-scoped routes
- Tenant-scoped upstreams
- Tenant-scoped rate limits
- Tenant identification (header, JWT claim, subdomain)

#### 13. Audit Log Shipping
Direct integration with log aggregators.

**Backends:**
- Kafka
- AWS S3 / GCS
- Elasticsearch
- Datadog

#### 14. Gateway API Support
Kubernetes Gateway API (sig-network standard).

**Resources:**
- GatewayClass
- Gateway
- HTTPRoute
- TCPRoute
- TLSRoute

---

### Out of Scope (P4)

Not planned for 0.3.0:

- WebTransport (requires HTTP/3 maturity)
- Plugin marketplace (needs ecosystem first)
- Usage metering (commercial feature)
- GraphQL-aware routing (niche)

---

## Release Criteria

### 0.3.0 Release Requirements

- [ ] Control Plane API functional with auth
- [ ] Dynamic upstream add/remove working
- [ ] Helm chart published to artifact hub
- [ ] Adaptive LB integrated and tested
- [ ] Technical debt items resolved
- [ ] All P1 items complete
- [ ] Documentation updated
- [ ] Migration guide from 0.2.0

### Quality Gates

- All existing tests pass
- New features have >80% test coverage
- Load test: No regression from 0.2.0 baseline
- Soak test: 1-hour stability verified
- Security review for admin API

---

## Timeline (Suggested)

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| Phase 1 | 2 weeks | Control Plane API |
| Phase 2 | 1 week | Dynamic Upstreams |
| Phase 3 | 1 week | Helm Chart |
| Phase 4 | 1 week | Adaptive LB + Tech Debt |
| Phase 5 | 1 week | Testing + Docs |
| **Total** | **6 weeks** | **0.3.0 Release** |

P2 items (WASM, Operator, HTTP/3) can run in parallel or defer to 0.3.x/0.4.0.

---

## Open Questions

1. **Admin API auth**: Bearer token vs mTLS vs both?
2. **Config persistence**: Should API changes write back to config file?
3. **Helm repo**: Use GitHub Pages or Artifact Hub directly?
4. **WASM runtime**: wasmtime vs wasmer vs wasm3?
5. **Operator scope**: Full CRD or just deployment helper?

---

## Appendix: Effort Estimates

| Effort | Duration | Examples |
|--------|----------|----------|
| LOW | 1-3 days | Config exposure, docs |
| MEDIUM | 1-2 weeks | API endpoints, integrations |
| HIGH | 3-6 weeks | New subsystems, protocols |

| Risk | Description |
|------|-------------|
| LOW | Well-understood, isolated changes |
| MEDIUM | Dependencies on external code, moderate complexity |
| HIGH | Unproven technology, spec instability |
