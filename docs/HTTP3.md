# HTTP/3 Support in Sentinel

HTTP/3 is the latest version of the Hypertext Transfer Protocol, built on QUIC (Quick UDP Internet Connections) transport protocol. Sentinel provides experimental HTTP/3 support to enable better performance, especially over lossy networks.

## Status

⚠️ **Current Status**: Prepared for future integration

While Sentinel has HTTP/3 configuration and infrastructure in place, full support depends on:
- Pingora adding native HTTP/3 support (planned as long-term goal)
- Or integration with alternative QUIC implementations (quinn, quiche)

The current implementation provides:
- Configuration schema for HTTP/3
- Alt-Svc header advertisement
- Metrics and monitoring preparation
- Graceful fallback to HTTP/2 and HTTP/1.1

## Benefits of HTTP/3

### 1. Eliminated Head-of-Line Blocking
- Independent stream delivery
- One lost packet doesn't block other streams
- Better performance on unreliable networks

### 2. Faster Connection Establishment
- 0-RTT connection resumption
- Combined crypto and transport handshake
- Reduced latency for repeat visitors

### 3. Connection Migration
- Seamless network transitions (WiFi → Cellular)
- Connection survives IP address changes
- Better mobile experience

### 4. Improved Security
- Encryption by default (TLS 1.3 required)
- Better privacy with encrypted headers
- Protection against protocol ossification

## Configuration

### Basic HTTP/3 Setup

```kdl
listeners {
    listener "http3" {
        address "0.0.0.0:443"
        protocol "h3"
        
        tls {
            cert_file "/etc/sentinel/certs/cert.pem"
            key_file "/etc/sentinel/certs/key.pem"
            min_version "TLSv1.3"  // Required for QUIC
            
            // ALPN protocols for HTTP/3
            alpn_protocols ["h3", "h3-29"]
        }
        
        quic {
            max_idle_timeout_ms 30000
            initial_max_data 10485760  // 10MB
            initial_max_streams_bidi 100
            congestion_control "cubic"
        }
    }
}
```

### Advertising HTTP/3 via Alt-Svc

For clients to discover HTTP/3 support, configure Alt-Svc headers:

```kdl
listener "https" {
    address "0.0.0.0:443"
    protocol "https"
    
    // Advertise HTTP/3 availability
    alt_svc {
        enabled true
        h3_port 443
        max_age 86400  // 24 hours
        persist true
    }
}
```

### 0-RTT (Zero Round Trip Time) Setup

Enable 0-RTT for faster subsequent connections:

```kdl
tls {
    early_data {
        enabled true
        max_size 16384
        replay_protection true
    }
}

// Route-specific 0-RTT control
route "api" {
    zero_rtt {
        enabled true
        allowed_methods ["GET", "HEAD"]  // Only safe methods
        reject_authenticated true  // Security consideration
    }
}
```

## Performance Tuning

### QUIC Transport Parameters

```kdl
quic {
    // Connection tuning
    initial_rtt_ms 100  // Adjust based on expected latency
    max_idle_timeout_ms 30000
    
    // Flow control
    initial_max_data 10485760  // 10MB
    initial_max_stream_data_bidi_local 1048576  // 1MB per stream
    
    // Packet sizing
    max_udp_payload_size 1350  // Conservative for compatibility
    
    // Congestion control
    congestion_control "bbr"  // Options: cubic, bbr, new_reno
}
```

### Buffer Sizing

```kdl
performance {
    recv_buffer_size 2097152  // 2MB
    send_buffer_size 2097152  // 2MB
    
    // Batch processing for efficiency
    batch_size 16
    
    // Enable GSO for better throughput
    enable_gso true
    gso_max_size 65536
}
```

### Connection Migration

```kdl
quic {
    migration {
        enabled true
        allow_network_change true
        path_validation true  // Validate new paths
    }
}
```

## Monitoring

### Metrics

HTTP/3 specific metrics available at `/metrics`:

```
# Connection metrics
sentinel_http3_connections_total
sentinel_http3_connections_active
sentinel_http3_connections_migrated_total
sentinel_http3_0rtt_accepted_total
sentinel_http3_0rtt_rejected_total

# Performance metrics
sentinel_http3_rtt_ms
sentinel_http3_packet_loss_rate
sentinel_http3_bytes_sent_total
sentinel_http3_bytes_received_total

# Stream metrics
sentinel_http3_streams_opened_total
sentinel_http3_streams_closed_total
sentinel_http3_streams_reset_total
```

### Debug Logging

Enable QUIC debug logging for troubleshooting:

```kdl
logging {
    quic_debug {
        enabled true
        log_packets false  // Very verbose
        log_frames false   // Very verbose
        log_congestion true
        log_recovery true
        log_handshake true
        log_migration true
    }
}
```

### Tracing

Trace HTTP/3 specific events:

```kdl
tracing {
    http3_tracing {
        trace_0rtt true
        trace_migration true
        trace_congestion true
        trace_stream_events true
    }
}
```

## WebTransport Support

HTTP/3 enables WebTransport for bidirectional communication:

```kdl
route "webtransport" {
    matches {
        path "/webtransport"
        header { name ":protocol" value "webtransport" }
    }
    
    webtransport {
        max_sessions_per_client 10
        session_timeout_secs 3600
        
        // Stream configuration
        max_bidirectional_streams 100
        max_unidirectional_streams 100
        
        // Datagram support
        enable_datagrams true
        max_datagram_size 1200
    }
}
```

