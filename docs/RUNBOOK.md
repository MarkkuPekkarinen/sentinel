# Sentinel Proxy - Operational Runbook (Phase 1)

## Table of Contents
1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Configuration Management](#configuration-management)
4. [Health Monitoring](#health-monitoring)
5. [Common Operations](#common-operations)
6. [Troubleshooting](#troubleshooting)
7. [Performance Tuning](#performance-tuning)
8. [Emergency Procedures](#emergency-procedures)
9. [Metrics and Alerting](#metrics-and-alerting)
10. [Maintenance Windows](#maintenance-windows)

---

## Overview

This runbook covers operational procedures for Sentinel Proxy Phase 1, which includes:
- ✅ Full route matching and upstream pool management
- ✅ Active and passive health checking
- ✅ Configuration hot reload with validation
- ✅ Graceful restart and connection draining
- ✅ Comprehensive observability (metrics/logs/traces)

### Key Principles
- **Sleepable Operations**: All operations are designed to be safe and predictable
- **Fail-Safe**: Default to fail-closed behavior for security
- **Observable**: Every operation produces metrics and logs
- **Rollback-Ready**: All changes can be safely reverted

---

## Quick Start

### Starting the Proxy

```bash
# Start with default configuration
./sentinel-proxy

# Start with specific config file
SENTINEL_CONFIG=/etc/sentinel/config.kdl ./sentinel-proxy

# Start with verbose logging
RUST_LOG=debug ./sentinel-proxy

# Start as daemon (systemd)
systemctl start sentinel
```

### Checking Status

```bash
# Health check
curl -f http://localhost:8080/health

# Readiness check
curl -f http://localhost:8080/ready

# Metrics
curl http://localhost:9090/metrics

# Check logs
journalctl -u sentinel -f

# Check active connections
ss -tnp | grep sentinel
```

### Stopping the Proxy

```bash
# Graceful shutdown (drains connections)
kill -TERM $(cat /var/run/sentinel.pid)

# Force shutdown (emergency only)
kill -9 $(cat /var/run/sentinel.pid)

# Via systemd
systemctl stop sentinel
```

---

## Configuration Management

### Configuration Hot Reload

The proxy supports zero-downtime configuration reloading:

```bash
# Trigger reload via signal
kill -HUP $(cat /var/run/sentinel.pid)

# Verify reload succeeded
tail -f /var/log/sentinel/sentinel.log | grep "Configuration reload completed"
```

### Configuration Validation

Always validate configuration before applying:

```bash
# Dry-run validation
./sentinel-proxy --validate --config /etc/sentinel/new-config.kdl

# Check current configuration
./sentinel-proxy --print-config
```

### Configuration Rollback

If a configuration causes issues:

```bash
# Automatic rollback on validation failure
# The proxy keeps the previous config and logs the error

# Manual rollback
cp /etc/sentinel/config.kdl.backup /etc/sentinel/config.kdl
kill -HUP $(cat /var/run/sentinel.pid)
```

### Configuration Best Practices

1. **Always keep backups**:
   ```bash
   cp /etc/sentinel/config.kdl /etc/sentinel/config.kdl.$(date +%Y%m%d-%H%M%S)
   ```

2. **Test in staging first**:
   ```bash
   # Apply to staging
   scp new-config.kdl staging:/etc/sentinel/config.kdl
   ssh staging "kill -HUP \$(cat /var/run/sentinel.pid)"
   ```

3. **Monitor after changes**:
   ```bash
   # Watch error rate
   watch -n 1 'curl -s localhost:9090/metrics | grep error_rate'
   ```

---

## Health Monitoring

### Active Health Checks

Configured per upstream in `config.kdl`:

```kdl
health-check {
    type "http" {
        path "/health"
        expected-status 200
    }
    interval-secs 10
    timeout-secs 5
    healthy-threshold 2    // Mark healthy after 2 successes
    unhealthy-threshold 3  // Mark unhealthy after 3 failures
}
```

### Passive Health Checks

Automatically enabled, monitoring real traffic:
- Failure rate threshold: 50% over 100 requests
- Automatic marking of unhealthy targets
- Works in conjunction with active checks

### Checking Upstream Health

```bash
# View upstream health status
curl -s localhost:9090/metrics | grep upstream_health

# Check specific upstream
curl -s localhost:9090/admin/upstreams/api-backend/health

# Force health check
curl -X POST localhost:9090/admin/upstreams/api-backend/check
```

### Circuit Breaker Status

```bash
# Check circuit breaker states
curl -s localhost:9090/metrics | grep circuit_breaker_state

# View circuit breaker trips
curl -s localhost:9090/metrics | grep circuit_breaker_trips_total
```

---

## Common Operations

### Adding a New Route

1. Edit configuration:
```kdl
route "new-api" {
    priority "normal"
    matches {
        path-prefix "/new-api/"
    }
    upstream "api-backend"
    policies {
        timeout-secs 30
        failure-mode "closed"
    }
}
```

2. Validate and reload:
```bash
./sentinel-proxy --validate --config /etc/sentinel/config.kdl
kill -HUP $(cat /var/run/sentinel.pid)
```

### Adding/Removing Upstream Targets

1. Update upstream configuration:
```kdl
upstream "api-backend" {
    targets {
        target {
            address "10.1.1.12:8080"  // New target
            weight 1
        }
        // ... existing targets
    }
}
```

2. Reload configuration:
```bash
kill -HUP $(cat /var/run/sentinel.pid)
```

### Draining a Target

1. Set weight to 0:
```kdl
target {
    address "10.1.1.10:8080"
    weight 0  // No new connections
}
```

2. Wait for connections to drain:
```bash
# Monitor active connections
watch -n 1 'ss -tn | grep 10.1.1.10:8080 | wc -l'
```

3. Remove target and reload

### Enabling Debug Logging

```bash
# Temporary debug logging
RUST_LOG=debug kill -USR1 $(cat /var/run/sentinel.pid)

# Disable debug logging
kill -USR2 $(cat /var/run/sentinel.pid)
```

---

## Troubleshooting

### High Error Rate

**Symptoms**: Increased 5xx errors, timeouts

**Investigation**:
```bash
# Check error metrics
curl -s localhost:9090/metrics | grep 'requests_total.*5[0-9][0-9]'

# Check upstream health
curl -s localhost:9090/metrics | grep upstream_health

# Review recent logs
tail -n 1000 /var/log/sentinel/sentinel.log | grep ERROR

# Check circuit breakers
curl -s localhost:9090/metrics | grep circuit_breaker_state
```

**Actions**:
1. Identify unhealthy upstreams
2. Check upstream server logs
3. Verify network connectivity
4. Consider increasing timeouts temporarily

### Memory Issues

**Symptoms**: High memory usage, OOM kills

**Investigation**:
```bash
# Check memory usage
curl -s localhost:9090/metrics | grep memory_usage

# Check connection count
curl -s localhost:9090/metrics | grep open_connections

# Check for memory leaks
pmap -x $(pidof sentinel-proxy) | tail -n 1
```

**Actions**:
1. Review connection limits in config
2. Check for request body buffering
3. Restart proxy if necessary (gracefully)

### Configuration Reload Failures

**Symptoms**: Config changes not taking effect

**Investigation**:
```bash
# Check reload metrics
curl -s localhost:9090/metrics | grep reload

# Review logs
grep "Configuration" /var/log/sentinel/sentinel.log | tail -20

# Validate configuration
./sentinel-proxy --validate --config /etc/sentinel/config.kdl
```

**Actions**:
1. Fix configuration errors
2. Ensure file permissions are correct
3. Check available disk space
4. Try manual reload

### Connection Refused

**Symptoms**: Clients getting connection refused

**Investigation**:
```bash
# Check if proxy is running
ps aux | grep sentinel-proxy

# Check listening ports
ss -tlnp | grep sentinel

# Check connection limits
curl -s localhost:9090/metrics | grep connections

# Review logs for bind errors
grep "bind\|listen" /var/log/sentinel/sentinel.log
```

**Actions**:
1. Verify proxy is running
2. Check firewall rules
3. Verify listen addresses in config
4. Check ulimits

---

## Performance Tuning

### System Tuning

```bash
# Increase file descriptors
ulimit -n 65535

# Kernel tuning (add to /etc/sysctl.conf)
net.ipv4.tcp_fin_timeout = 30
net.ipv4.tcp_tw_reuse = 1
net.core.somaxconn = 65535
net.ipv4.tcp_max_syn_backlog = 65535
net.core.netdev_max_backlog = 65535

# Apply settings
sysctl -p
```

### Proxy Tuning

```kdl
// Optimize for high throughput
server {
    worker-threads 0  // Use all CPU cores
    max-connections 50000
}

// Optimize connection pooling
connection-pool {
    max-connections 200
    max-idle 50
    idle-timeout-secs 120
}

// Adjust timeouts for fast failures
timeouts {
    connect-secs 5
    request-secs 30
    read-secs 15
    write-secs 15
}
```

### Monitoring Performance

```bash
# Request latency percentiles
curl -s localhost:9090/metrics | grep request_duration | grep quantile

# Throughput
curl -s localhost:9090/metrics | grep requests_total

# Connection pool efficiency
curl -s localhost:9090/metrics | grep connection_pool

# CPU and memory
top -p $(pidof sentinel-proxy)
```

---

## Emergency Procedures

### Total Upstream Failure

**Immediate Actions**:
1. Check upstream health:
   ```bash
   curl -s localhost:9090/admin/upstreams/*/health
   ```

2. Enable emergency static response:
   ```kdl
   route "emergency" {
       priority "critical"
       matches { path-prefix "/" }
       upstream "emergency-static"
   }
   ```

3. Notify on-call team

### Memory Exhaustion

**Immediate Actions**:
1. Reduce connection limits:
   ```bash
   echo 'limits { max-total-connections 1000 }' >> /tmp/emergency.kdl
   cat /etc/sentinel/config.kdl >> /tmp/emergency.kdl
   mv /tmp/emergency.kdl /etc/sentinel/config.kdl
   kill -HUP $(cat /var/run/sentinel.pid)
   ```

2. Force garbage collection:
   ```bash
   kill -USR1 $(cat /var/run/sentinel.pid)
   ```

3. If critical, rolling restart

### DDoS Attack

**Immediate Actions**:
1. Enable rate limiting:
   ```kdl
   policies {
       rate-limit {
           requests-per-second 10
           key "client_ip"
       }
   }
   ```

2. Block suspicious IPs:
   ```bash
   # Add to firewall
   iptables -A INPUT -s $ATTACKER_IP -j DROP
   ```

3. Enable challenge mode (Phase 3)

---

## Metrics and Alerting

### Key Metrics to Monitor

| Metric | Alert Threshold | Action |
|--------|----------------|--------|
| Error rate | > 1% | Investigate upstreams |
| p99 latency | > 1s | Check slow upstreams |
| Active connections | > 80% limit | Scale or increase limits |
| Circuit breaker trips | > 10/min | Check upstream health |
| Memory usage | > 80% | Review configuration |
| Reload failures | Any | Check config validity |

### Prometheus Alerts

```yaml
groups:
  - name: sentinel
    rules:
      - alert: HighErrorRate
        expr: rate(sentinel_requests_total{status=~"5.."}[1m]) > 0.01
        for: 2m
        annotations:
          summary: "High error rate detected"

      - alert: UpstreamUnhealthy
        expr: sentinel_upstream_health == 0
        for: 1m
        annotations:
          summary: "Upstream {{ $labels.upstream }} is unhealthy"

      - alert: CircuitBreakerOpen
        expr: sentinel_circuit_breaker_state == 1
        for: 30s
        annotations:
          summary: "Circuit breaker open for {{ $labels.component }}"
```

### Dashboard Queries

```sql
-- Request rate
rate(sentinel_requests_total[1m])

-- Error rate
rate(sentinel_requests_total{status=~"5.."}[1m]) / rate(sentinel_requests_total[1m])

-- p50, p95, p99 latency
histogram_quantile(0.5, rate(sentinel_request_duration_seconds_bucket[5m]))
histogram_quantile(0.95, rate(sentinel_request_duration_seconds_bucket[5m]))
histogram_quantile(0.99, rate(sentinel_request_duration_seconds_bucket[5m]))

-- Upstream health
avg by (upstream) (sentinel_upstream_health)
```

---

## Maintenance Windows

### Pre-Maintenance Checklist

- [ ] Announce maintenance window
- [ ] Prepare rollback plan
- [ ] Verify backup configuration
- [ ] Test changes in staging
- [ ] Document changes
- [ ] Notify on-call team

### During Maintenance

1. **Enable maintenance mode**:
   ```kdl
   route "maintenance" {
       priority "critical"
       matches { path-prefix "/" }
       upstream "maintenance-page"
   }
   ```

2. **Monitor closely**:
   ```bash
   tail -f /var/log/sentinel/sentinel.log
   watch -n 1 'curl -s localhost:9090/metrics | grep error'
   ```

3. **Test after changes**:
   ```bash
   ./test-suite.sh
   ```

### Post-Maintenance

- [ ] Verify all services healthy
- [ ] Check error rates normal
- [ ] Review logs for issues
- [ ] Update documentation
- [ ] Send completion notice

---

## Appendix

### Useful Commands

```bash
# Get current version
./sentinel-proxy --version

# Test configuration
./sentinel-proxy --test-config /etc/sentinel/config.kdl

# Dump current routes
curl localhost:9090/admin/routes

# Force health check all upstreams
curl -X POST localhost:9090/admin/health-check

# Get correlation ID for request
grep "correlation_id" /var/log/sentinel/access.log | tail -1

# Track request through logs
grep "$CORRELATION_ID" /var/log/sentinel/*.log
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SENTINEL_CONFIG` | Config file path | `config/sentinel.kdl` |
| `SENTINEL_WORKERS` | Worker threads | CPU cores |
| `RUST_LOG` | Log level | `info` |
| `SENTINEL_METRICS_ADDR` | Metrics address | `0.0.0.0:9090` |

### Support Contacts

- On-call: Use PagerDuty
- Slack: #sentinel-ops
- Documentation: https://sentinel.internal/docs
- Runbook: This document

---

**Remember**: The goal is sleepable operations. If something might wake you at 3 AM, add better bounds, timeouts, and failure handling.