## Security Considerations

### Address Validation

Prevent amplification attacks:

```kdl
security {
    require_address_validation true
    amplification_limit 3  // Max 3x response before validation
}
```

### Stateless Reset

Configure stateless reset for connection recovery:

```kdl
security {
    stateless_reset {
        enabled true
        key_rotation_interval_secs 86400
    }
}
```

### 0-RTT Replay Protection

```kdl
early_data {
    replay_protection true
    replay_cache_size 10000
    replay_window_secs 60
}
```

## Fallback Strategy

Configure graceful fallback when HTTP/3 fails:

```kdl
http3_global {
    fallback {
        enabled true
        protocols ["h2", "http/1.1"]  // Fallback order
        retry_after_secs 300  // Retry HTTP/3 after 5 minutes
    }
}
```

## Client Configuration

### Browser Support

Modern browsers with HTTP/3 support:
- Chrome 87+ (enabled by default)
- Firefox 88+ (enabled by default)
- Safari 14+ (experimental)
- Edge 87+ (enabled by default)

### Testing HTTP/3

Test HTTP/3 connectivity:

```bash
# Using curl with HTTP/3 support
curl --http3 https://example.com

# Chrome with logging
google-chrome --enable-logging --log-level=0 \
  --enable-quic --quic-version=h3

# Firefox
about:config → network.http.http3.enabled = true
```

### Client Libraries

- **Rust**: quinn, quiche
- **Go**: quic-go
- **Python**: aioquic
- **Node.js**: node-quiche

## Troubleshooting

### Common Issues

#### 1. HTTP/3 Not Negotiated

Check:
- UDP port 443 is open
- Firewall allows UDP traffic
- Alt-Svc header is present
- Client supports HTTP/3

```bash
# Check Alt-Svc header
curl -I https://example.com | grep alt-svc

# Test UDP connectivity
nc -u -v example.com 443
```

#### 2. High Packet Loss

Monitor and adjust:
```kdl
quic {
    congestion_control "bbr"  // Better for lossy networks
    loss_detection {
        packet_threshold 3
        time_threshold 1.125
    }
}
```

#### 3. 0-RTT Failures

Check replay protection:
```kdl
early_data {
    replay_protection true
    replay_cache_size 10000  // Increase if needed
}
```

#### 4. Connection Migration Issues

```kdl
migration {
    path_validation true  // Ensure path validation
    // Disable if causing issues
    enabled false
}
```

### Debug Tools

- **qlog**: QUIC logging format for analysis
- **Wireshark**: QUIC protocol dissector
- **Chrome DevTools**: Protocol tab shows HTTP/3
- **Firefox DevTools**: Network tab with HTTP version

## Performance Comparison

| Metric | HTTP/1.1 | HTTP/2 | HTTP/3 |
|--------|----------|---------|---------|
| Connection Setup | 2-3 RTT | 2-3 RTT | 0-1 RTT |
| Head-of-Line Blocking | Yes | Yes (TCP) | No |
| Packet Loss Impact | High | High | Low |
| Connection Migration | No | No | Yes |
| CPU Usage | Low | Medium | Medium-High |
| Memory Usage | Low | Medium | Medium |

## Best Practices

### 1. Enable Selectively

Start with specific routes:
```kdl
route "performance-critical" {
    protocols ["h3", "h2", "http/1.1"]  // Priority order
}
```

### 2. Monitor Carefully

Track success rates:
- Connection establishment rate
- 0-RTT acceptance rate
- Fallback frequency
- Performance metrics

### 3. Optimize for Your Use Case

**High Latency Networks**:
```kdl
quic {
    initial_rtt_ms 500
    max_idle_timeout_ms 60000
}
```

**Mobile Networks**:
```kdl
migration { enabled true }
congestion_control "bbr"
```

**CDN/Static Content**:
```kdl
zero_rtt { enabled true }
max_udp_payload_size 1450
```

### 4. Security First

- Always validate addresses
- Limit 0-RTT to safe operations
- Monitor for amplification attacks
- Rotate stateless reset keys

## Future Roadmap

### Short Term
- Complete Pingora HTTP/3 integration
- Production testing with select routes
- Performance benchmarking

### Medium Term
- WebTransport support
- Multipath QUIC experiments
- Enhanced congestion control

### Long Term
- MASQUE protocol support
- Quantum-resistant cryptography
- Full HTTP/3 push support

## Migration Guide

### From HTTP/2 to HTTP/3

1. **Add HTTP/3 listener** alongside existing HTTPS
2. **Configure Alt-Svc** headers
3. **Monitor** adoption and performance
4. **Gradually increase** max_age
5. **Optimize** based on metrics

### Configuration Example

```kdl
// Phase 1: Testing (small percentage)
alt_svc {
    max_age 300  // 5 minutes
}

// Phase 2: Validation (increasing adoption)
alt_svc {
    max_age 3600  // 1 hour
}

// Phase 3: Production (full adoption)
alt_svc {
    max_age 86400  // 24 hours
    persist true
}
```

## References

- [RFC 9114](https://www.rfc-editor.org/rfc/rfc9114.html) - HTTP/3
- [RFC 9000](https://www.rfc-editor.org/rfc/rfc9000.html) - QUIC Transport
- [RFC 9001](https://www.rfc-editor.org/rfc/rfc9001.html) - QUIC TLS
- [WebTransport](https://w3c.github.io/webtransport/) - W3C Draft
- [QUIC Working Group](https://quicwg.org/) - IETF QUIC WG